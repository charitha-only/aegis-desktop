#!/usr/bin/env node
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted') {
  throw new Error('Native GUI smoke is restricted to disposable GitHub-hosted runners');
}
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-native-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
try {
  const app = path.join(fixture, 'NativeFixture');
  await mkdir(path.join(app, 'ui'), {recursive: true});
  await cp(path.resolve('src-core/target/debug', executable), path.join(app, executable));
  const config = JSON.parse(await readFile('aegis.config.json', 'utf8'));
  config.window.width = 800; config.window.height = 600;
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  await writeFile(path.join(app, 'ui/index.html'), '<!doctype html><html><head><title>Aegis Native Smoke Fixture</title></head><body><h1>Native fixture</h1></body></html>');
  const child = spawn(path.join(app, executable), ['--port=0', '--native-window'], {cwd: app, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
    ...process.env, AEGIS_NATIVE_SMOKE: '1', HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
    XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache'),
  }});
  let output = '';
  for (const stream of [child.stdout, child.stderr]) stream.on('data', data => {output = (output + data.toString()).slice(-16000);});
  const timer = setTimeout(() => child.kill(), 30000);
  const code = await new Promise((resolve, reject) => {child.once('error', reject); child.once('exit', resolve);});
  clearTimeout(timer);
  if (code !== 0 || !output.includes('AEGIS_NATIVE_SMOKE_READY')) throw new Error(`Native GUI fixture failed: ${output}`);
  console.log('PASS: native dynamic-port window created, fixture document loaded and process exited');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true, maxRetries: 10, retryDelay: 100});
}
