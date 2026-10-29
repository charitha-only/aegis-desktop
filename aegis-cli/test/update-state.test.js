import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {generateKeyPairSync,sign} from 'node:crypto';
import {initializeUpdateState,verifyTrackedUpdate} from '../lib/update-state.js';
async function fixture(run) {
 const root=await mkdtemp(path.join(tmpdir(),'aegis-tracked-test-')),keys=generateKeyPairSync('ed25519'),now=Date.now();
 const options={artifactFile:path.join(root,'artifact.bin'),envelopeFile:path.join(root,'envelope.json'),publicKeyFile:path.join(root,'public.pem'),policyFile:path.join(root,'policy.json'),stateDirectory:path.join(root,'state'),now};
 const manifest={schemaVersion:1,appId:'org.example.app',platform:'windows',architecture:'x64',version:'2.0.0',sequence:2,expiresAt:now+3600000,artifact:{size:3,sha256:'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'}};
 async function publish(change={}) {
  const payload=Buffer.from(JSON.stringify({...manifest,...change}));
  await writeFile(options.envelopeFile,JSON.stringify({schemaVersion:1,payload:payload.toString('base64'),signature:sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),keys.privateKey).toString('base64')}));
 }
 try {
  await writeFile(options.artifactFile,'abc');await writeFile(options.publicKeyFile,keys.publicKey.export({format:'pem',type:'spki'}));
  await writeFile(options.policyFile,JSON.stringify({appId:manifest.appId,platform:manifest.platform,architecture:manifest.architecture,currentVersion:'1.0.0',maxSeenSequence:1}));await publish();
  await run({options,publish,manifest});
 } finally {await rm(root,{recursive:true,force:true});}
}
test('tracked verification persists an observed release and permits the same release to retry after reopening',()=>fixture(async({options})=>{
 await initializeUpdateState(options);
 const first=await verifyTrackedUpdate(options);assert.equal(first.tracked,true);assert.equal(first.retry,false);assert.equal(first.installed,false);
 const reopened=await verifyTrackedUpdate({...options});assert.equal(reopened.retry,true);assert.equal(reopened.version,'2.0.0');assert.equal(reopened.autoApply,false);
}));
test('observed sequence and version floors reject rollback and changed same-sequence metadata',()=>fixture(async({options,publish})=>{
 await initializeUpdateState(options);await verifyTrackedUpdate(options);
 for(const change of [{version:'1.5.0',sequence:1},{version:'1.5.0',sequence:3},{version:'2.0.0',sequence:3},{version:'2.1.0',sequence:2},{expiresAt:options.now+7200000}]) {
  await publish(change);await assert.rejects(verifyTrackedUpdate(options));
 }
 await publish({version:'3.0.0',sequence:3});assert.equal((await verifyTrackedUpdate(options)).retry,false);
 await publish();await assert.rejects(verifyTrackedUpdate(options));
}));
test('missing, corrupt, oversized or unknown-schema state fails closed and cannot be silently reinitialized',()=>fixture(async({options})=>{
 await assert.rejects(verifyTrackedUpdate(options));await initializeUpdateState(options);
 await assert.rejects(initializeUpdateState(options));
 const filename=path.join(options.stateDirectory,'state.json');
 for(const content of ['{bad',JSON.stringify({schemaVersion:9,lastVerified:null})+'\n',JSON.stringify({schemaVersion:1,lastVerified:null})+' '.repeat(8192)]) {
  await writeFile(filename,content);await assert.rejects(verifyTrackedUpdate(options));assert.equal(await readFile(filename,'utf8'),content);
 }
 await rm(filename);await assert.rejects(verifyTrackedUpdate(options));await assert.rejects(initializeUpdateState(options));
}));
test('an occupied state lock fails closed without modifying the observed release',()=>fixture(async({options})=>{
 await initializeUpdateState(options);await verifyTrackedUpdate(options);
 const lock=path.join(options.stateDirectory,'.lock');await writeFile(lock,'operator-owned');
 await assert.rejects(verifyTrackedUpdate(options));assert.equal(await readFile(lock,'utf8'),'operator-owned');
 await rm(lock);assert.equal((await verifyTrackedUpdate(options)).retry,true);
}));
test('concurrent observers commit one new observation and never discard a pending state file',()=>fixture(async({options})=>{
 await initializeUpdateState(options);
 const attempts=await Promise.allSettled([verifyTrackedUpdate(options),verifyTrackedUpdate(options)]);
 const successful=attempts.filter(x=>x.status==='fulfilled').map(x=>x.value);
 assert.ok(successful.length>=1);assert.equal(successful.filter(x=>!x.retry).length,1);
 assert.equal((await verifyTrackedUpdate(options)).retry,true);
 const pending=path.join(options.stateDirectory,'state.pending');await writeFile(pending,'interrupted');
 await assert.rejects(verifyTrackedUpdate(options),/Pending/);assert.equal(await readFile(pending,'utf8'),'interrupted');
}));
import {spawnSync} from 'node:child_process';
test('tracked CLI retries survive a new process and initialization cannot overwrite state',()=>fixture(async({options})=>{
 const cli=path.resolve('aegis-cli/bin/aegis.js'),invoke=args=>spawnSync(process.execPath,[cli,...args],{encoding:'utf8',timeout:10000});
 const init=invoke(['init-update-state',options.stateDirectory]);assert.equal(init.status,0,init.stderr);
 const args=['verify-tracked-update',options.artifactFile,options.envelopeFile,options.publicKeyFile,options.policyFile,options.stateDirectory];
 const first=invoke(args);assert.equal(first.status,0,first.stderr);assert.equal(JSON.parse(first.stdout).retry,false);
 const next=invoke(args);assert.equal(next.status,0,next.stderr);assert.equal(JSON.parse(next.stdout).retry,true);
 assert.equal(invoke(['init-update-state',options.stateDirectory]).status,1);
}));

test('a state directory stays bound to its original app and target even if external policy changes',()=>fixture(async({options,publish})=>{
 await initializeUpdateState(options);await verifyTrackedUpdate(options);
 const policy=JSON.parse(await readFile(options.policyFile,'utf8'));policy.appId='org.other.app';await writeFile(options.policyFile,JSON.stringify(policy));
 await publish({appId:policy.appId,version:'3.0.0',sequence:3});await assert.rejects(verifyTrackedUpdate(options),/identity/);
}));
