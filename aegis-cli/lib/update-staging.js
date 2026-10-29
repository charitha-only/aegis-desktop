import {mkdir,lstat,open,rm,opendir,rename} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import path from 'node:path';
import {verifyUpdate} from './update-verification.js';
function location(value) {
 if(typeof value!=='string'||!value||value.length>4096) throw new Error('A staging directory is required');
 return path.resolve(value);
}
async function copyBounded(source,destination,limit) {
 const info=await lstat(source);
 if(!info.isFile()||info.isSymbolicLink()||info.size>limit) throw new Error('Invalid or oversized staging source');
 const file=await open(destination,'wx',0o600);let size=0;
 try {
  for await(const chunk of createReadStream(source)) {
   size+=chunk.length;if(size>limit) throw new Error('Staging copy exceeds signed size limit');
   let offset=0;
   while(offset<chunk.length) {
    const {bytesWritten}=await file.write(chunk,offset,chunk.length-offset);
    if(bytesWritten===0) throw new Error('Staging copy made no progress');offset+=bytesWritten;
   }
  }
  if(size!==info.size) throw new Error('Staging source changed while reading');
  await file.sync();
 } finally {await file.close();}
}
function inputs(options,root) {return {...options,artifactFile:path.join(root,'artifact.bin'),envelopeFile:path.join(root,'envelope.json')};}
function result(verified,root) {return {...verified,staged:true,stageDirectory:root,artifactFile:path.join(root,'artifact.bin'),envelopeFile:path.join(root,'envelope.json')};}
/** New directory only. Ready marker is written after the copied bytes reverify. */
export async function stageUpdate(options) {
 const root=location(options.stageDirectory),verified=await verifyUpdate(options);let owned=false;
 try {
  await mkdir(root,{mode:0o700});owned=true;
  await copyBounded(options.artifactFile,path.join(root,'artifact.bin'),verified.size);
  await copyBounded(options.envelopeFile,path.join(root,'envelope.json'),128*1024);
  const copied=await verifyUpdate(inputs(options,root));
  if(JSON.stringify(copied)!==JSON.stringify(verified)) throw new Error('Update changed during staging');
  const pending=path.join(root,'ready.pending');
  const ready=await open(pending,'wx',0o600);
  try {await ready.writeFile(JSON.stringify(copied)+'\n');await ready.sync();} finally {await ready.close();}
  await rename(pending,path.join(root,'ready.json'));
  return result(copied,root);
 } catch(error) {
  if(owned) {
   const relative=path.relative(path.dirname(root),root);
   if(relative&&relative!=='..'&&!relative.startsWith('..'+path.sep)&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
  }
  throw error;
 }
}
export async function inspectStagedUpdate(options) {
 const root=location(options.stageDirectory);
 const info=await lstat(root);
 if(!info.isDirectory()||info.isSymbolicLink()) throw new Error('Stage must be an ordinary directory');
 const expected=new Set(['artifact.bin','envelope.json','ready.json']);let count=0;
 for await(const entry of await opendir(root)) {
  if(++count>3||!expected.has(entry.name)) throw new Error('Unexpected staging entry');
 }
 if(count!==3) throw new Error('Incomplete update stage');
 const readyFile=path.join(root,'ready.json'),readyInfo=await lstat(readyFile);
 if(!readyInfo.isFile()||readyInfo.isSymbolicLink()||readyInfo.size>8192) throw new Error('Invalid stage readiness marker');
 const chunks=[];let size=0;
 for await(const chunk of createReadStream(readyFile)) {
  size+=chunk.length;if(size>8192) throw new Error('Stage readiness marker exceeds limit');chunks.push(chunk);
 }
 if(size!==readyInfo.size) throw new Error('Readiness marker changed while reading');
 const verified=await verifyUpdate(inputs(options,root));
 if(!Buffer.concat(chunks).equals(Buffer.from(JSON.stringify(verified)+'\n'))) throw new Error('Stage readiness marker mismatch');
 return result(verified,root);
}
