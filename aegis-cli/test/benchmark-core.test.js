import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {withBenchmarkCore} from '../lib/benchmark-core.js';

test('benchmark readiness waits for the complete port line and cleans up after callback failure', async () => {
  let pid;
  const script="process.on('SIGTERM',()=>{});process.stdout.write('IPC Bridge Server actively listening on http://127.0.0.1:443');setTimeout(()=>process.stdout.write('21\\n'),30);setInterval(()=>{},1000)";
  await assert.rejects(withBenchmarkCore({binary:process.execPath,args:['-e',script]}, async core=>{
    pid=core.pid;
    assert.equal(core.base,'http://127.0.0.1:44321');
    throw new Error('TEST_ONLY workload failure');
  }),/TEST_ONLY workload failure/);
  assert.throws(()=>process.kill(pid,0),{code:'ESRCH'});
});

test('startup timeout stops only the owned non-ready process', async () => {
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-lifecycle-test-'));
  try {
    const pidFile=path.join(fixture,'pid.txt');
    const script=`process.on('SIGTERM',()=>{});require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>{},1000)`;
    await assert.rejects(withBenchmarkCore({binary:process.execPath,args:['-e',script],startupTimeoutMs:1500},()=>assert.fail('must not measure an unready core')),/Core startup timed out/);
    const pid=Number(await readFile(pidFile,'utf8'));
    assert.ok(Number.isSafeInteger(pid)&&pid>0);
    assert.throws(()=>process.kill(pid,0),{code:'ESRCH'});
  } finally {
    const relative=path.relative(tmpdir(),fixture);
    if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(fixture,{recursive:true,force:true});
  }
});
