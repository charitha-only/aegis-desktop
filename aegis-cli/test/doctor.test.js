import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {mkdtemp,readdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {inspectProject} from '../lib/doctor.js';
const cli=fileURLToPath(new URL('../bin/aegis.js',import.meta.url));
test('doctor reports project prerequisites as JSON without starting application services',()=>{
  const result=spawnSync(process.execPath,[cli,'doctor','--json'],{encoding:'utf8',windowsHide:true,timeout:30000});
  const report=JSON.parse(result.stdout);
  assert.equal(report.schemaVersion,1);
  assert.equal(report.platform,process.platform);
  for(const id of ['node','cargo','npm','configuration','rust-lockfile','ui-lockfile','ui-dependencies']) {
    assert.equal(typeof report.checks.find(check=>check.id===id)?.ok,'boolean',id);
  }
  assert.equal(result.status,report.ok?0:1);
  assert.equal(report.securityCertification,false);
});
test('doctor identifies missing setup without creating project files',async()=>{
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-doctor-'));
  try {
    const report=await inspectProject(fixture);
    assert.equal(report.ok,false);
    for(const id of ['configuration','rust-lockfile','ui-lockfile','ui-dependencies']) assert.equal(report.checks.find(check=>check.id===id).ok,false);
    assert.deepEqual(await readdir(fixture),[]);
  } finally {await rm(fixture,{recursive:true,force:true});}
});
