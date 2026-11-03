import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,cp,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
test('package refuses occupied build locks before prerequisite checks and preserves owner bytes',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'aegis-package-preflight-'));
 try{
  await cp(fileURLToPath(new URL('..',import.meta.url)),path.join(root,'aegis-cli'),{recursive:true});
  await mkdir(path.join(root,'src-ui','src'),{recursive:true});
  await cp(fileURLToPath(new URL('../../src-ui/src/bridge-origin.mjs',import.meta.url)),path.join(root,'src-ui','src','bridge-origin.mjs'));
  await writeFile(path.join(root,'package.json'),JSON.stringify({type:'module'}));
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({name:'demo'}));
  await mkdir(path.join(root,'dist-bundle'));
  const lock=path.join(root,'dist-bundle','.aegis-bundle-demo.lock');
  await writeFile(lock,'TEST_ONLY_OWNER');
  const result=spawnSync(process.execPath,[path.join(root,'aegis-cli','bin','aegis.js'),'package'],{encoding:'utf8',windowsHide:true,timeout:15000});
  assert.notEqual(result.status,0);assert.match(result.stderr,/Bundle build lock already exists/);
  assert.equal(await readFile(lock,'utf8'),'TEST_ONLY_OWNER');
 }finally{await rm(root,{recursive:true,force:true});}
});
test('package rejects a linked output parent before tools and preserves the external owner',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'aegis-package-linked-'));
 try{
  await cp(fileURLToPath(new URL('..',import.meta.url)),path.join(root,'aegis-cli'),{recursive:true});
  await mkdir(path.join(root,'src-ui','src'),{recursive:true});
  await cp(fileURLToPath(new URL('../../src-ui/src/bridge-origin.mjs',import.meta.url)),path.join(root,'src-ui','src','bridge-origin.mjs'));
  await writeFile(path.join(root,'package.json'),JSON.stringify({type:'module'}));
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({name:'demo'}));
  const target=path.join(root,'external');await mkdir(target);await writeFile(path.join(target,'owner'),'TEST_ONLY_EXTERNAL');
  const {symlink,readdir}=await import('node:fs/promises');
  await symlink(target,path.join(root,'dist-bundle'),process.platform==='win32'?'junction':'dir');
  const result=spawnSync(process.execPath,[path.join(root,'aegis-cli','bin','aegis.js'),'package'],{encoding:'utf8',windowsHide:true,timeout:15000});
  assert.notEqual(result.status,0);assert.match(result.stderr,/Bundle output parent must be an ordinary directory/);
  assert.deepEqual(await readdir(target),['owner']);assert.equal(await readFile(path.join(target,'owner'),'utf8'),'TEST_ONLY_EXTERNAL');
 }finally{await rm(root,{recursive:true,force:true});}
});
