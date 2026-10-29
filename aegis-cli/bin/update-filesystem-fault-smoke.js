#!/usr/bin/env node
import {mkdtemp,mkdir,writeFile,readFile,open,rm,access,realpath,statfs} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {generateKeyPairSync,createHash,sign} from 'node:crypto';
import {spawnSync,execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
// Never mount or fill a filesystem on a developer's workstation.
if(process.platform!=='linux'||process.env.GITHUB_ACTIONS!=='true'||process.env.RUNNER_ENVIRONMENT!=='github-hosted') throw new Error('Filesystem fault acceptance requires a disposable GitHub-hosted Linux runner');
const uid=process.getuid(),gid=process.getgid();
assert.ok(Number.isSafeInteger(uid)&&uid>0&&Number.isSafeInteger(gid)&&gid>=0);
const parent=await realpath(tmpdir());
const root=await mkdtemp(path.join(parent,'aegis-update-fault-'));
const mountPoint=path.join(root,'limited');let mounted=false;
const run=(program,args)=>execFileSync(program,args,{encoding:'utf8',timeout:30000,maxBuffer:1024*1024});
function owned() {
 const relative=path.relative(parent,root);
 assert.ok(relative&&!relative.startsWith('..')&&!path.isAbsolute(relative));
 assert.match(path.basename(root),/^aegis-update-fault-/);
 assert.equal(path.dirname(mountPoint),root);
}
try {
 owned();await mkdir(mountPoint);
 const artifact=path.join(root,'download.bin'),envelopeFile=path.join(root,'envelope.json'),keyFile=path.join(root,'public.pem'),policyFile=path.join(root,'policy.json');
 const content=Buffer.alloc(8*1024*1024,0x41),keys=generateKeyPairSync('ed25519');
 const policy={appId:'org.aegis.ci.fault',platform:'linux',architecture:process.arch,currentVersion:'1.0.0',maxSeenSequence:1};
 const manifest={schemaVersion:1,appId:policy.appId,platform:policy.platform,architecture:policy.architecture,version:'2.0.0',sequence:2,expiresAt:Date.now()+3600000,artifact:{size:content.length,sha256:createHash('sha256').update(content).digest('hex')}};
 await writeFile(artifact,content);await writeFile(keyFile,keys.publicKey.export({format:'pem',type:'spki'}));
 const policyBytes=JSON.stringify(policy);await writeFile(policyFile,policyBytes);
 async function publish(value) {
  const bytes=Buffer.from(JSON.stringify(value));
  await writeFile(envelopeFile,JSON.stringify({schemaVersion:1,payload:bytes.toString('base64'),signature:sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),bytes]),keys.privateKey).toString('base64')}));
 }
 await publish(manifest);
 const cli=path.resolve('aegis-cli/bin/aegis.js');
 const invoke=args=>spawnSync(process.execPath,[cli,...args],{encoding:'utf8',timeout:30000});
 const stage=destination=>invoke(['stage-update',artifact,envelopeFile,keyFile,policyFile,destination]);
 const inspect=destination=>invoke(['inspect-update',destination,keyFile,policyFile]);
 const verified=invoke(['verify-update',artifact,envelopeFile,keyFile,policyFile]);assert.equal(verified.status,0,verified.stderr);
 run('sudo',['mount','-t','tmpfs','-o',`size=1m,nr_inodes=128,uid=${uid},gid=${gid},mode=0700,nodev,nosuid,noexec`,'aegis-update-fault',mountPoint]);mounted=true;
 assert.equal(run('findmnt',['--noheadings','--mountpoint',mountPoint,'--output','FSTYPE']).trim(),'tmpfs');
 const capacity=await statfs(mountPoint);assert.ok(capacity.blocks*capacity.bsize<=2*1024*1024);
 const destination=path.join(mountPoint,'stage');
 const full=stage(destination);assert.equal(full.status,1,full.stderr);assert.match(full.stderr,/ENOSPC/);
 await assert.rejects(access(destination),{code:'ENOENT'});assert.equal(inspect(destination).status,1);
 assert.equal(await readFile(policyFile,'utf8'),policyBytes);
 owned();run('sudo',['mount','-o','remount,size=32m',mountPoint]);
 const retried=stage(destination);assert.equal(retried.status,0,retried.stderr);assert.equal(inspect(destination).status,0);
 console.log('PASS: kernel ENOSPC during an 8 MiB staging copy leaves no ready/partial destination; increasing tmpfs capacity permits retry at the same path');
 const state=path.join(mountPoint,'trusted-state');
 const initialized=invoke(['init-update-state',state]);assert.equal(initialized.status,0,initialized.stderr);
 const track=()=>invoke(['verify-tracked-update',artifact,envelopeFile,keyFile,policyFile,state]);
 const firstObservation=track();assert.equal(firstObservation.status,0,firstObservation.stderr);assert.equal(JSON.parse(firstObservation.stdout).retry,false);
 // Fill only the bounded private filesystem, then protect an already-ready stage.
 const filler=path.join(mountPoint,'filler.bin'),file=await open(filler,'wx');let exhausted=false;
 try {
  const block=Buffer.alloc(64*1024);
  for(let written=0;written<33*1024*1024;) {
   try {const result=await file.write(block);assert.ok(result.bytesWritten>0);written+=result.bytesWritten;}
   catch(error) {if(error.code!=='ENOSPC') throw error;exhausted=true;break;}
  }
 } finally {await file.close();}
 assert.equal(exhausted,true,'Private tmpfs must actually reach ENOSPC');
 const second=path.join(mountPoint,'second');const rejected=stage(second);assert.equal(rejected.status,1,rejected.stderr);assert.match(rejected.stderr,/ENOSPC/);
 await assert.rejects(access(second),{code:'ENOENT'});
 const retained=inspect(destination);assert.equal(retained.status,0,retained.stderr);
 await publish({...manifest,version:'3.0.0',sequence:3});
 const failedObservation=track();assert.equal(failedObservation.status,1,failedObservation.stderr);assert.match(failedObservation.stderr,/ENOSPC/);
 const diagnostic=invoke(['inspect-update-state',state]);assert.equal(diagnostic.status,0,diagnostic.stderr);
 const snapshot=JSON.parse(diagnostic.stdout);assert.equal(snapshot.lastVerified.version,'2.0.0');assert.equal(snapshot.recordAuthenticated,false);assert.equal(snapshot.verificationBlockedAtInspection,false);
 await publish(manifest);const oldObservation=track();assert.equal(oldObservation.status,0,oldObservation.stderr);assert.equal(JSON.parse(oldObservation.stdout).retry,true);
 await rm(filler);const recovered=stage(second);assert.equal(recovered.status,0,recovered.stderr);assert.equal(inspect(second).status,0);
 assert.equal(await readFile(policyFile,'utf8'),policyBytes);
 await publish({...manifest,version:'3.0.0',sequence:3});
 const advanced=track();assert.equal(advanced.status,0,advanced.stderr);assert.equal(JSON.parse(advanced.stdout).retry,false);
 const older=invoke(['verify-tracked-update',path.join(destination,'artifact.bin'),path.join(destination,'envelope.json'),keyFile,policyFile,state]);
 assert.equal(older.status,1);assert.match(older.stderr,/rollback/);
 console.log('PASS: ENOSPC preserves the committed update observation; freeing space allows advancement and older staged metadata is rejected');
 console.log('PASS: a full private filesystem preserves an existing verified stage and trusted policy; freeing space permits a second clean retry');
} finally {
 owned();
 const probe=spawnSync('findmnt',['--json','--mountpoint',mountPoint,'--output','TARGET,FSTYPE,SOURCE'],{encoding:'utf8',timeout:10000});
 if(probe.status===0) {
  const mounts=JSON.parse(probe.stdout).filesystems;assert.equal(mounts.length,1);
  assert.equal(mounts[0].target,mountPoint);assert.equal(mounts[0].fstype,'tmpfs');assert.equal(mounts[0].source,'aegis-update-fault');
  run('sudo',['umount','--',mountPoint]);mounted=false;
 } else {assert.equal(probe.status,1,probe.stderr);assert.equal(mounted,false,'Expected private mount disappeared');}
 // A failed ownership check or unmount stops before recursive cleanup.
 await rm(root,{recursive:true,force:true});
}
