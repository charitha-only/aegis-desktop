import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {developmentPlan,parseDevelopmentArgs} from '../lib/development.js';

test('development reserves its own dynamic core and defers desktop opening to UI readiness', () => {
  const plan = developmentPlan({openBrowser:false});
  assert.deepEqual(plan.coreArgs, ['--port=0']);
  assert.equal(plan.uiOrigin, 'http://127.0.0.1:5173');
  assert.equal(plan.openBrowser, false);
});

test('development options reject malformed or ambiguous settings before launch', () => {
  const parsed=parseDevelopmentArgs(['--no-open','--ui-port=55123','--print-plan']);
  assert.equal(developmentPlan(parsed).uiOrigin,'http://127.0.0.1:55123');
  assert.equal(parsed.printPlan,true);
  for (const args of [['--ui-port=0'],['--ui-port=65536'],['--ui-port=5173','--ui-port=5174'],['--unknown'],['--core-binary=']]) {
    assert.throws(()=>parseDevelopmentArgs(args));
  }
});

test('CLI development inspection does not compile or launch any desktop', () => {
  const cli=fileURLToPath(new URL('../bin/aegis.js',import.meta.url));
  const result=spawnSync(process.execPath,[cli,'dev','--no-open','--print-plan','--ui-port=55123'],{encoding:'utf8',windowsHide:true,timeout:10000});
  assert.equal(result.status,0,result.stderr);
  const plan=JSON.parse(result.stdout);
  assert.deepEqual(plan.coreArgs,['--port=0']);
  assert.equal(plan.uiOrigin,'http://127.0.0.1:55123');
  assert.equal(plan.openBrowser,false);
});

test('UI origin follows browser normalization for the default HTTP port', () => {
  assert.equal(developmentPlan({uiPort:80}).uiOrigin,'http://127.0.0.1');
});
