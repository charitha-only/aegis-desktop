#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-launch-plan-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
const app = path.join(fixture, 'LaunchFixture');
const env = {...process.env, HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
  XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache')};
function plan(...args) {
  const result = spawnSync(path.join(app, executable), ['--port=0', '--print-launch-plan', ...args],
    {cwd: fixture, env, windowsHide: true, encoding: 'utf8', timeout: 15000, maxBuffer: 32768});
  assert.equal(result.error, undefined, 'Launch inspection must finish without starting a desktop or server loop');
  assert.equal(result.status, 0, result.stderr);
  const line = result.stdout.split(/\r?\n/).find(line => line.startsWith('AEGIS_LAUNCH_PLAN='));
  assert.ok(line, 'Core must expose its effective launch target');
  const value = JSON.parse(line.slice('AEGIS_LAUNCH_PLAN='.length));
  const bridge = new URL(value.bridgeUrl);
  assert.equal(bridge.hostname, '127.0.0.1');
  assert.ok(Number(bridge.port) > 0);
  assert.equal(JSON.stringify(value).includes('token'), false);
  return value;
}
try {
  await mkdir(path.join(app, 'ui'), {recursive: true});
  await cp(path.resolve('src-core/target/release', executable), path.join(app, executable));
  await writeFile(path.join(app, 'ui/index.html'), '<!doctype html><title>Owned launch fixture</title>');
  const config = JSON.parse(await readFile('aegis.config.json', 'utf8'));
  config.appId = 'org.example.launch-plan-fixture';
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  for (const args of [[], ['--native-window'], ['--spawn-window']]) {
    const value = plan(...args);
    assert.equal(value.packaged, true);
    assert.equal(value.uiUrl, value.bridgeUrl, 'Packaged UI must use the actual listener, never the requested port zero');
  }
  await rm(path.join(app, 'ui/index.html'));
  const development = plan();
  assert.equal(development.packaged, false);
  assert.equal(development.uiUrl, 'http://localhost:5173', 'Development UI remains on its separate Vite origin');
  console.log('PASS: packaged dynamic-port launch targets and separate development UI, without desktop interaction');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
