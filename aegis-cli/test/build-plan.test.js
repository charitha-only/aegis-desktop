import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,rm,readdir,mkdir,writeFile,readFile,cp} from 'node:fs/promises';
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


test('bundle runs the newly compiled Cargo artifact when a target setting leaves a stale default binary',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis target artifact '));
  const {resolveDeveloperTools}=await import('../lib/developer-tools.js');
  const tools=await resolveDeveloperTools(path.resolve('.'));
  const env={...process.env}; delete env.CARGO_BUILD_TARGET;
  const run=(file,args,options={})=>{
    const result=spawnSync(file,args,{cwd:fixture,env,encoding:'utf8',windowsHide:true,timeout:120000,...options});
    assert.equal(result.status,0,result.stderr||result.error?.message); return result;
  };
  try {
    const compiler=run(tools.rustc,['-vV']);
    const host=compiler.stdout.match(/^host: (\S+)$/m)?.[1];assert.ok(host);
    await cp(fileURLToPath(new URL('..',import.meta.url)),path.join(fixture,'aegis-cli'),{recursive:true});
    await mkdir(path.join(fixture,'src-core/src'),{recursive:true});
    await mkdir(path.join(fixture,'src-ui/src'),{recursive:true});
    await cp(fileURLToPath(new URL('../../src-ui/src/bridge-origin.mjs',import.meta.url)),path.join(fixture,'src-ui/src/bridge-origin.mjs'));
    await writeFile(path.join(fixture,'src-core/Cargo.toml'),'[package]\nname="aegis-core"\nversion="0.1.0"\nedition="2021"\n');
    const source=marker=>`fn main() { if std::env::args().any(|x| x == "--bundle") { std::fs::write("bundled-marker.txt", "${marker}").unwrap(); } }`;
    await writeFile(path.join(fixture,'src-core/src/main.rs'),source('STALE'));
    run(tools.cargo,['generate-lockfile','--manifest-path','src-core/Cargo.toml']);
    run(tools.cargo,['build','--locked','--release','--manifest-path','src-core/Cargo.toml','--target-dir',path.join(fixture,'src-core/target')]);
    await writeFile(path.join(fixture,'src-core/src/main.rs'),source('NEW_ARTIFACT'));
    await writeFile(path.join(fixture,'src-ui/package.json'),JSON.stringify({scripts:{build:'node -e "console.log(123)"'}}));
    run(process.execPath,[path.join(fixture,'aegis-cli/bin/aegis.js'),'bundle'],{env:{...env,CARGO_BUILD_TARGET:host}});
    assert.equal(await readFile(path.join(fixture,'bundled-marker.txt'),'utf8'),'NEW_ARTIFACT','Bundling must use the artifact reported by this successful Cargo build');
    await mkdir(path.join(fixture,'.cargo'));
    await writeFile(path.join(fixture,'.cargo/config.toml'),`[build]\ntarget="${host}"\n`);
    await writeFile(path.join(fixture,'src-core/src/main.rs'),source('CONFIG_ARTIFACT'));
    run(process.execPath,[path.join(fixture,'aegis-cli/bin/aegis.js'),'bundle']);
    assert.equal(await readFile(path.join(fixture,'bundled-marker.txt'),'utf8'),'CONFIG_ARTIFACT');
  } finally {await rm(fixture,{recursive:true,force:true});}
});
