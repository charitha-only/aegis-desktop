#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdtemp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {once} from 'node:events';
import {createServer} from '../../src-ui/node_modules/vite/dist/node/index.js';

const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-sdk-'));
const binary = path.resolve(`src-core/target/release/aegis-core${process.platform === 'win32' ? '.exe' : ''}`);
const child = spawn(binary, ['--port=0'], {cwd: fixture, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
  ...process.env, HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
  XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache'),
}});
const ended = once(child, 'close');
const servers = [];
async function client(origin) {
  const server = await createServer({root: path.resolve('src-ui'), configFile: false, envDir: false, mode: 'development',
    server: {middlewareMode: true, watch: null, hmr: false, ws: false},
    define: {'import.meta.env.PROD': 'false', 'import.meta.env.VITE_AEGIS_BRIDGE_URL': JSON.stringify(origin)}});
  servers.push(server);
  return (await server.ssrLoadModule('/src/aegis-bridge.ts')).aegis;
}
try {
  const base = await new Promise((resolve, reject) => {
    let output = '';
    const timer = setTimeout(() => reject(new Error('SDK fixture startup timed out')), 15000);
    child.once('error', error => {clearTimeout(timer); reject(error);});
    child.once('exit', () => {clearTimeout(timer); reject(new Error('SDK fixture exited before ready'));});
    child.stderr.resume();
    child.stdout.on('data', data => {
      output = (output + data.toString()).slice(-8192);
      const match = output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
      if (match) {clearTimeout(timer); resolve(match[1]);}
    });
  });
  const sdk = await client(base);
  const platform = await sdk.platformInfo();
  assert.equal(platform.success, true, platform.error);
  assert.equal(platform.data.os, {win32:'windows',linux:'linux',darwin:'macos'}[process.platform]);
  const directories = await sdk.platformDirectories('sdkfixture');
  assert.equal(directories.success, true, directories.error);
  for (const name of ['data','config','cache']) assert.equal(path.isAbsolute(directories.data[name]), true);
  const security = await sdk.getSecurityStatus();
  assert.equal(security.success, true, security.error);
  assert.equal(security.data.nonceVerified, true);
  assert.equal(typeof security.data.activeCapabilities.crypto, 'boolean');
  const password='TEST_ONLY_password_123';
  const stored=await sdk.vaultEncryptStore('sdk-contract','TEST_ONLY_hello',password);
  assert.equal(stored.success,true,stored.error);
  assert.equal(stored.data.filename,'sdk-contract.vault');
  const decrypted=await sdk.vaultDecryptRetrieve(stored.data.filename,password);
  assert.equal(decrypted.success,true,decrypted.error);
  assert.equal(decrypted.data.decryptedContent,'TEST_ONLY_hello');
  const unlocked=await sdk.vaultSessionUnlock(stored.data.filename,password);
  if(platform.data.os==='windows' && security.data.windowsLifecycleMonitorActive && security.data.windowsSessionAvailable) {
    assert.equal(unlocked.success,true,unlocked.error);
    const read=await sdk.vaultSessionRead(stored.data.filename,unlocked.data.sessionToken);
    assert.equal(read.success,true,read.error);
    assert.equal(read.data.decryptedContent,'TEST_ONLY_hello');
  } else {
    assert.equal(unlocked.success,false,'Retained session must fail when lifecycle monitoring is unavailable');
  }
  assert.equal((await sdk.vaultSessionLock()).success,true);
  if(unlocked.success) assert.equal((await sdk.vaultSessionRead(stored.data.filename,unlocked.data.sessionToken)).success,false);
  const denied = await client('file:///not-an-ipc-origin');
  const init = await denied.init();
  assert.equal(init.success, false);
  assert.match(init.error, /Invalid IPC bridge origin/);
  const invoke = await denied.invoke('platform_info', {}, 'TEST_ONLY_invalid-nonce');
  assert.equal(invoke.success, false);
  assert.match(invoke.error, /Invalid IPC bridge origin/);
  console.log('PASS: real TypeScript SDK validates platform/security/vault contracts, retained-session lifecycle gates and invalid endpoints');
} finally {
  await Promise.all(servers.map(server => server.close()));
  if (child.exitCode === null) child.kill();
  await ended;
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
