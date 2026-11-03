#!/usr/bin/env node
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted') {
  throw new Error('Native GUI smoke is restricted to disposable GitHub-hosted runners');
}
const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-native-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
try {
  const app = path.join(fixture, 'NativeFixture');
  await mkdir(path.join(app, 'ui'), {recursive: true});
  await cp(path.resolve('src-core/target/debug', executable), path.join(app, executable));
  const config = JSON.parse(await readFile('aegis.config.json', 'utf8'));
  config.window.width = 800; config.window.height = 600;
  config.security.capabilities.system.allowManagedWindows = true;
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  await writeFile(path.join(app, 'ui/child.html'), '<!doctype html><title>Owned child fixture</title><h1>Managed child</h1>');
  await writeFile(path.join(app, 'ui/managed-fixture.js'), `
  (async()=>{
    const {token}=await fetch('/handshake').then(r=>r.json());let seq=0;
    const call=async(command,payload={})=>fetch('/ipc',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({id:String(++seq),command,payload,nonce:token})}).then(r=>r.json());
    const check=(ok,message)=>{if(!ok)throw Error(message);};
    const first=await call('managed_window_create',{url:location.origin+'/child.html',title:'Owned child 1',resizable:false,decorations:false});
    check(first.success,'first create: '+first.error);
    const second=await call('managed_window_create',{url:location.origin+'/child.html',title:'Owned child 2'});
    check(second.success,'second create: '+second.error);
    check(first.data.windowId!==second.data.windowId,'unique IDs');
    const listed=await call('managed_window_list');
    check(listed.success&&listed.data.windowIds.length===2&&listed.data.windowIds.includes(first.data.windowId)&&listed.data.windowIds.includes(second.data.windowId),'list ownership');
    const denied=await call('managed_window_create',{url:'http://127.0.0.1:1/'});check(!denied.success,'cross origin rejection');
    check(!(await call('managed_window_close',{windowId:'child-99999'})).success,'unknown close');
    check((await call('managed_window_close',{windowId:first.data.windowId})).success,'close first');
    check((await call('managed_window_close',{windowId:second.data.windowId})).success,'close second');
    check((await call('managed_window_list')).data.windowIds.length===0,'empty after close');
    const reopened=await call('managed_window_create',{url:location.origin+'/child.html'});check(reopened.success,'reopen');
    check(reopened.data.windowId!==first.data.windowId&&reopened.data.windowId!==second.data.windowId,'no handle reuse');
    check((await call('managed_window_close',{windowId:reopened.data.windowId})).success,'close reopened');
    document.title='Aegis Native Smoke Fixture';
  })().catch(error=>{document.title='MANAGED_WINDOW_FIXTURE_FAILED: '+error.message;});
`);
  await writeFile(path.join(app, 'ui/index.html'), '<!doctype html><title>Managed window verification pending</title><h1>Native fixture</h1><script src="/managed-fixture.js"></script>');
  const child = spawn(path.join(app, executable), ['--port=0', '--native-window'], {cwd: app, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
    ...process.env, AEGIS_NATIVE_SMOKE: '1', HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
    XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache'),
  }});
  let output = '';
  for (const stream of [child.stdout, child.stderr]) stream.on('data', data => {output = (output + data.toString()).slice(-16000);});
  const timer = setTimeout(() => child.kill(), 30000);
  const code = await new Promise((resolve, reject) => {child.once('error', reject); child.once('exit', resolve);});
  clearTimeout(timer);
  if (code !== 0 || !output.includes('AEGIS_NATIVE_SMOKE_READY')) throw new Error(`Native GUI fixture failed: ${output}`);
  console.log('PASS: native dynamic-port primary, managed child create/list/close/reopen, origin rejection and shutdown');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true, maxRetries: 10, retryDelay: 100});
}
