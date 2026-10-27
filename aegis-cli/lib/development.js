import {spawn, spawnSync} from 'node:child_process';
import {access, stat, realpath} from 'node:fs/promises';
import path from 'node:path';
import {resolveBridgeOrigin} from '../../src-ui/src/bridge-origin.mjs';

/** Inspect development launch settings without compiling, binding or launching. */
export function developmentPlan({uiPort = 5173, openBrowser = true} = {}) {
  if (!Number.isInteger(uiPort) || uiPort < 1 || uiPort > 65535 || typeof openBrowser !== 'boolean') {
    throw new Error('Invalid development UI port or browser setting');
  }
  return {coreArgs: ['--port=0'], uiOrigin: new URL(`http://127.0.0.1:${uiPort}`).origin, openBrowser};
}
export function parseDevelopmentArgs(args) {
  const options = {uiPort:5173, openBrowser:true, printPlan:false};
  const seen = new Set();
  for (const arg of args) {
    const key = arg.split('=')[0];
    if (seen.has(key)) throw new Error('Duplicate development option');
    seen.add(key);
    if (arg === '--no-open') options.openBrowser = false;
    else if (arg === '--print-plan') options.printPlan = true;
    else if (/^--ui-port=[1-9][0-9]{0,4}$/.test(arg)) options.uiPort = Number(arg.slice(10));
    else if (arg.startsWith('--core-binary=') && arg.length > 14) options.coreBinary = path.resolve(arg.slice(14));
    else throw new Error('Unknown or malformed development option');
  }
  developmentPlan(options);
  return options;
}
function completion(child) {
  return new Promise(resolve => child.once('close', resolve));
}
async function bounded(promise, milliseconds, message) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => {timer=setTimeout(() => reject(new Error(message)), milliseconds);})]); }
  finally {clearTimeout(timer);}
}
async function stopOwned(child, closed) {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) {
    if (closed) await bounded(closed, 5000, 'Owned process did not close');
    return;
  }
  if (process.platform === 'win32') {
    spawnSync('taskkill', ['/pid', String(child.pid), '/t', '/f'], {windowsHide:true, stdio:'ignore', timeout:5000});
    if (child.exitCode === null && child.signalCode === null) child.kill();
  } else {
    try {process.kill(-child.pid, 'SIGTERM');} catch {child.kill();}
  }
  try {await bounded(closed, 3000, 'Owned process shutdown timed out');}
  catch {
    if (child.exitCode === null && child.signalCode === null) {
      if (process.platform === 'win32') child.kill('SIGKILL');
      else {try {process.kill(-child.pid, 'SIGKILL');} catch {child.kill('SIGKILL');}}
    }
    await bounded(closed, 3000, 'Cannot stop owned development process');
  }
}
async function buildCore(projectRoot, signal, log) {
  const candidate = path.join(process.env.USERPROFILE || process.env.HOME || '', '.cargo/bin', process.platform === 'win32' ? 'cargo.exe' : 'cargo');
  let cargo = 'cargo';
  try {await access(candidate); cargo=candidate;} catch {}
  const child = spawn(cargo, ['build','--locked','--manifest-path','src-core/Cargo.toml','--message-format=json-render-diagnostics'],
    {cwd:projectRoot, windowsHide:true, detached:process.platform !== 'win32', stdio:['ignore','pipe','pipe']});
  const closed=completion(child);
  child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');
  let pending='', artifact;
  const record=line=>{
    if(!line) return;
    try {
      const item=JSON.parse(line);
      if(item.reason==='compiler-artifact' && item.target?.name==='aegis-core' && item.target.kind.includes('bin') && item.executable) artifact=item.executable;
      if(item.reason==='compiler-message' && item.message?.rendered) log(item.message.rendered);
    } catch {log(line.slice(0,8192)+'\n');}
  };
  child.stdout.on('data',data=>{
    pending+=data.toString();
    let end;
    while((end=pending.indexOf('\n'))!==-1){record(pending.slice(0,end));pending=pending.slice(end+1);}
    if(pending.length>1024*1024){pending='';void stopOwned(child,closed).catch(()=>{});}
  });
  child.stderr.on('data', data => log(data.toString()));
  const result=new Promise((resolve,reject) => {child.once('error',reject); child.once('exit',code => code===0?resolve():reject(new Error('Development core build failed')));});
  result.catch(() => {});
  const abort=() => {void stopOwned(child,closed).catch(() => {});};
  signal?.addEventListener('abort',abort,{once:true});
  try {if(signal?.aborted) abort(); await result;}
  catch(error) {if(signal?.aborted) throw new Error('Development startup cancelled'); throw error;}
  finally {signal?.removeEventListener('abort',abort); await stopOwned(child,closed);}
  record(pending);
  if(!artifact) throw new Error('Cargo did not report an aegis-core executable');
  return artifact;
}

