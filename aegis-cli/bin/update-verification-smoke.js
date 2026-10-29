#!/usr/bin/env node
import {mkdtemp,cp,readFile,writeFile,open,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {generateKeyPairSync,createHash,sign} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
const root=await mkdtemp(path.join(tmpdir(),'aegis-update-native-'));
try {
 const artifact=path.join(root,'artifact.bin');
 const binary=path.resolve('src-core/target/release/aegis-core'+(process.platform==='win32'?'.exe':''));
 await cp(binary,artifact);
 const content=await readFile(artifact),keys=generateKeyPairSync('ed25519');
 const policy={appId:'org.aegis.ci.update',platform:{win32:'windows',linux:'linux',darwin:'macos'}[process.platform],architecture:process.arch,currentVersion:'1.0.0',maxSeenSequence:1};
 const manifest={schemaVersion:1,appId:policy.appId,platform:policy.platform,architecture:policy.architecture,version:'2.0.0',sequence:2,expiresAt:Date.now()+3600000,artifact:{size:content.length,sha256:createHash('sha256').update(content).digest('hex')}};
 const payload=Buffer.from(JSON.stringify(manifest));
 const envelope={schemaVersion:1,payload:payload.toString('base64'),signature:sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),keys.privateKey).toString('base64')};
 const envelopeFile=path.join(root,'envelope.json'),keyFile=path.join(root,'public.pem'),policyFile=path.join(root,'policy.json');
 await writeFile(envelopeFile,JSON.stringify(envelope));await writeFile(keyFile,keys.publicKey.export({format:'pem',type:'spki'}));await writeFile(policyFile,JSON.stringify(policy));
 const args=[path.resolve('aegis-cli/bin/aegis.js'),'verify-update',artifact,envelopeFile,keyFile,policyFile];
 const invoke=()=>spawnSync(process.execPath,args,{encoding:'utf8',windowsHide:true,timeout:30000});
 const valid=invoke();assert.equal(valid.status,0,valid.stderr);assert.equal(JSON.parse(valid.stdout).publisherAuthenticated,true);
 const stage=path.join(root,'quarantine');
 const staged=spawnSync(process.execPath,[path.resolve('aegis-cli/bin/aegis.js'),'stage-update',artifact,envelopeFile,keyFile,policyFile,stage],{encoding:'utf8',windowsHide:true,timeout:30000});
 assert.equal(staged.status,0,staged.stderr);assert.equal(JSON.parse(staged.stdout).staged,true);
 const inspect=()=>spawnSync(process.execPath,[path.resolve('aegis-cli/bin/aegis.js'),'inspect-update',stage,keyFile,policyFile],{encoding:'utf8',windowsHide:true,timeout:30000});
 const file=await open(artifact,'r+');try {await file.write(Buffer.from([content[0]^1]),0,1,0);} finally {await file.close();}
 const altered=invoke();assert.equal(altered.status,1);assert.match(altered.stderr,/mismatch/);
 const retained=inspect();assert.equal(retained.status,0,retained.stderr);
 const copied=await open(path.join(stage,'artifact.bin'),'r+');try {await copied.write(Buffer.from([content[0]^1]),0,1,0);} finally {await copied.close();}
 const badStage=inspect();assert.equal(badStage.status,1);assert.match(badStage.stderr,/mismatch/);
 console.log('PASS: staged native artifact survives source changes and inspection rejects staged-byte tampering');
 console.log('PASS: offline update CLI authenticates the actual native core against an ephemeral test key and rejects byte tampering; no installation performed');
} finally {
 const relative=path.relative(tmpdir(),root);
 if(relative&&!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
}
