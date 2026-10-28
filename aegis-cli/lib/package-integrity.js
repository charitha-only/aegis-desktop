import {lstat,opendir,writeFile} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';

const MANIFEST='aegis-integrity.json';
const MAX_BYTES=1024*1024*1024,MAX_MANIFEST=1024*1024;
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
function safeName(name) {
  const stem=name.split('.')[0];
  return Buffer.byteLength(name)<=255 && !/[<>:"/\\|?*\p{Cc}\p{Cf}]/u.test(name)
    && !/[. ]$/.test(name) && !/^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i.test(stem);
}
async function hashFile(filename,size) {
  let bytes=0;
  const hash=createHash('sha256');
  for await(const chunk of createReadStream(filename)) {
    bytes+=chunk.length;
    if(bytes>size || bytes>MAX_BYTES) throw new Error('Package file size changed or exceeded limit');
    hash.update(chunk);
  }
  if(bytes!==size) throw new Error('Package file size changed');
  return hash.digest('hex');
}
async function inventory(directory) {
  if(typeof directory!=='string' || !directory) throw new Error('A package directory is required');
  const root=path.resolve(directory);
  const metadata=await lstat(root);
  if(!metadata.isDirectory() || metadata.isSymbolicLink()) throw new Error('Package root must be an ordinary directory');
  const files=[],seen=new Set();
  let entries=0,totalBytes=0;
  async function walk(relative,depth) {
    if(depth>32) throw new Error('Package directory depth exceeds limit');
    const names=[];
    for await(const entry of await opendir(path.join(root,relative))) {
      if((relative || entry.name!==MANIFEST) && ++entries>8192) throw new Error('Package entry count exceeds limit');
      names.push(entry.name);
    }
    names.sort();
    for(const name of names) {
      if(!safeName(name)) throw new Error('Unsafe package filename');
      const filePath=relative?`${relative}/${name}`:name;
      if(filePath.length>4096) throw new Error('Package path exceeds limit');
      const key=filePath.normalize('NFC').toLowerCase();
      if(seen.has(key)) throw new Error('Package path collision');
      seen.add(key);
      if(!relative && key===MANIFEST) {
        if(name!==MANIFEST) throw new Error('Reserved manifest filename collision');
        continue;
      }
      const filename=path.join(root,...filePath.split('/'));
      const info=await lstat(filename);
      if(info.isSymbolicLink()) throw new Error('Linked package entry rejected');
      if(info.isDirectory()) await walk(filePath,depth+1);
      else {
        if(!info.isFile()) throw new Error('Non-file package entry rejected');
        totalBytes+=info.size;
        if(files.length>=4096 || !Number.isSafeInteger(info.size) || totalBytes>MAX_BYTES) throw new Error('Package size or file count exceeds limit');
        files.push({path:filePath,size:info.size,sha256:await hashFile(filename,info.size)});
      }
    }
  }
  await walk('',0);
  files.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);
  const bytes=Buffer.from(JSON.stringify({schemaVersion:1,files},null,2)+'\n');
  if(bytes.length>MAX_MANIFEST) throw new Error('Package manifest exceeds limit');
  return {root,bytes,totalBytes,fileCount:files.length};
}
export async function writePackageIntegrity(directory) {
  const result=await inventory(directory);
  const manifestPath=path.join(result.root,MANIFEST);
  await writeFile(manifestPath,result.bytes,{flag:'wx',mode:0o644});
  return {manifestPath,manifestSha256:digest(result.bytes),fileCount:result.fileCount,totalBytes:result.totalBytes};
}
export async function verifyPackageIntegrity(directory,expectedManifestSha256) {
  if(typeof expectedManifestSha256!=='string' || !/^[a-f0-9]{64}$/i.test(expectedManifestSha256)) throw new Error('A separately trusted manifest SHA-256 is required');
  const expected=expectedManifestSha256.toLowerCase();
  if(typeof directory!=='string' || !directory) throw new Error('A package directory is required');
  const root=path.resolve(directory),filename=path.join(root,MANIFEST);
  const rootInfo=await lstat(root);
  if(!rootInfo.isDirectory() || rootInfo.isSymbolicLink()) throw new Error('Package root must be an ordinary directory');
  const info=await lstat(filename);
  if(!info.isFile() || info.isSymbolicLink() || info.size>MAX_MANIFEST) throw new Error('Invalid package manifest file');
  const chunks=[];
  let length=0;
  for await(const chunk of createReadStream(filename)) {
    length+=chunk.length;
    if(length>MAX_MANIFEST) throw new Error('Package manifest exceeds limit');
    chunks.push(chunk);
  }
  const bytes=Buffer.concat(chunks,length);
  if(digest(bytes)!==expected) throw new Error('Package manifest digest mismatch');
  // Never interpret manifest-supplied paths. Rebuild the bounded canonical
  // inventory and compare it to the independently pinned descriptor digest.
  const actual=await inventory(root);
  if(digest(actual.bytes)!==expected) throw new Error('Package inventory mismatch');
  return {valid:true,publisherAuthenticated:false,verificationScope:'fileBytesAgainstPinnedManifest',manifestSha256:expected,fileCount:actual.fileCount,totalBytes:actual.totalBytes};
}
