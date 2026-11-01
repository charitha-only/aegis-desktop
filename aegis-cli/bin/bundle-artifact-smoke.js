#!/usr/bin/env node
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,copyFile,chmod,rm} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import path from 'node:path';

const root=await mkdtemp(path.join(tmpdir(),'aegis actual bundle artifact '));
const name=process.platform==='win32'?'aegis-core.exe':'aegis-core';
const source=path.resolve('src-core/target/release',name);
const hash=data=>createHash('sha256').update(data).digest('hex');
try {
  const selected=path.join(root,'src-core/target/selected-host/release',name);
  const stale=path.join(root,'src-core/target/release',name);
  await mkdir(path.dirname(selected),{recursive:true});
  await mkdir(path.dirname(stale),{recursive:true});
  await mkdir(path.join(root,'src-ui/dist'),{recursive:true});
  await writeFile(path.join(root,'src-ui/dist/index.html'),'<!doctype html><title>TEST_ONLY</title>');
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({name:'ArtifactApp',appId:'test.artifact',version:'0.1.0'}));
  await copyFile(source,selected);await chmod(selected,0o755);
  await writeFile(stale,'TEST_ONLY_STALE_DEFAULT_BINARY');
  const expected=hash(await readFile(selected));
  const bundle=path.join(root,'dist-bundle/ArtifactApp');
  for(const scenario of ['stale-default','missing-default']) {
    const result=spawnSync(selected,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
    assert.equal(result.status,0,result.stderr||result.error?.message);
    assert.equal(hash(await readFile(path.join(bundle,name))),expected,`Actual packaged executable must match the running selected artifact: ${scenario}`);
    console.log(`PASS: actual core bundle copies selected executable bytes with ${scenario}`);
    await rm(bundle,{recursive:true});
    if(scenario==='stale-default') await rm(stale);
  }
} finally {
  const relative=path.relative(tmpdir(),root);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
}
