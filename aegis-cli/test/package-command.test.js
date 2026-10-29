import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const cli=fileURLToPath(new URL('../bin/aegis.js',import.meta.url));
test('package exposes a side-effect-free plan with locked builds and explicit integrity scope',()=>{
  const run=spawnSync(process.execPath,[cli,'package','--print-plan'],{encoding:'utf8',windowsHide:true});
  assert.equal(run.status,0,run.stderr);
  const plan=JSON.parse(run.stdout);
  assert.deepEqual(plan.steps,['check-prerequisites','build-locked-core','build-ui','bundle','write-integrity','verify-integrity']);
  assert.equal(plan.publisherAuthenticated,false);
  assert.equal(plan.overwrite,false);
  assert.match(plan.bundleDirectory,/dist-bundle/);
});
