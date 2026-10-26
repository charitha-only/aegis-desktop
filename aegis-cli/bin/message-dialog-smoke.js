#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawn, spawnSync} from 'node:child_process';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';

if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted' || process.platform !== 'linux') {
  throw new Error('Message UI fixture is restricted to disposable Linux hosted runners');
}
const binary = path.resolve('src-core/target/debug/aegis-core');
async function exercise(key, acknowledge) {
const child = spawn(binary, ['--message-dialog-helper'], {stdio: ['pipe', 'pipe', 'pipe'], env: {...process.env, ZENITY_OK:'9', ZENITY_CANCEL:'0', ZENITY_ESC:'0'}});
const done = new Promise((resolve, reject) => {child.once('close', resolve); child.once('error', reject);});
let stdout = '', stderr = '';
child.stdout.on('data', data => {stdout = (stdout + data).slice(-8192);});
child.stderr.on('data', data => {stderr = (stderr + data).slice(-8192);});
child.stdin.on('error', () => {});
child.stdin.end(JSON.stringify({title: 'Aegis Message Fixture', message: 'Literal <b>text</b> අ\nFixture only', kind: 'info'}));
try {
  const deadline = Date.now() + 15000;
  let windowId;
  while (Date.now() < deadline && child.exitCode === null) {
    const search = spawnSync('xdotool', ['search', '--onlyvisible', '--pid', String(child.pid), '--name', '^Aegis Message Fixture$'], {encoding: 'utf8', timeout: 2000});
    if (search.status === 0 && /^\d+/.test(search.stdout)) {
      windowId = search.stdout.trim().split('\n')[0];
      break;
    }
    await delay(100);
  }
  assert.ok(windowId, `Owned message window did not appear: ${stderr}`);
  const focus = spawnSync('xdotool', ['windowfocus', '--sync', windowId], {encoding: 'utf8', timeout: 2000});
  assert.equal(focus.status, 0, `Owned visible window focus failed: ${focus.stderr}`);
  assert.equal(spawnSync('xdotool', ['key', key], {timeout: 2000}).status, 0);
  let timeout;
  const code = await Promise.race([done, new Promise((_, reject) => {timeout = setTimeout(() => reject(new Error('Owned dialog did not respond')), 10000);})]).finally(() => clearTimeout(timeout));
  if (acknowledge) assert.equal(code, 0, stderr);
  else assert.notEqual(code, 0, 'Escape must not be misreported as acknowledgement');
  assert.equal(stdout, '', 'Successful zenity acknowledgement has empty stdout');

} finally {
  if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  await done.catch(() => {});
}
}
await exercise('Return', true);
await exercise('Escape', false);
console.log('Actual Linux message windows acknowledged/dismissed correctly despite inherited exit-code overrides');
