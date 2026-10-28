#!/usr/bin/env node
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
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
  const bundleRoot=path.join(fixture,'dist-bundle/AegisFixture');
  const cli=path.join(repo,'aegis-cli/bin/aegis.js');
  const manifest=spawnSync(process.execPath,[cli,'package-manifest',bundleRoot],{encoding:'utf8',windowsHide:true,timeout:30000});
  assert.equal(manifest.status,0,manifest.stderr);
  const pin=JSON.parse(manifest.stdout).manifestSha256;
  const verified=spawnSync(process.execPath,[cli,'verify-package',bundleRoot,pin],{encoding:'utf8',windowsHide:true,timeout:30000});
  assert.equal(verified.status,0,verified.stderr);
  assert.equal(JSON.parse(verified.stdout).valid,true);
  const runtime = path.join(fixture, 'dist-bundle/AegisFixture', executable);
  const smoke = spawnSync(process.execPath, [path.join(repo, 'aegis-cli/bin/portable-smoke.js'), runtime], {cwd: repo, stdio: 'inherit', windowsHide: true, timeout: 30000});
  if (smoke.status !== 0) throw new Error('Packaged runtime smoke failed');
  await writeFile(path.join(bundleRoot,'ui/index.html'),'TEST_ONLY altered asset');
  const tampered=spawnSync(process.execPath,[cli,'verify-package',bundleRoot,pin],{encoding:'utf8',windowsHide:true,timeout:30000});
  assert.equal(tampered.status,1);
  assert.match(tampered.stderr,/mismatch/);
  console.log('PASS: real package integrity CLI detects modified bundled assets with an independently retained pin');
  console.log('PASS: isolated native portable bundle boots and serves real IPC');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
