#!/usr/bin/env node
import {mkdtemp, access, rm} from 'node:fs/promises';
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
  const result = spawnSync(process.execPath, args, {stdio: 'inherit', windowsHide: true, timeout: 120000});
  if (result.status !== 0) throw new Error('Generated-project acceptance failed');
}
try {
  run([path.resolve('aegis-cli/bin/aegis.js'), 'create', project]);
  run([npmCli, 'ci', '--prefix', path.join(project, 'src-ui')]);
  run([npmCli, '--prefix', path.join(project, 'src-ui'), 'run', 'build']);
  await access(path.join(project, 'src-ui/dist/index.html'));
  console.log('PASS: generated source project installs locked UI dependencies and builds');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
