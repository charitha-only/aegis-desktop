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
    await access(path.join(destination, 'src-core/Cargo.toml'));
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
