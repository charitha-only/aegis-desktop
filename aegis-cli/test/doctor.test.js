import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync,spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {mkdtemp,readdir,rm,access,mkdir,copyFile,chmod} from 'node:fs/promises';
import {createServer} from 'node:http';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {inspectProject} from '../lib/doctor.js';
const cli=fileURLToPath(new URL('../bin/aegis.js',import.meta.url));
test('doctor reports project prerequisites as JSON without starting application services',()=>{
  const result=spawnSync(process.execPath,[cli,'doctor','--json'],{encoding:'utf8',windowsHide:true,timeout:30000});
  const report=JSON.parse(result.stdout);
  assert.equal(report.schemaVersion,1);
  assert.equal(report.platform,process.platform);
  for(const id of ['node','cargo','npm','configuration','rust-lockfile','ui-lockfile','ui-dependencies']) {
    assert.equal(typeof report.checks.find(check=>check.id===id)?.ok,'boolean',id);
  }
  assert.equal(result.status,report.ok?0:1);
  assert.equal(report.securityCertification,false);
});
test('doctor identifies missing setup without creating project files',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-doctor-'));
  try {
    const report=await inspectProject(fixture);
    assert.equal(report.ok,false);
    for(const id of ['configuration','rust-lockfile','ui-lockfile','ui-dependencies']) assert.equal(report.checks.find(check=>check.id===id).ok,false);
    assert.deepEqual(await readdir(fixture),[]);
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('doctor identifies the same tools as release builds and checks the core Rust compiler prerequisite',()=>{
  const diagnostic=spawnSync(process.execPath,[cli,'doctor','--json'],{encoding:'utf8',windowsHide:true,timeout:30000});
  const report=JSON.parse(diagnostic.stdout);
  const planned=spawnSync(process.execPath,[cli,'build','--print-plan'],{encoding:'utf8',windowsHide:true});
  assert.equal(planned.status,0,planned.stderr);
  const plan=JSON.parse(planned.stdout);
  assert.equal(report.tools.node,process.execPath);
  assert.equal(report.tools.cargo,plan.commands[0].executable);
  assert.equal(report.tools.npmCli,plan.commands[1].args[0]);
  assert.equal(typeof report.tools.rustc,'string');
  assert.equal(typeof report.checks.find(check=>check.id==='rustc')?.ok,'boolean');
});
test('doctor reports missing Rust tools without failing JSON diagnostics or relying on PATH npm',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-doctor-tools-'));
  try {
    const result=spawnSync(process.execPath,[cli,'doctor','--json'],{env:{...process.env,PATH:'',HOME:fixture,USERPROFILE:fixture,CARGO_HOME:fixture},encoding:'utf8',windowsHide:true,timeout:30000});
    assert.equal(result.status,1);
    const report=JSON.parse(result.stdout);
    assert.equal(report.ok,false);
    assert.equal(report.checks.find(check=>check.id==='cargo').ok,false);
    assert.equal(report.checks.find(check=>check.id==='rustc').ok,false);
    assert.equal(report.checks.find(check=>check.id==='npm').ok,true);
    assert.match(report.checks.find(check=>check.id==='rustc').detail,/1\.92/);
    assert.deepEqual(await readdir(fixture),[]);
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('doctor does not let Rustup auto-install a missing selected toolchain',async context=>{
  const root=path.resolve(path.dirname(cli),'../..');
  const tools=(await inspectProject(root)).tools;
  const rustup=path.join(path.dirname(tools.cargo),process.platform==='win32'?'rustup.exe':'rustup');
  try {await access(rustup);} catch {context.skip('Rustup proxy fixture requires an installed Rustup binary');return;}
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-doctor-rustup-'));
  let requests=0;
  const server=createServer((_,response)=>{requests++;response.writeHead(503,{'Connection':'close'});response.end('TEST_ONLY_no_distribution');});
  try {
    const cargoHome=path.join(fixture,'cargo');
    await mkdir(path.join(cargoHome,'bin'),{recursive:true});
    for(const name of ['cargo','rustc']) {
      const proxy=path.join(cargoHome,'bin',process.platform==='win32'?`${name}.exe`:name);
      await copyFile(rustup,proxy);await chmod(proxy,0o755);
    }
    await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
    const origin=`http://127.0.0.1:${server.address().port}`;
    const child=spawn(process.execPath,[cli,'doctor','--json'],{windowsHide:true,stdio:['ignore','pipe','pipe'],env:{...process.env,CARGO_HOME:cargoHome,RUSTUP_HOME:path.join(fixture,'rustup'),RUSTUP_TOOLCHAIN:'stable',RUSTUP_AUTO_INSTALL:'1',RUSTUP_DIST_SERVER:origin,RUSTUP_UPDATE_ROOT:origin}});
    let output='';child.stdout.on('data',data=>{output+=data;});child.stderr.resume();
    const status=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('close',resolve);});
    assert.equal(status,1);
    const report=JSON.parse(output);
    assert.equal(report.checks.find(check=>check.id==='rustc').ok,false);
    assert.equal(requests,0,'Diagnostics must not attempt a toolchain distribution download');
  } finally {
    if(server.listening) await new Promise(resolve=>server.close(resolve));
    await rm(fixture,{recursive:true,force:true});
  }
});
