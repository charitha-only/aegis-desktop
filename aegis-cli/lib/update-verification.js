import {lstat} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import {createHash,createPublicKey,verify} from 'node:crypto';
const MAX_ARTIFACT=1024*1024*1024;
const text=bytes=>new TextDecoder('utf-8',{fatal:true}).decode(bytes);
function fields(value,names) {
 return value!==null&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===names.length&&names.every(name=>Object.hasOwn(value,name));
}
async function ordinary(filename,limit) {
 const info=await lstat(filename);
 if(!info.isFile()||info.isSymbolicLink()) throw new Error('Update input must be an ordinary file');
 if(info.size>limit) throw new Error('Update input exceeds limit');
 return info;
}
async function bounded(filename,limit) {
 const info=await ordinary(filename,limit);let count=0;const chunks=[];
 for await(const chunk of createReadStream(filename)) {
  count+=chunk.length;if(count>limit) throw new Error('Update input exceeds limit');chunks.push(chunk);
 }
 if(count!==info.size) throw new Error('Update input changed while reading');
 return Buffer.concat(chunks);
}
function base64(value,limit) {
 if(typeof value!=='string'||value.length>Math.ceil(limit/3)*4) throw new Error('Update encoding exceeds limit');
 const result=Buffer.from(value,'base64');
 if(result.length>limit||result.toString('base64')!==value) throw new Error('Invalid update base64 encoding');
 return result;
}
function version(value) {
 if(typeof value!=='string'||!/^(0|[1-9][0-9]{0,9})\.(0|[1-9][0-9]{0,9})\.(0|[1-9][0-9]{0,9})$/.test(value)) throw new Error('Invalid update version');
 const parts=value.split('.').map(Number);
 if(parts.some(n=>n>4294967295)) throw new Error('Invalid update version');
 return parts;
}
/** Offline, point-in-time verification only. Trust inputs must be provisioned separately. */
export async function verifyUpdate({artifactFile,envelopeFile,publicKeyFile,policyFile,now=Date.now()}) {
 const policy=JSON.parse(text(await bounded(policyFile,8192)));
 if(!fields(policy,['appId','platform','architecture','currentVersion','maxSeenSequence'])
   ||typeof policy.appId!=='string'||!/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(policy.appId)
   ||policy.appId.endsWith('.')||/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)/i.test(policy.appId)
   ||!['windows','linux','macos'].includes(policy.platform)||!['x64','arm64'].includes(policy.architecture)
   ||!Number.isSafeInteger(policy.maxSeenSequence)||policy.maxSeenSequence<0
   ||!Number.isSafeInteger(now)||now<0) throw new Error('Invalid trusted update policy');
 const current=version(policy.currentVersion);
 const envelope=JSON.parse(text(await bounded(envelopeFile,128*1024)));
 if(!fields(envelope,['schemaVersion','payload','signature'])||envelope.schemaVersion!==1) throw new Error('Invalid update envelope schema');
 const payload=base64(envelope.payload,64*1024),signature=base64(envelope.signature,64);
 if(signature.length!==64) throw new Error('Invalid update signature length');
 const pem=text(await bounded(publicKeyFile,8192));
 if(!pem.startsWith('-----BEGIN PUBLIC KEY-----')) throw new Error('A separately trusted public key is required');
 const publicKey=createPublicKey(pem);
 if(publicKey.asymmetricKeyType!=='ed25519'||!verify(null,Buffer.concat([Buffer.from('AEGIS_UPDATE_V1\0'),payload]),publicKey,signature)) throw new Error('Update signature rejected');
 const manifest=JSON.parse(text(payload));
 if(!fields(manifest,['schemaVersion','appId','platform','architecture','version','sequence','expiresAt','artifact'])||manifest.schemaVersion!==1
   ||!fields(manifest.artifact,['size','sha256'])||!Number.isSafeInteger(manifest.artifact.size)||manifest.artifact.size<1||manifest.artifact.size>MAX_ARTIFACT
   ||typeof manifest.artifact.sha256!=='string'||!/^[a-f0-9]{64}$/.test(manifest.artifact.sha256)
   ||!Buffer.from(JSON.stringify(manifest)).equals(payload)) throw new Error('Invalid or noncanonical signed update metadata');
 if(manifest.appId!==policy.appId||manifest.platform!==policy.platform||manifest.architecture!==policy.architecture
   ||!Number.isSafeInteger(manifest.expiresAt)||manifest.expiresAt<=now
   ||!Number.isSafeInteger(manifest.sequence)||manifest.sequence<=policy.maxSeenSequence) throw new Error('Update identity, expiry or rollback policy rejected');
 const next=version(manifest.version),differing=next.findIndex((n,i)=>n!==current[i]);
 if(differing<0||next[differing]<current[differing]) throw new Error('Update version must be newer');
 const info=await ordinary(artifactFile,manifest.artifact.size);
 if(info.size!==manifest.artifact.size) throw new Error('Update artifact size mismatch');
 const hash=createHash('sha256');let size=0;
 for await(const chunk of createReadStream(artifactFile)) {
  size+=chunk.length;if(size>manifest.artifact.size) throw new Error('Update artifact exceeds signed size limit');hash.update(chunk);
 }
 const sha256=hash.digest('hex');
 if(size!==manifest.artifact.size||sha256!==manifest.artifact.sha256) throw new Error('Update artifact mismatch');
 return {valid:true,publisherAuthenticated:true,signatureAlgorithm:'Ed25519',appId:manifest.appId,platform:manifest.platform,architecture:manifest.architecture,version:manifest.version,sha256,size,nextSequence:manifest.sequence,expiresAt:manifest.expiresAt,autoApply:false};
}
