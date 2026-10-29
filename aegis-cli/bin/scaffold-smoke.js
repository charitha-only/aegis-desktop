#!/usr/bin/env node
import {mkdtemp, access, rm, readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
import {verifyPackageIntegrity} from '../lib/package-integrity.js';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-template-'));
const project = path.join(fixture, 'ExampleApp');
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
  run([path.resolve('aegis-cli/bin/aegis.js'), 'create', project]);
  run([npmCli, 'ci', '--prefix', path.join(project, 'src-ui')]);
  const cli=path.join(project,'aegis-cli/bin/aegis.js');
  run([cli,'doctor','--json']);
  run([cli,'package']);
  await access(path.join(project, 'src-ui/dist/index.html'));
  const bundle=path.join(project,'dist-bundle/ExampleApp');
  const config=JSON.parse(await readFile(path.join(bundle,'aegis.config.json'),'utf8'));
  assert.equal(config.appId,'app.exampleapp');
  const {createHash}=await import('node:crypto');
  const pin=createHash('sha256').update(await readFile(path.join(bundle,'aegis-integrity.json'))).digest('hex');
  assert.equal((await verifyPackageIntegrity(bundle,pin)).valid,true);
  await access(path.join(bundle,process.platform==='win32'?'aegis-core.exe':'aegis-core'));
  console.log('PASS: clean generated application passes doctor, locked Rust/UI builds and portable package integrity verification');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
