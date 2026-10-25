#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {once} from 'node:events';

// Real release-binary IPC checks; all writes stay in an owned temporary project.
const binary = path.resolve(process.argv[2] || `src-core/target/release/aegis-core${process.platform === 'win32' ? '.exe' : ''}`);
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-portable-'));
const child = spawn(binary, ['--port=0'], {cwd: fixture, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
  ...process.env, HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
  XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache'),
}});
const ended = once(child, 'exit');
try {
  const base = await new Promise((resolve, reject) => {
    let output = '';
    const timer = setTimeout(() => reject(new Error('Core startup timed out')), 15000);
    child.once('error', error => { clearTimeout(timer); reject(error); });
    child.once('exit', () => { clearTimeout(timer); reject(new Error('Core exited before ready')); });
    child.stdout.on('data', chunk => {
      output = (output + chunk.toString()).slice(-8192);
      const match = output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
      if (match) { clearTimeout(timer); resolve(match[1]); }
    });
    child.stderr.resume();
  });
  const handshake = await fetch(`${base}/handshake`, {signal: AbortSignal.timeout(5000)}).then(r => r.json());
  const browserHandshake = await fetch(`${base}/handshake`, {headers: {Origin: base}, signal: AbortSignal.timeout(5000)});
  assert.equal(browserHandshake.status, 200, 'The actual bound application origin must work with a dynamically allocated port');
  assert.equal((await fetch(`${base}/handshake`, {headers: {Origin: 'https://untrusted.example'}, signal: AbortSignal.timeout(5000)})).status, 403);
  let seq = 0;
  const call = async (command, payload = {}) => fetch(`${base}/ipc`, {
    method: 'POST', headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({id: String(++seq), command, payload, nonce: handshake.token}), signal: AbortSignal.timeout(10000),
  }).then(r => r.json());
  const info = await call('platform_info');
  assert.equal(info.success, true, 'Platform capability interface must exist');
  assert.equal(info.data.os, {win32: 'windows', linux: 'linux', darwin: 'macos'}[process.platform]);
  const dirs = await call('platform_directories', {appId: 'PortableFixture'});
  assert.equal(dirs.success, true);
  for (const key of ['data', 'config', 'cache']) assert.equal(path.isAbsolute(dirs.data[key]), true);
  assert.equal((await call('platform_directories', {appId: '../outside'})).success, false);
  const password = 'TEST_ONLY_portable-fixture-password';
  assert.equal((await call('vault_encrypt_store', {filename: 'fixture.vault', content: 'fixture', passphrase: password})).success, true);
  const read = await call('vault_decrypt_retrieve', {filename: 'fixture.vault', passphrase: password});
  assert.equal(read.data.decryptedContent, 'fixture');
  if (process.platform !== 'win32') {
    assert.equal((await call('vault_session_unlock', {filename: 'fixture.vault', passphrase: password})).success, false,
      'Retained sessions require an available lifecycle adapter');
  }
  console.log('PASS: native platform IPC, directories, encrypted storage and unavailable lifecycle behavior');
} finally {
  if (child.exitCode === null) child.kill();
  await ended;
  await rm(fixture, {recursive: true, force: true});
}
