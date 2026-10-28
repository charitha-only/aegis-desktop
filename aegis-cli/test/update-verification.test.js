import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {generateKeyPairSync,sign} from 'node:crypto';
import {verifyUpdate} from '../lib/update-verification.js';
const digest='ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad';
async function fixture(run) {
 const root=await mkdtemp(path.join(tmpdir(),'aegis-update-test-'));
 const keys=generateKeyPairSync('ed25519'),now=Date.now();
 const policy={appId:'org.example.app',platform:'windows',architecture:'x64',currentVersion:'1.0.0',maxSeenSequence:7};
 const manifest={schemaVersion:1,appId:policy.appId,platform:policy.platform,architecture:policy.architecture,version:'2.0.0',sequence:8,expiresAt:now+3600000,artifact:{size:3,sha256:digest}};
 const files={artifactFile:path.join(root,'app.bin'),envelopeFile:path.join(root,'envelope.json'),publicKeyFile:path.join(root,'publisher.pem'),policyFile:path.join(root,'policy.json'),now};
 const publish=async(value=manifest,key=keys.privateKey)=>{
  const payload=Buffer.from(JSON.stringify(value));
  const signature=sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),key);
  await writeFile(files.envelopeFile,JSON.stringify({schemaVersion:1,payload:payload.toString('base64'),signature:signature.toString('base64')}));
 };
 try {
  await writeFile(files.artifactFile,'abc');await writeFile(files.publicKeyFile,keys.publicKey.export({format:'pem',type:'spki'}));
  await writeFile(files.policyFile,JSON.stringify(policy));await publish();
  await run({files,manifest,policy,publish,root,keys});
 } finally {await rm(root,{recursive:true,force:true});}
}
test('signed update verification authenticates a newer app-specific artifact against a separately trusted key',()=>fixture(async({files})=>{
 const result=await verifyUpdate(files);
 assert.equal(result.valid,true);assert.equal(result.publisherAuthenticated,true);
 assert.equal(result.sha256,digest);assert.equal(result.size,3);assert.equal(result.nextSequence,8);assert.equal(result.autoApply,false);
}));
test('wrong publishers, altered signed metadata and altered artifact bytes are rejected',()=>fixture(async({files,manifest,publish})=>{
 const stranger=generateKeyPairSync('ed25519');await publish(manifest,stranger.privateKey);
 await assert.rejects(verifyUpdate(files),/signature/);
 await publish();const envelope=JSON.parse(await (await import('node:fs/promises')).readFile(files.envelopeFile,'utf8'));
 envelope.payload=Buffer.from(JSON.stringify({...manifest,version:'9.0.0'})).toString('base64');
 await writeFile(files.envelopeFile,JSON.stringify(envelope));await assert.rejects(verifyUpdate(files),/signature/);
 await publish();await writeFile(files.artifactFile,'abd');await assert.rejects(verifyUpdate(files),/mismatch/);
}));
test('trusted policy rejects wrong app/OS/CPU, expired metadata, old sequences and non-newer versions',()=>fixture(async({files,manifest,publish})=>{
 for(const change of [{appId:'org.other.app'},{platform:'linux'},{architecture:'arm64'},{expiresAt:files.now},{sequence:7},{sequence:6},{version:'1.0.0'},{version:'0.9.9'},{version:'2.bad.0'},{version:'2.0.0.1'},{version:'02.0.0'}]) {
  await publish({...manifest,...change});await assert.rejects(verifyUpdate(files));
 }
}));
test('schema, encoding and key types fail closed instead of accepting ambiguous signed inputs',()=>fixture(async({files,manifest,publish,keys})=>{
 for(const change of [{schemaVersion:2},{extra:'unrecognized'},{artifact:{...manifest.artifact,size:2**31}},{artifact:{...manifest.artifact,sha256:digest.toUpperCase()}}]) {
  await publish({...manifest,...change});await assert.rejects(verifyUpdate(files));
 }
 await publish();const {readFile}=await import('node:fs/promises');
 const envelope=JSON.parse(await readFile(files.envelopeFile,'utf8'));
 await writeFile(files.envelopeFile,JSON.stringify({...envelope,signature:envelope.signature+'?'}));
 await assert.rejects(verifyUpdate(files));
 await publish();await writeFile(files.publicKeyFile,keys.privateKey.export({format:'pem',type:'pkcs8'}));
 await assert.rejects(verifyUpdate(files));
}));
test('oversized verification metadata is rejected before parsing',()=>fixture(async({files})=>{
 const {readFile}=await import('node:fs/promises');
 const envelope=await readFile(files.envelopeFile,'utf8');
 await writeFile(files.envelopeFile,envelope+' '.repeat(128*1024));
 await assert.rejects(verifyUpdate(files),/limit/);
}));
import {spawnSync} from 'node:child_process';
test('verification CLI authenticates valid input and rejects tampering without installing or changing policy',()=>fixture(async({files})=>{
 const {readFile}=await import('node:fs/promises');const before=await readFile(files.policyFile);
 const args=[path.resolve('aegis-cli/bin/aegis.js'),'verify-update',files.artifactFile,files.envelopeFile,files.publicKeyFile,files.policyFile];
 const valid=spawnSync(process.execPath,args,{encoding:'utf8',timeout:10000});
 assert.equal(valid.status,0,valid.stderr);assert.equal(JSON.parse(valid.stdout).publisherAuthenticated,true);
 await writeFile(files.artifactFile,'abd');const altered=spawnSync(process.execPath,args,{encoding:'utf8',timeout:10000});
 assert.equal(altered.status,1);assert.match(altered.stderr,/mismatch/);
 assert.deepEqual(await readFile(files.policyFile),before);
}));
test('signed payload bytes must match the documented JSON form, including rejection of a UTF-8 BOM',()=>fixture(async({files,manifest,keys})=>{
 const payload=Buffer.concat([Buffer.from([0xef,0xbb,0xbf]),Buffer.from(JSON.stringify(manifest))]);
 const signature=sign(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),keys.privateKey);
 await writeFile(files.envelopeFile,JSON.stringify({schemaVersion:1,payload:payload.toString('base64'),signature:signature.toString('base64')}));
 await assert.rejects(verifyUpdate(files),/noncanonical/);
}));
