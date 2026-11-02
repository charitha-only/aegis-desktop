import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const cli=new URL('../bin/aegis.js',import.meta.url);
function inspect(root,name='demo'){return spawnSync(process.execPath,[fileURLToPath(cli),'inspect-bundle',root,name],{encoding:'utf8'});}
test('bundle inspection reports remnants without modifying owner data',async()=>{
 const root=await mkdtemp(path.join(os.tmpdir(),'aegis-inspect-'));
 try{
  await mkdir(path.join(root,'demo'));
  await writeFile(path.join(root,'.aegis-bundle-demo.lock'),'owner');
  await mkdir(path.join(root,'.aegis-stage-demo-'+'a'.repeat(32)));
  await mkdir(path.join(root,'.aegis-stage-other-'+'b'.repeat(32)));
  const result=inspect(root);assert.equal(result.status,0,result.stderr);
  const report=JSON.parse(result.stdout);
  assert.equal(report.payload,'directory');assert.equal(report.lock,'file');
  assert.deepEqual(report.stages,[{name:'.aegis-stage-demo-'+'a'.repeat(32),type:'directory'}]);
  assert.equal(report.snapshotOnly,true);assert.equal(report.publisherAuthenticated,false);
  assert.equal(await readFile(path.join(root,'.aegis-bundle-demo.lock'),'utf8'),'owner');
 }finally{await rm(root,{recursive:true,force:true});}
});
test('bundle inspection leaves missing roots absent and rejects unsafe names',async()=>{
 const root=await mkdtemp(path.join(os.tmpdir(),'aegis-inspect-'));
 try{
  const missing=path.join(root,'missing'),result=inspect(missing);
  assert.equal(result.status,0,result.stderr);assert.equal(JSON.parse(result.stdout).root,'absent');
  const {lstat}=await import('node:fs/promises');
  await assert.rejects(lstat(missing),{code:'ENOENT'});
  for(const name of ['../escape','.','a..b','a/b'])assert.notEqual(inspect(root,name).status,0);
 }finally{await rm(root,{recursive:true,force:true});}
});
test('bundle inspection rejects linked roots and bounds all scanned entries',async()=>{
 const root=await mkdtemp(path.join(os.tmpdir(),'aegis-inspect-'));
 try{
  const {symlink}=await import('node:fs/promises');
  const link=path.join(root,'link'),target=path.join(root,'target');await mkdir(target);
  await symlink(target,link,process.platform==='win32'?'junction':'dir');
  assert.notEqual(inspect(link).status,0);
  for(let start=0;start<4097;start+=128)await Promise.all(Array.from({length:Math.min(128,4097-start)},(_,i)=>writeFile(path.join(target,'entry-'+(start+i)),'')));
  const result=inspect(target);assert.notEqual(result.status,0);assert.match(result.stderr,/4096/);assert.equal(result.stdout,'');
 }finally{await rm(root,{recursive:true,force:true});}
});
