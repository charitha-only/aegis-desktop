#!/usr/bin/env node
import {mkdtemp, access, rm, readFile, readdir} from 'node:fs/promises';
import assert from 'node:assert/strict';
import {verifyPackageIntegrity} from '../lib/package-integrity.js';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-template-'));
const nodeDir = path.dirname(process.execPath);
const candidates = [path.join(nodeDir, 'node_modules/npm/bin/npm-cli.js'), path.join(nodeDir, '../lib/node_modules/npm/bin/npm-cli.js')];
let npmCli;
for (const candidate of candidates) {try {await access(candidate); npmCli = candidate; break;} catch {}}
if (!npmCli) throw new Error('Cannot locate npm-cli.js beside the Node runtime');
function run(args) {
  const result = spawnSync(process.execPath, args, {stdio: 'inherit', windowsHide: true, timeout: 900000});
  if (result.status !== 0) throw new Error('Generated-project acceptance failed');
}
try {
  for(const [template,name] of [['react','ExampleApp'],['vanilla','VanillaApp']]) {
    const project=path.join(fixture,name);
    run([path.resolve('aegis-cli/bin/aegis.js'), 'create', project, `--template=${template}`]);
    run([npmCli, 'ci', '--prefix', path.join(project, 'src-ui')]);
    const cli=path.join(project,'aegis-cli/bin/aegis.js');
    run([cli,'doctor','--json']);
    run([cli,'package']);
    await access(path.join(project, 'src-ui/dist/index.html'));
    const bundle=path.join(project,'dist-bundle',name);
    const config=JSON.parse(await readFile(path.join(bundle,'aegis.config.json'),'utf8'));
    assert.equal(config.appId,`app.${name.toLowerCase()}`);
    const {createHash}=await import('node:crypto');
    const pin=createHash('sha256').update(await readFile(path.join(bundle,'aegis-integrity.json'))).digest('hex');
    assert.equal((await verifyPackageIntegrity(bundle,pin)).valid,true);
    await access(path.join(bundle,process.platform==='win32'?'aegis-core.exe':'aegis-core'));
    if(template==='vanilla') {
      const assets=path.join(project,'src-ui/dist/assets');
      let javascriptBytes=0;
      for(const file of await readdir(assets)) if(file.endsWith('.js')) javascriptBytes+=(await readFile(path.join(assets,file))).byteLength;
      assert.ok(javascriptBytes>0 && javascriptBytes<=64*1024,`Vanilla starter JS exceeds the explicit 64 KiB fixture budget: ${javascriptBytes}`);
      console.log(`Vanilla starter raw JavaScript bytes: ${javascriptBytes}; budget: 65536`);
    }
    console.log(`PASS: ${template} generated application passes doctor, locked Rust/UI builds and portable package integrity verification`);
  }
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
