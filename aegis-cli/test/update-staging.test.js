import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {generateKeyPairSync,sign} from 'node:crypto';
import {stageUpdate,inspectStagedUpdate} from '../lib/update-staging.js';
async function fixture(run) {
 const root=await mkdtemp(path.join(tmpdir(),'aegis-staging-test-')),now=Date.now();
 const files={artifactFile:path.join(root,'download.bin'),envelopeFile:path.join(root,'download.json'),publicKeyFile:path.join(root,'public.pem'),policyFile:path.join(root,'policy.json'),stageDirectory:path.join(root,'stage'),now};
 const keys=generateKeyPairSync('ed25519');
 const manifest={schemaVersion:1,appId:'org.example.app',platform:'windows',architecture:'x64',version:'2.0.0',sequence:2,expiresAt:now+3600000,artifact:{size:3,sha256:'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'}};
 const payload=Buffer.from(JSON.stringify(manifest));
 try {
  await writeFile(files.artifactFile,'abc');await writeFile(files.envelopeFile,JSON.stringify({schemaVersion:1,payload:payload.toString('base64'),signature:sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),keys.privateKey).toString('base64')}));
  await writeFile(files.publicKeyFile,keys.publicKey.export({format:'pem',type:'spki'}));
  await writeFile(files.policyFile,JSON.stringify({appId:manifest.appId,platform:manifest.platform,architecture:manifest.architecture,currentVersion:'1.0.0',maxSeenSequence:1}));
  await run({files,root});
 } finally {await rm(root,{recursive:true,force:true});}
}
test('a staged update remains independently verifiable after the original download changes',()=>fixture(async({files})=>{
 const staged=await stageUpdate(files);assert.equal(staged.staged,true);assert.equal(staged.autoApply,false);
 await writeFile(files.artifactFile,'BAD');await writeFile(files.envelopeFile,'BAD');
 const checked=await inspectStagedUpdate(files);assert.equal(checked.publisherAuthenticated,true);assert.equal(checked.version,'2.0.0');
 assert.equal(await readFile(checked.artifactFile,'utf8'),'abc');
}));
test('incomplete or changed staged data is rejected while existing stages are never overwritten',()=>fixture(async({files})=>{
 const staged=await stageUpdate(files);
 await assert.rejects(stageUpdate(files));assert.equal(await readFile(staged.artifactFile,'utf8'),'abc');
 const ready=path.join(files.stageDirectory,'ready.json');const original=await readFile(ready);
 await rm(ready);await assert.rejects(inspectStagedUpdate(files));
 await writeFile(ready,original);await writeFile(staged.artifactFile,'BAD');await assert.rejects(inspectStagedUpdate(files));
}));
test('concurrent staging publishes one complete stage without changing trusted policy',()=>fixture(async({files})=>{
 const before=await readFile(files.policyFile);
 const attempts=await Promise.allSettled([stageUpdate(files),stageUpdate(files)]);
 assert.equal(attempts.filter(result=>result.status==='fulfilled').length,1);
 assert.equal(attempts.filter(result=>result.status==='rejected').length,1);
 assert.equal((await inspectStagedUpdate(files)).staged,true);
 assert.deepEqual(await readFile(files.policyFile),before);
}));
test('rejected downloads leave no stage and repaired downloads can retry at the same destination',()=>fixture(async({files})=>{
 await writeFile(files.artifactFile,'BAD');await assert.rejects(stageUpdate(files));
 await assert.rejects((await import('node:fs/promises')).access(files.stageDirectory),{code:'ENOENT'});
 await writeFile(files.artifactFile,'abc');assert.equal((await stageUpdate(files)).staged,true);
}));
test('inspection rejects changed markers, extra files, expired metadata and newly disallowed sequences',()=>fixture(async({files})=>{
 await stageUpdate(files);const marker=path.join(files.stageDirectory,'ready.json'),original=await readFile(marker);
 await writeFile(marker,'{}');await assert.rejects(inspectStagedUpdate(files),/marker/);
 await writeFile(marker,original);const extra=path.join(files.stageDirectory,'extra');await writeFile(extra,'unexpected');
 await assert.rejects(inspectStagedUpdate(files),/entry/);await rm(extra);
 await assert.rejects(inspectStagedUpdate({...files,now:files.now+3600000}));
 const policy=JSON.parse(await readFile(files.policyFile,'utf8'));policy.maxSeenSequence=2;await writeFile(files.policyFile,JSON.stringify(policy));
 await assert.rejects(inspectStagedUpdate(files));
}));
import {spawnSync} from 'node:child_process';
test('staging and inspection CLI commands preserve a verified copy and refuse replacement',()=>fixture(async({files})=>{
 const cli=path.resolve('aegis-cli/bin/aegis.js');
 const staged=spawnSync(process.execPath,[cli,'stage-update',files.artifactFile,files.envelopeFile,files.publicKeyFile,files.policyFile,files.stageDirectory],{encoding:'utf8',timeout:10000});
 assert.equal(staged.status,0,staged.stderr);assert.equal(JSON.parse(staged.stdout).staged,true);
 const checked=spawnSync(process.execPath,[cli,'inspect-update',files.stageDirectory,files.publicKeyFile,files.policyFile],{encoding:'utf8',timeout:10000});
 assert.equal(checked.status,0,checked.stderr);assert.equal(JSON.parse(checked.stdout).autoApply,false);
 const again=spawnSync(process.execPath,[cli,'stage-update',files.artifactFile,files.envelopeFile,files.publicKeyFile,files.policyFile,files.stageDirectory],{encoding:'utf8',timeout:10000});
 assert.equal(again.status,1);assert.equal(await readFile(path.join(files.stageDirectory,'artifact.bin'),'utf8'),'abc');
}));
