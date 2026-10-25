#!/usr/bin/env node
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const repo = process.cwd();
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-package-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
try {
  const release = path.join(fixture, 'src-core/target/release');
  await mkdir(release, {recursive: true});
  await cp(path.join(repo, 'src-core/target/release', executable), path.join(release, executable));
  await cp(path.join(repo, 'src-ui/dist'), path.join(fixture, 'src-ui/dist'), {recursive: true});
  const config = JSON.parse(await readFile(path.join(repo, 'aegis.config.json'), 'utf8'));
  config.name = 'AegisFixture';
  config.version = '0.0.0-test';
  await writeFile(path.join(fixture, 'aegis.config.json'), JSON.stringify(config));
  const bundle = spawnSync(path.join(release, executable), ['--bundle'], {cwd: fixture, stdio: 'inherit', windowsHide: true, timeout: 30000});
  if (bundle.status !== 0) throw new Error('Native portable bundling failed');
  const runtime = path.join(fixture, 'dist-bundle/AegisFixture', executable);
  const smoke = spawnSync(process.execPath, [path.join(repo, 'aegis-cli/bin/portable-smoke.js'), runtime], {cwd: repo, stdio: 'inherit', windowsHide: true, timeout: 30000});
  if (smoke.status !== 0) throw new Error('Packaged runtime smoke failed');
  console.log('PASS: isolated native portable bundle boots and serves real IPC');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
