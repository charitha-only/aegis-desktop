import {mkdir,lstat,open,rename,rm} from 'node:fs/promises';
import path from 'node:path';
import {createReadStream} from 'node:fs';
import {verifyUpdate} from './update-verification.js';
function root(options) {if(typeof options.stateDirectory!=='string'||!options.stateDirectory) throw new Error('Trusted state directory required');return path.resolve(options.stateDirectory);}
function record(v) {return {appId:v.appId,platform:v.platform,architecture:v.architecture,version:v.version,sequence:v.nextSequence,expiresAt:v.expiresAt,sha256:v.sha256,size:v.size};}
function fields(value,names) {return value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===names.length&&names.every(name=>Object.hasOwn(value,name));}
function validVersion(value) {return typeof value==='string'&&/^(0|[1-9][0-9]{0,9})\.(0|[1-9][0-9]{0,9})\.(0|[1-9][0-9]{0,9})$/.test(value)&&value.split('.').every(n=>Number(n)<=4294967295);}
async function readState(directory) {
 const folder=await lstat(directory);if(!folder.isDirectory()||folder.isSymbolicLink()) throw new Error('Trusted state root must be an ordinary directory');
 const filename=path.join(directory,'state.json'),info=await lstat(filename);
 if(!info.isFile()||info.isSymbolicLink()||info.size>8192) throw new Error('Invalid or oversized update state');
 let size=0;const chunks=[];
 for await(const chunk of createReadStream(filename)) {size+=chunk.length;if(size>8192) throw new Error('Update state exceeds limit');chunks.push(chunk);}
 if(size!==info.size) throw new Error('Update state changed while reading');
 const bytes=Buffer.concat(chunks),state=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));
 if(!bytes.equals(Buffer.from(JSON.stringify(state)+'\n'))) throw new Error('Noncanonical update state');
 if(!fields(state,['schemaVersion','lastVerified'])||state.schemaVersion!==1) throw new Error('Invalid update state schema');
 const v=state.lastVerified;
 if(v!==null&&(!fields(v,['appId','platform','architecture','version','sequence','expiresAt','sha256','size'])
  ||typeof v.appId!=='string'||!/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(v.appId)||v.appId.endsWith('.')||/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\.|$)/i.test(v.appId)
  ||!['windows','linux','macos'].includes(v.platform)||!['x64','arm64'].includes(v.architecture)||!validVersion(v.version)
  ||!Number.isSafeInteger(v.sequence)||v.sequence<1||!Number.isSafeInteger(v.expiresAt)||v.expiresAt<1
  ||typeof v.sha256!=='string'||!/^[a-f0-9]{64}$/.test(v.sha256)||!Number.isSafeInteger(v.size)||v.size<1||v.size>1024*1024*1024)) throw new Error('Invalid recorded update release');
 return state;
}
async function writeState(directory,state) {
 const pending=path.join(directory,'state.pending');let owned=false;
 try {
  const file=await open(pending,'wx',0o600);owned=true;
  try {await file.writeFile(JSON.stringify(state)+'\n');await file.sync();} finally {await file.close();}
  await rename(pending,path.join(directory,'state.json'));owned=false;
 } finally {if(owned) await rm(pending);}
}
export async function initializeUpdateState(options) {
 const directory=root(options);await mkdir(directory,{mode:0o700});
 await writeState(directory,{schemaVersion:1,lastVerified:null});
 return {initialized:true,stateDirectory:directory,installed:false,autoApply:false};
}
export async function verifyTrackedUpdate(options) {
 const directory=root(options),folder=await lstat(directory);
 if(!folder.isDirectory()||folder.isSymbolicLink()) throw new Error('Trusted state root must be an ordinary directory');
 const lockPath=path.join(directory,'.lock'),lock=await open(lockPath,'wx',0o600);
 try {
  try {await lstat(path.join(directory,'state.pending'));throw new Error('Pending update state requires operator recovery');}
  catch(error) {if(error.code!=='ENOENT') throw error;}
  const state=await readState(directory),verified=await verifyUpdate(options),next=record(verified);
  const retry=state.lastVerified!==null&&JSON.stringify(state.lastVerified)===JSON.stringify(next);
  if(state.lastVerified&&!retry) {
   const old=state.lastVerified;
   if(old.appId!==next.appId||old.platform!==next.platform||old.architecture!==next.architecture||next.sequence<=old.sequence) throw new Error('Tracked update identity or rollback rejected');
   const before=old.version.split('.').map(Number),after=next.version.split('.').map(Number),i=after.findIndex((n,i)=>n!==before[i]);
   if(i<0||after[i]<before[i]) throw new Error('Tracked update version must advance');
  }
  if(!retry) await writeState(directory,{schemaVersion:1,lastVerified:next});
  return {...verified,tracked:true,retry,installed:false,stateDirectory:directory};
 } finally {await lock.close();await rm(lockPath);}
}
