import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,rm,readdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const cli=fileURLToPath(new URL('../bin/aegis.js',import.meta.url));
test('build plan is read-only without Cargo on PATH and pins locked output plus Node-owned npm',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-build-plan-'));
  try {
    const result=spawnSync(process.execPath,[cli,'build','--print-plan'],{cwd:fixture,env:{...process.env,PATH:'',HOME:fixture,USERPROFILE:fixture,CARGO_HOME:fixture},encoding:'utf8',windowsHide:true,timeout:15000});
    assert.equal(result.status,0,result.stderr);
    const plan=JSON.parse(result.stdout);
    assert.equal(plan.commands.length,2);
    const [core,ui]=plan.commands;
    assert.equal(core.args.includes('--locked'),true);
    const target=core.args[core.args.indexOf('--target-dir')+1];
    assert.equal(target,path.join(core.cwd,'src-core/target'));
    assert.equal(ui.executable,process.execPath);
    assert.match(ui.args[0],/npm-cli\.js$/);
    assert.deepEqual(ui.args.slice(1),['run','build']);
    assert.equal(core.shell,false);assert.equal(ui.shell,false);
    assert.deepEqual(await readdir(fixture),[]);
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('release aliases and bundling expose the same pinned toolchain without executing it',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-release-plan-'));
  try {
    for(const command of ['build-release','bundle']) {
      const result=spawnSync(process.execPath,[cli,command,'--print-plan'],{cwd:fixture,env:{...process.env,PATH:'',HOME:fixture,USERPROFILE:fixture,CARGO_HOME:fixture},encoding:'utf8',windowsHide:true});
      assert.equal(result.status,0,result.stderr);
      const plan=JSON.parse(result.stdout);
      assert.equal(plan.commands[0].args.includes('--locked'),true);
      assert.equal(plan.commands[1].executable,process.execPath);
      if(command==='bundle') {
        assert.equal(plan.commands.length,3);
        assert.deepEqual(plan.commands[2].args,['--bundle']);
        assert.equal(plan.commands[2].executable,path.join(plan.commands[0].cwd,'src-core/target/release',process.platform==='win32'?'aegis-core.exe':'aegis-core'));
      }
    }
    assert.deepEqual(await readdir(fixture),[]);
  } finally {await rm(fixture,{recursive:true,force:true});}
});
