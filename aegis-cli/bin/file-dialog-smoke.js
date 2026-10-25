#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import path from 'node:path';

if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted') {
  throw new Error('File picker helper smoke is restricted to disposable hosted runners');
}
const binary = path.resolve(`src-core/target/debug/aegis-core${process.platform === 'win32' ? '.exe' : ''}`);
const fixtures = [
  {flag: '--file-picker-helper', inputs: [JSON.stringify({title: 'invalid\0title'}), 'not json', 'x'.repeat(8193)], error: /Invalid file picker|exceeds limits/},
  {flag: '--message-dialog-helper', inputs: [JSON.stringify({title: 'invalid\0title', message: 'fixture', kind: 'info'}), JSON.stringify({title:'fixture', message:'x'.repeat(16385), kind:'info'}), 'not json', 'x'.repeat(131073)], error: /Invalid message dialog|exceeds limits/},
];
for (const fixture of fixtures) {
  for (const input of fixture.inputs) {
    const result = spawnSync(binary, [fixture.flag], {
      input, encoding: 'utf8', timeout: 5000, windowsHide: true, maxBuffer: 65536,
    });
    assert.equal(result.error, undefined, 'Helper must reject invalid input before native UI work');
    assert.notEqual(result.status, 0);
    assert.equal(result.stdout, '', 'Helper output must contain no bootstrap logs or rejected data');
    assert.match(result.stderr, fixture.error);
  }
}
console.log('Feature-enabled file and message helpers reject malformed input before native UI work');
