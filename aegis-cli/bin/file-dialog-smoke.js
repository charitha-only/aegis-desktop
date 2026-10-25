#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import path from 'node:path';

if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted') {
  throw new Error('File picker helper smoke is restricted to disposable hosted runners');
}
const binary = path.resolve(`src-core/target/debug/aegis-core${process.platform === 'win32' ? '.exe' : ''}`);
for (const input of [JSON.stringify({title: 'invalid\0title'}), 'not json', 'x'.repeat(8193)]) {
  const result = spawnSync(binary, ['--file-picker-helper'], {
    input, encoding: 'utf8', timeout: 5000, windowsHide: true, maxBuffer: 65536,
  });
  assert.equal(result.error, undefined, 'Helper must reject invalid input without opening a picker');
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, '', 'Helper output must contain no bootstrap logs or rejected data');
  assert.match(result.stderr, /Invalid file picker|exceeds limits/);
}
console.log('Feature-enabled file picker helper rejects malformed input before native UI work');
