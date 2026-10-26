#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawn, spawnSync} from 'node:child_process';
import {mkdtemp, mkdir, cp, rename, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-identity-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
const env = {...process.env, HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
  XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache')};
async function boot(app, use) {
  const child = spawn(path.join(app, executable), ['--port=0'], {cwd: fixture, env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe']});
  const done = new Promise(resolve => child.once('close', resolve));
  try {
    const base = await new Promise((resolve, reject) => {
      let output = '';
      const timer = setTimeout(() => reject(new Error('Identity fixture startup timed out')), 15000);
      child.once('error', error => {clearTimeout(timer); reject(error);});
      child.once('exit', () => {clearTimeout(timer); reject(new Error('Identity fixture exited before ready'));});
      child.stderr.resume();
      child.stdout.on('data', data => {
        output = (output + data.toString()).slice(-8192);
        const match = output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
        if (match) {clearTimeout(timer); resolve(match[1]);}
      });
    });
    const handshake = await fetch(`${base}/handshake`, {signal: AbortSignal.timeout(5000)}).then(r => r.json());
    let seq = 0;
    const call = async (command, payload) => fetch(`${base}/ipc`, {method: 'POST', headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({id: String(++seq), command, payload, nonce: handshake.token}), signal: AbortSignal.timeout(10000)}).then(r => r.json());
    await use(call);
  } finally {
    if (child.pid && child.exitCode === null) child.kill();
    await done;
  }
}
try {
  let app = path.join(fixture, 'FirstPackageFolder');
  await mkdir(path.join(app, 'ui'), {recursive: true});
  await writeFile(path.join(app, 'ui/index.html'), '<!doctype html><title>Owned identity fixture</title>');
  await cp(path.resolve('src-core/target/release', executable), path.join(app, executable));
  const config = JSON.parse(await readFile('aegis.config.json', 'utf8'));
  config.appId = 'org.example.identity-fixture';
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  const payload = {filename: 'identity.vault', content: 'Retained across package rename', passphrase: 'TEST_ONLY_identity-password'};
  await boot(app, async call => assert.equal((await call('vault_encrypt_store', payload)).success, true));
  const renamed = path.join(fixture, 'CompletelyRenamedPackage');
  await rename(app, renamed);
  app = renamed;
  await boot(app, async call => {
    const result = await call('vault_decrypt_retrieve', {filename: payload.filename, passphrase: payload.passphrase});
    assert.equal(result.success, true, 'Configured identity must retain vault access after package-folder rename');
    assert.equal(result.data.decryptedContent, payload.content);
  });
  config.appId = '../escape';
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  const invalid = spawnSync(path.join(app, executable), ['--port=0'], {cwd: fixture, env, windowsHide: true, encoding: 'utf8', timeout: 10000});
  assert.equal(invalid.error, undefined, 'Invalid identity must exit instead of starting a server');
  assert.notEqual(invalid.status, 0);
  assert.notEqual(invalid.status, null);
  console.log('PASS: packaged identity survives rename and invalid identity fails closed');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
