#!/usr/bin/env node
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,copyFile,chmod,symlink,rm,access,unlink} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import path from 'node:path';
const root=await mkdtemp(path.join(tmpdir(),'aegis asset preflight '));
const executable=process.platform==='win32'?'aegis-core.exe':'aegis-core';
const type=process.platform==='win32'?'junction':'dir';
try {
  const core=path.join(root,executable),ui=path.join(root,'src-ui'),dist=path.join(ui,'dist');
  const outside=path.join(root,'outside-assets');
  await copyFile(path.resolve('src-core/target/release',executable),core);await chmod(core,0o755);
  await mkdir(ui);await mkdir(outside);
  await writeFile(path.join(outside,'index.html'),'TEST_ONLY_EXTERNAL_ASSET');
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({name:'AssetApp',appId:'test.assets',version:'0.1.0'}));
  await symlink(outside,dist,type);
  const reject=async label=>{
    const result=spawnSync(core,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
    assert.ok(Number.isInteger(result.status)&&result.status>0,`Invalid frontend asset must be rejected: ${label}`);
    assert.match(result.stderr,/Cannot validate frontend assets/);
    await assert.rejects(access(path.join(root,'dist-bundle')),{code:'ENOENT'});
    assert.equal(await readFile(path.join(outside,'index.html'),'utf8'),'TEST_ONLY_EXTERNAL_ASSET');
    console.log(`PASS: frontend preflight rejects ${label} before output and preserves source fixtures`);
  };
  await reject('linked dist root');
  await unlink(dist);await mkdir(dist);
  await writeFile(path.join(dist,'index.html'),'TEST_ONLY_VALID_UI');
  const nested=path.join(dist,'nested');
  await symlink(outside,nested,type);await reject('nested directory link');await unlink(nested);
  await rm(ui,{recursive:true});
  const parent=path.join(root,'outside-parent');
  await mkdir(path.join(parent,'dist'),{recursive:true});
  await writeFile(path.join(parent,'dist/index.html'),'TEST_ONLY_PARENT');
  await symlink(parent,ui,type);await reject('linked src-ui parent');await unlink(ui);
  await mkdir(dist,{recursive:true});await writeFile(path.join(dist,'index.html'),'TEST_ONLY_VALID_UI');
  if(process.platform!=='win32') {
    const leaf=path.join(dist,'linked-file');
    await symlink(path.join(outside,'index.html'),leaf);await reject('linked file');await unlink(leaf);
    const fifo=path.join(dist,'pipe');
    const created=spawnSync('mkfifo',[fifo],{encoding:'utf8',timeout:5000});
    assert.equal(created.status,0,created.stderr||created.error?.message);
    await reject('special FIFO');await unlink(fifo);
  }
  const valid=spawnSync(core,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
  assert.equal(valid.status,0,valid.stderr);
  assert.equal(await readFile(path.join(root,'dist-bundle/AssetApp/ui/index.html'),'utf8'),'TEST_ONLY_VALID_UI');
  console.log('PASS: ordinary frontend assets package successfully after link removal');
} finally {
  const relative=path.relative(tmpdir(),root);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
}