/** Start one owned core and UI. Return only after both endpoints are ready. */
export async function startDevelopment({projectRoot, coreBinary, uiPort=5173, openBrowser=true, signal,
  log=text => process.stdout.write(text)} = {}) {
  const plan=developmentPlan({uiPort,openBrowser});
  projectRoot=await realpath(path.resolve(projectRoot));
  const checkCancelled=() => {if(signal?.aborted) throw new Error('Development startup cancelled');};
  checkCancelled();
  const {createServer}=await import(new URL('../../src-ui/node_modules/vite/dist/node/index.js',import.meta.url).href);
  if (!coreBinary) coreBinary=await buildCore(projectRoot,signal,log);
  checkCancelled();
  coreBinary=path.resolve(coreBinary);
  if (!(await stat(coreBinary)).isFile()) throw new Error('Development core must be a regular executable file');
  try {await access(path.join(path.dirname(coreBinary),'ui/index.html')); throw new Error('Use an unpackaged development core binary');}
  catch(error) {if(error.code !== 'ENOENT') throw error;}
  let vite, core, closed, running=false, stopping=false, fatal, stopPromise, uiMonitor;
  let resolveFinished,rejectFinished;
  const finished=new Promise((resolve,reject) => {resolveFinished=resolve;rejectFinished=reject;});
  finished.catch(() => {});
  const stop=error => {
    if(stopPromise) return stopPromise;
    stopping=true;
    clearInterval(uiMonitor);
    stopPromise=(async()=>{
      signal?.removeEventListener('abort',abort);
      const results=await Promise.allSettled([vite?.close(),stopOwned(core,closed)]);
      const failure=results.find(result=>result.status==='rejected');
      if(error || failure) rejectFinished(error || failure.reason); else resolveFinished();
      if(failure) throw failure.reason;
    })();
    return stopPromise;
  };
  const abort=()=>{if(running) void stop().catch(()=>{});};
  signal?.addEventListener('abort',abort,{once:true});
  const check=()=>{checkCancelled(); if(fatal) throw fatal;};
  try {
    check();
    core=spawn(coreBinary,plan.coreArgs,{cwd:projectRoot,windowsHide:true,detached:process.platform!=='win32',stdio:['ignore','pipe','pipe']});
    closed=completion(core);
    core.stdout.setEncoding('utf8');core.stderr.setEncoding('utf8');
    core.once('exit',(code,exitSignal)=>{
      fatal=new Error(`Development core exited (${code ?? exitSignal})`);
      if(running && !stopping) void stop(fatal).catch(()=>{});
    });
    core.stderr.on('data',data=>log(data.toString()));
    const ready=new Promise((resolve,reject)=>{
      let output='';
      const cancel=()=>reject(new Error('Development startup cancelled'));
      signal?.addEventListener('abort',cancel,{once:true});
      const remove=()=>signal?.removeEventListener('abort',cancel);
      core.once('error',error=>{remove();reject(error);});
      core.once('exit',()=>{remove();reject(fatal || new Error('Core exited before ready'));});
      core.stdout.on('data',data=>{
        log(data.toString());
        output=(output+data.toString()).slice(-16384);
        const match=output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
        if(match){remove();resolve(match[1]);}
      });
    });
    const coreOrigin=resolveBridgeOrigin({production:false,configuredOrigin:await bounded(ready,30000,'Development core startup timed out')});
    check();
    const probe=await fetch(`${coreOrigin}/handshake`,{headers:{Origin:plan.uiOrigin},redirect:'error',signal:AbortSignal.timeout(5000)});
    if(!probe.ok) throw new Error('Core rejected the development UI origin; check allowedOrigins');
    await probe.arrayBuffer();
    check();
    vite=await createServer({root:path.join(projectRoot,'src-ui'),server:{host:'127.0.0.1',port:uiPort,strictPort:true,open:false},
      define:{'import.meta.env.VITE_AEGIS_BRIDGE_URL':JSON.stringify(coreOrigin)}});
    check();
    await vite.listen();
    check();
    const uiOrigin=new URL(vite.resolvedUrls.local[0]).origin;
    if(uiOrigin!==plan.uiOrigin) throw new Error('Unexpected development UI address');
    running=true;
    // Vite replaces its HTTP server during an ordinary configuration restart.
    // Observe the current owned server and allow a bounded restart interval.
    let lastHealthy=performance.now();
    uiMonitor=setInterval(()=>{
      if(vite.httpServer?.listening) lastHealthy=performance.now();
      else if(!stopping && performance.now()-lastHealthy>5000) void stop(new Error('Development UI server did not recover')).catch(()=>{});
    },100);
    uiMonitor.unref();
    if(openBrowser) vite.openBrowser();
    log(`[Aegis Dev] UI ready at ${uiOrigin}; core ${coreOrigin}\n`);
    return {coreOrigin,uiOrigin,corePid:core.pid,finished,restartUi:()=>vite.restart(),close:()=>stop()};
  } catch(error) {await stop(error); throw error;}
}
