import {spawn} from 'node:child_process';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {resolveBridgeOrigin} from '../../src-ui/src/bridge-origin.mjs';

async function bounded(promise,ms,message) {
  let timer;
  try {return await Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error(message)),ms);})]);}
  finally {clearTimeout(timer);}
}
async function stop(child,closed) {
  if(child.pid && child.exitCode===null && child.signalCode===null) child.kill();
  try {await bounded(closed,3000,'Owned benchmark core did not close');}
  catch(error) {
    if(!child.pid || child.exitCode!==null || child.signalCode!==null) throw error;
    child.kill('SIGKILL');
    await bounded(closed,3000,'Cannot stop owned benchmark core');
  }
}

/** Run a measurement with one owned headless core; close it before returning. */
export async function withBenchmarkCore({binary,args=['--port=0'],startupTimeoutMs=15000},run) {
  if(typeof binary!=='string' || !binary || !Array.isArray(args) || args.some(arg=>typeof arg!=='string')
    || !Number.isInteger(startupTimeoutMs) || startupTimeoutMs<1 || startupTimeoutMs>60000 || typeof run!=='function') {
    throw new Error('Invalid benchmark core settings');
  }
  const fixture=await mkdtemp(path.join(tmpdir(),'aegis-benchmark-'));
  const started=performance.now();
  const child=spawn(binary,args,{cwd:fixture,windowsHide:true,stdio:['ignore','pipe','pipe']});
  const closed=new Promise(resolve=>child.once('close',resolve));
  child.stdout.setEncoding('utf8');
  child.stderr.resume();
  try {
    const ready=new Promise((resolve,reject)=>{
      let output='';
      child.once('error',reject);
      child.once('exit',()=>reject(new Error('Core exited before ready')));
      child.stdout.on('data',data=>{
        output=(output+data).slice(-16384);
        // A chunk can end halfway through the numeric port. Require its newline.
        const match=output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)\r?\n/);
        if(match) resolve(match[1]);
      });
    });
    const base=resolveBridgeOrigin({production:false,configuredOrigin:await bounded(ready,startupTimeoutMs,'Core startup timed out')});
    return await run({base,pid:child.pid,startupMs:performance.now()-started});
  } finally {
    await stop(child,closed);
    const relative=path.relative(tmpdir(),fixture);
    if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(fixture,{recursive:true,force:true,maxRetries:10,retryDelay:100});
  }
}
