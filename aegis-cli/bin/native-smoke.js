#!/usr/bin/env node
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
if (process.env.GITHUB_ACTIONS !== 'true' || process.env.RUNNER_ENVIRONMENT !== 'github-hosted') {
  throw new Error('Native GUI smoke is restricted to disposable GitHub-hosted runners');
}
const args = process.argv.slice(2);
if (args.length > 1 || (args[0] && !/^--repeat=(1|3)$/.test(args[0]))) throw new Error('Usage: native-smoke.js [--repeat=1|3]');
const repetitions = args[0] === '--repeat=3' ? 3 : 1;
for (let iteration = 1; iteration <= repetitions; iteration++) {
console.log(`NATIVE_SMOKE_RUN ${iteration}/${repetitions}`);
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
    const call=async(command,payload={})=>{await new Promise(resolve=>setTimeout(resolve,70));return fetch('/ipc',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({id:String(++seq),command,payload,nonce:token})}).then(r=>r.json());};
    const check=(ok,message)=>{if(!ok)throw Error(message);};
    const first=await call('managed_window_create',{url:location.origin+'/child.html',title:'Owned child 1',resizable:false,decorations:false});
    check(first.success,'first create: '+first.error);
    const second=await call('managed_window_create',{url:location.origin+'/child.html',title:'Owned child 2'});
    check(second.success,'second create: '+second.error);
    const focused=await call('managed_window_focus',{windowId:first.data.windowId});
    check(focused.success&&focused.data.windowId===first.data.windowId&&focused.data.action==='focus'&&focused.data.requested===true&&focused.data.stateConfirmed===false,'focus request receipt');
    for(const minimized of [true,false]){
      const changed=await call('managed_window_set_minimized',{windowId:second.data.windowId,minimized});
      check(changed.success&&changed.data.windowId===second.data.windowId&&changed.data.action==='setMinimized'&&changed.data.requested===true&&changed.data.requestedMinimized===minimized&&changed.data.stateConfirmed===false,'minimize/restore request receipt');
    }
    check(!(await call('managed_window_focus',{windowId:'child-99999'})).success,'unknown focus target');
    for(const payload of [{windowId:second.data.windowId,minimized:'false'},{windowId:second.data.windowId,minimized:false,extra:true}])check(!(await call('managed_window_set_minimized',payload)).success,'malformed minimize denied');

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
    const live=[];
    for(let i=0;i<8;i++){
      const created=await call('managed_window_create',{url:location.origin+'/child.html',title:'Limit child '+i});
      check(created.success,'create within limit: '+created.error);live.push(created.data.windowId);
    }
    const overflow=await call('managed_window_create',{url:location.origin+'/child.html'});
    check(!overflow.success&&overflow.error.includes('limit'),'ninth child rejected');
    const full=await call('managed_window_list');
    check(full.success&&full.data.windowIds.length===8&&live.every(id=>full.data.windowIds.includes(id)),'limit rejection preserves handles');
    const retired=live.pop();check((await call('managed_window_close',{windowId:retired})).success,'free one slot');
    const replacement=await call('managed_window_create',{url:location.origin+'/child.html'});
    check(replacement.success&&!live.includes(replacement.data.windowId)&&replacement.data.windowId!==retired,'freed slot reusable with fresh ID');
    live.push(replacement.data.windowId);
    for(const id of live.slice(1))check((await call('managed_window_close',{windowId:id})).success,'close surplus child');
    const remaining=await call('managed_window_list');
    check(remaining.success&&remaining.data.windowIds.length===1&&remaining.data.windowIds[0]===live[0],'one live child retained for primary shutdown');
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
  const timer = setTimeout(() => child.kill(), 60000);
  const code = await new Promise((resolve, reject) => {child.once('error', reject); child.once('exit', resolve);});
  clearTimeout(timer);
  if (code !== 0 || (!output.includes('AEGIS_NATIVE_SMOKE_READY') || !output.includes('AEGIS_NATIVE_SMOKE_CHILDREN_DROPPED=1'))) throw new Error(`Native GUI fixture failed: ${output}`);
  for (const line of output.split(/\r?\n/)) if (line.startsWith('AEGIS_NATIVE_SMOKE_STAGE ')) console.log(line);
  console.log('PASS: native dynamic-port primary, managed child lifecycle and focus/minimize request receipts, eight-child limit, capacity recovery and live-child shutdown');
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true, maxRetries: 10, retryDelay: 100});
}

}
console.log(`PASS: all ${repetitions} fresh-fixture native runs passed`);
