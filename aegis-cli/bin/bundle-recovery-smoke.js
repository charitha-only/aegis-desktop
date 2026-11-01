#!/usr/bin/env node
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,copyFile,chmod,rm,access,readdir} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import path from 'node:path';
const root=await mkdtemp(path.join(tmpdir(),'aegis bundle recovery '));
const name=process.platform==='win32'?'aegis-core.exe':'aegis-core';
const blocked=path.join(root,'src-ui/dist/blocked.txt');
try {
  const core=path.join(root,name),output=path.join(root,'dist-bundle'),bundle=path.join(output,'RecoveryApp');
  await copyFile(path.resolve('src-core/target/release',name),core);await chmod(core,0o755);
  await mkdir(path.join(root,'src-ui/dist'),{recursive:true});
  await writeFile(path.join(root,'src-ui/dist/index.html'),'TEST_ONLY_UI');
  await writeFile(blocked,'TEST_ONLY_SOURCE');
  await writeFile(path.join(root,'aegis.config.json'),JSON.stringify({name:'RecoveryApp',appId:'test.recovery',version:'0.1.0'}));
  const sidecar=path.join(output,'RecoveryApp-installer.iss');
  if(process.platform==='win32') {await mkdir(sidecar,{recursive:true});await writeFile(path.join(sidecar,'owner.txt'),'TEST_ONLY_PRESERVE');}
  else await chmod(blocked,0o000);
  const result=spawnSync(core,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
  assert.ok(Number.isInteger(result.status)&&result.status>0,'Owned filesystem obstruction must fail packaging');
  await assert.rejects(access(bundle),{code:'ENOENT'});
  assert.match(result.stderr,process.platform==='win32'?/Installer sidecar/:/Failed to copy frontend/);
  assert.equal((await readdir(output)).some(entry=>entry.startsWith('.aegis-')),false,'Failure must clean owned staging/lock files');
  if(process.platform==='win32') {
    assert.equal(await readFile(path.join(sidecar,'owner.txt'),'utf8'),'TEST_ONLY_PRESERVE');
    await rm(sidecar,{recursive:true});
  } else await chmod(blocked,0o600);
  await mkdir(output,{recursive:true});
  const lock=path.join(output,'.aegis-bundle-RecoveryApp.lock');
  await writeFile(lock,'TEST_ONLY_OTHER_BUILD',{flag:'wx'});
  const busy=spawnSync(core,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
  assert.ok(Number.isInteger(busy.status)&&busy.status>0);
  assert.match(busy.stderr,/build lock unavailable/);
  assert.equal(await readFile(lock,'utf8'),'TEST_ONLY_OTHER_BUILD');
  await assert.rejects(access(bundle),{code:'ENOENT'});
  await rm(lock);
  const retried=spawnSync(core,['--bundle'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:20000});
  assert.equal(retried.status,0,retried.stderr);
  assert.equal(await readFile(path.join(bundle,'ui/blocked.txt'),'utf8'),'TEST_ONLY_SOURCE');
  if(process.platform==='win32') assert.doesNotMatch(await readFile(sidecar,'utf8'),/\.aegis-stage-/);
  assert.equal((await readdir(output)).some(entry=>entry.startsWith('.aegis-')),false,'Normal failure/success must clean owned staging and lock files');
  console.log('PASS: occupied build lock is preserved and prevents publication');
  console.log('PASS: failed package leaves no final directory, preserves inputs and retries after repair');
} finally {
  await chmod(blocked,0o600).catch(()=>{});
  const relative=path.relative(tmpdir(),root);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
}
