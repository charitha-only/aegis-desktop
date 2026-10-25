#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawn, spawnSync} from 'node:child_process';
import path from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';

if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted' || process.platform !== 'linux') {
  throw new Error('Message UI fixture is restricted to disposable Linux hosted runners');
}
const binary = path.resolve('src-core/target/debug/aegis-core');
const child = spawn(binary, ['--message-dialog-helper'], {stdio: ['pipe', 'pipe', 'pipe']});
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
    const search = spawnSync('xdotool', ['search', '--pid', String(child.pid), '--name', '^Aegis Message Fixture$'], {encoding: 'utf8', timeout: 2000});
    if (search.status === 0 && /^\d+/.test(search.stdout)) {
      windowId = search.stdout.trim().split('\n')[0];
      break;
    }
    await delay(100);
  }
  assert.ok(windowId, `Owned message window did not appear: ${stderr}`);
  assert.equal(spawnSync('xdotool', ['windowfocus', '--sync', windowId], {timeout: 2000}).status, 0);
  assert.equal(spawnSync('xdotool', ['key', 'Return'], {timeout: 2000}).status, 0);
  const code = await Promise.race([done, delay(10000).then(() => {throw new Error('Owned dialog did not acknowledge');})]);
  assert.equal(code, 0, stderr);
  assert.equal(stdout, '', 'Successful zenity acknowledgement has empty stdout');
  console.log('Actual Linux message helper window opened and acknowledged in disposable X11 display');
} finally {
  if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  await done.catch(() => {});
}
