#!/usr/bin/env node
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const fixture=await mkdtemp(path.join(tmpdir(),'aegis-budget-'));
try {
  const budget=path.join(fixture,'budget.json');
  await writeFile(budget,JSON.stringify({startupP95Ms:0}));
  const binary=path.resolve(`src-core/target/release/aegis-core${process.platform==='win32'?'.exe':''}`);
  const result=spawnSync(process.execPath,['aegis-cli/bin/benchmark.js',binary,`--budget=${budget}`],
    {encoding:'utf8',windowsHide:true,timeout:60000});
  assert.equal(result.status,1,result.stderr);
  const report=JSON.parse(result.stdout);
  assert.equal(report.schemaVersion,2);
  assert.equal(report.budget.passed,false);
  assert.equal(report.rawSamples.startupMs.length,5);
  assert.equal(report.rawSamples.ipcMs.length,20);
  assert.match(report.provenance.binarySha256,/^[a-f0-9]{64}$/);
  assert.equal(report.budget.checks[0].limit,0);
  console.log('PASS: real benchmark retains raw samples/provenance and fails an explicit exceeded budget');
} finally {
  const relative=path.relative(tmpdir(),fixture);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(fixture,{recursive:true,force:true});
}
