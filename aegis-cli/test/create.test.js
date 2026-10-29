import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, readFile, writeFile, access, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('CLI exposes discoverable help and framework version without starting services', () => {
  const cli = path.join(repo, 'aegis-cli/bin/aegis.js');
  const help = spawnSync(process.execPath, [cli, '--help'], {encoding: 'utf8', windowsHide: true});
  assert.equal(help.status, 0);
  assert.match(help.stdout, /create/);
  assert.match(help.stdout, /bundle/);
  const version = spawnSync(process.execPath, [cli, '--version'], {encoding: 'utf8', windowsHide: true});
  assert.equal(version.status, 0);
  assert.match(version.stdout.trim(), /^\d+\.\d+\.\d+$/);
});

test('create produces a self-contained source project and refuses to overwrite it', async () => {
  const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-create-'));
  const destination = path.join(fixture, 'ExampleApp');
  const cli = path.join(repo, 'aegis-cli/bin/aegis.js');
  try {
    const run = () => spawnSync(process.execPath, [cli, 'create', destination], {encoding: 'utf8', windowsHide: true, timeout: 10000});
    const created = run();
    assert.equal(created.status, 0, created.stderr);
    assert.equal(JSON.parse(await readFile(path.join(destination, 'aegis.config.json'), 'utf8')).name, 'ExampleApp');
    assert.equal(JSON.parse(await readFile(path.join(destination, 'aegis.config.json'), 'utf8')).appId, 'app.exampleapp');
    await access(path.join(destination, 'src-core/Cargo.toml'));
    await access(path.join(destination, 'src-core/vendor/tiny_http/src/connection.rs'));
    await access(path.join(destination, 'src-core/vendor/tiny_http/LICENSE-MIT'));
    await access(path.join(destination, 'src-ui/src/App.tsx'));
    await assert.rejects(access(path.join(destination, 'sandbox')));
    await assert.rejects(access(path.join(destination, 'src-core/target')));
    await assert.rejects(access(path.join(destination, '.git')));
    const before = await readFile(path.join(destination, 'src-ui/src/App.tsx'), 'utf8');
    assert.notEqual(run().status, 0);
    assert.equal(await readFile(path.join(destination, 'src-ui/src/App.tsx'), 'utf8'), before);
    await writeFile(path.join(destination, 'src-ui/src/.env'), 'TEST_ONLY=value');
    const second = path.join(fixture, 'SecondApp');
    const rejected = spawnSync(process.execPath, [path.join(destination, 'aegis-cli/bin/aegis.js'), 'create', second], {encoding: 'utf8', windowsHide: true});
    assert.notEqual(rejected.status, 0, 'Credential-shaped files must not enter source templates');
    await assert.rejects(access(second));
  } finally {
    const relative = path.relative(tmpdir(), fixture);
    if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
  }
});
test('create offers a vanilla TypeScript starter with the shared SDK and application identity',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-vanilla-'));
  try {
    const destination=path.join(fixture,'SmallApp');
    const result=spawnSync(process.execPath,[path.join(repo,'aegis-cli/bin/aegis.js'),'create',destination,'--template=vanilla'],{encoding:'utf8',windowsHide:true});
    assert.equal(result.status,0,result.stderr);
    assert.match(await readFile(path.join(destination,'src-ui/index.html'),'utf8'),/\/src\/main\.ts"/);
    await access(path.join(destination,'src-ui/src/main.ts'));
    await access(path.join(destination,'src-ui/src/ipc-contract.ts'));
    await assert.rejects(access(path.join(destination,'src-ui/src/App.tsx')));
    await assert.rejects(access(path.join(destination,'src-ui/src/main.tsx')));
    assert.equal(JSON.parse(await readFile(path.join(destination,'aegis.config.json'),'utf8')).appId,'app.smallapp');
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('create rejects malformed source configuration before publishing a partial project',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-template-config-'));
  const cli=path.join(repo,'aegis-cli/bin/aegis.js');
  try {
    const source=path.join(fixture,'Source');
    assert.equal(spawnSync(process.execPath,[cli,'create',source],{encoding:'utf8',windowsHide:true}).status,0);
    for(const [index,configuration] of ['{','null','{"window":null}'].entries()) {
      await writeFile(path.join(source,'aegis.config.json'),configuration);
      const destination=path.join(fixture,`Rejected${index}`);
      const result=spawnSync(process.execPath,[path.join(source,'aegis-cli/bin/aegis.js'),'create',destination],{encoding:'utf8',windowsHide:true});
      assert.notEqual(result.status,0);
      await assert.rejects(access(destination));
    }
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('create rejects unsupported and repeated template options before creating output',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-template-options-'));
  try {
    for(const [index,args] of [['--template=unknown'],['--template=react','--template=vanilla'],['--wrong']].entries()) {
      const destination=path.join(fixture,`Rejected${index}`);
      const result=spawnSync(process.execPath,[path.join(repo,'aegis-cli/bin/aegis.js'),'create',destination,...args],{encoding:'utf8',windowsHide:true});
      assert.notEqual(result.status,0);
      await assert.rejects(access(destination));
    }
  } finally {await rm(fixture,{recursive:true,force:true});}
});
test('a generated vanilla project can itself create either supported starter',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-template-reuse-'));
  try {
    const source=path.join(fixture,'VanillaSource');
    const initial=spawnSync(process.execPath,[path.join(repo,'aegis-cli/bin/aegis.js'),'create',source,'--template=vanilla'],{encoding:'utf8',windowsHide:true});
    assert.equal(initial.status,0,initial.stderr);
    for(const template of ['vanilla','react']) {
      const output=path.join(fixture,template==='react'?'ReactAgain':'VanillaAgain');
      const result=spawnSync(process.execPath,[path.join(source,'aegis-cli/bin/aegis.js'),'create',output,`--template=${template}`],{encoding:'utf8',windowsHide:true});
      assert.equal(result.status,0,result.stderr);
      await access(path.join(output,'src-ui/src',template==='react'?'main.tsx':'main.ts'));
      await assert.rejects(access(path.join(output,'src-ui/src',template==='react'?'main.ts':'main.tsx')));
    }
  } finally {await rm(fixture,{recursive:true,force:true});}
});
