#!/usr/bin/env node
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,copyFile,chmod,rm,access} from 'node:fs/promises';
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
    if(process.platform==='win32') await rm(path.join(root,'dist-bundle/ArtifactApp-installer.iss'));
    if(scenario==='stale-default') await rm(stale);
  }
  await rm(path.join(root,'dist-bundle'),{recursive:true,force:true});
  const base={name:'ArtifactApp',appId:'test.artifact',version:'0.1.0'};
  for(const config of [null,[],{}, {name:'ArtifactApp'}, {version:'0.1.0'},
    {...base,version:123}, {...base,version:null}, {...base,name:false},
    {...base,author:{}}, {...base,description:[]}, {...base,icon:7}]) {
    await writeFile(path.join(root,'aegis.config.json'),JSON.stringify(config));
    const invalid=spawnSync(selected,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
    assert.ok(Number.isInteger(invalid.status)&&invalid.status>0,'Invalid explicit metadata must fail instead of silently substituting defaults');
    assert.match(invalid.stderr,/Bundle configuration/i);
    await assert.rejects(access(path.join(root,'dist-bundle')),{code:'ENOENT'});
  }
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({...base,icon:null}));
  const valid=spawnSync(selected,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
  assert.equal(valid.status,0,valid.stderr);
  const metadata=JSON.parse(await readFile(path.join(bundle,'aegis-bundle.json'),'utf8'));
  assert.equal(metadata.version,'0.1.0');assert.equal(metadata.name,'ArtifactApp');
  assert.equal(metadata.author,'Aegis Developer');
  assert.equal(metadata.description,'Secure Aegis Desktop Application');
  assert.equal(metadata.icon,null);
  console.log('PASS: actual CLI bundle rejects invalid metadata before output and preserves explicit identity plus absent optional defaults');
} finally {
  const relative=path.relative(tmpdir(),root);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
}
