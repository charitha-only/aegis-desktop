#!/usr/bin/env node
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtemp,mkdir,cp,readFile,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import http from 'node:http';
import {startDevelopment} from '../lib/development.js';

const fixtureRoot=await mkdtemp(path.join(tmpdir(),'aegis-dev-'));
const fixture=path.join(fixtureRoot,'long-development-project');

const coreBinary=path.resolve(`src-core/target/release/aegis-core${process.platform==='win32'?'.exe':''}`);
let session, blocker;
async function configure(port) {
  const config=JSON.parse(await readFile('aegis.config.json','utf8'));
  config.security.allowedOrigins=[`http://127.0.0.1:${port}`];
  await writeFile(path.join(fixture,'aegis.config.json'),JSON.stringify(config));
}
function listen(server) {return new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',()=>resolve(server.address().port));});}
async function freePort() {const probe=http.createServer();const port=await listen(probe);await new Promise(resolve=>probe.close(resolve));return port;}
const stopped=async origin=>assert.rejects(fetch(`${origin}/handshake`,{signal:AbortSignal.timeout(1500)}));
try {
  await mkdir(path.join(fixture,'src-ui/src'),{recursive:true});
  await writeFile(path.join(fixture,'src-ui/package.json'),'{"type":"module"}');
  await writeFile(path.join(fixture,'src-ui/index.html'),'<!doctype html><title>Owned dev fixture</title><script type="module" src="/src/aegis-bridge.ts"></script>');
  for(const name of ['aegis-bridge.ts','bridge-origin.mjs']) await cp(path.join('src-ui/src',name),path.join(fixture,'src-ui/src',name));
  await writeFile(path.join(fixture,'src-ui/.env.local'),'VITE_AEGIS_BRIDGE_URL=file:///TEST_ONLY_stale-override');
  let alias=fixture;
  if(process.platform==='win32') {
    const result=spawnSync('cmd.exe',['/d','/c',`for %I in ("${fixture}") do @echo %~sI`],{encoding:'utf8',windowsHide:true,windowsVerbatimArguments:true});
    assert.equal(result.status,0,result.stderr);
    alias=result.stdout.trim();
  }
  const port=await freePort();await configure(port);
  session=await startDevelopment({projectRoot:alias,coreBinary,uiPort:port,openBrowser:false,log:()=>{}});
  assert.equal((await fetch(session.uiOrigin)).status,200);
  const module=await fetch(`${session.uiOrigin}/src/aegis-bridge.ts`).then(r=>r.text());
  assert.ok(module.includes(session.coreOrigin),'Vite must inject the actual owned core origin');
  assert.equal((await fetch(`${session.coreOrigin}/handshake`,{headers:{Origin:session.uiOrigin}})).status,200);
  await session.restartUi();
  assert.equal((await fetch(`${session.coreOrigin}/handshake`)).status,200,'UI restart must preserve its owned core');
  assert.equal((await fetch(session.uiOrigin)).status,200,'UI restart must return to readiness');
  const ownedCore=session.coreOrigin, ownedUi=session.uiOrigin;
  await session.close();await session.finished;session=undefined;
  await stopped(ownedCore);await assert.rejects(fetch(ownedUi));
  console.log('PASS: automatic dynamic core selection, UI readiness, stale-override replacement and owned shutdown');

  const nextPort=await freePort();await configure(nextPort);
  session=await startDevelopment({projectRoot:fixture,coreBinary,uiPort:nextPort,openBrowser:false,log:()=>{}});
  const ui=session.uiOrigin;
  process.kill(session.corePid);
  await assert.rejects(session.finished,/Development core exited/);
  session=undefined;
  await assert.rejects(fetch(ui));
  console.log('PASS: unexpected core exit closes the development UI');

  const cancelPort=await freePort();await configure(cancelPort);
  const controller=new AbortController();
  session=await startDevelopment({projectRoot:fixture,coreBinary,uiPort:cancelPort,openBrowser:false,signal:controller.signal,log:()=>{}});
  const cancelledCore=session.coreOrigin,cancelledUi=session.uiOrigin;
  controller.abort();await session.finished;session=undefined;
  await stopped(cancelledCore);await assert.rejects(fetch(cancelledUi));
  console.log('PASS: cancellation stops the owned UI and core');

  if(process.env.GITHUB_ACTIONS==='true' && process.env.RUNNER_ENVIRONMENT==='github-hosted') {
    await mkdir(path.join(fixture,'src-core'),{recursive:true});
    for(const name of ['Cargo.toml','Cargo.lock','src','vendor']) await cp(path.join('src-core',name),path.join(fixture,'src-core',name),{recursive:true});
    const buildPort=await freePort();await configure(buildPort);
    session=await startDevelopment({projectRoot:fixture,uiPort:buildPort,openBrowser:false,log:()=>{}});
    assert.equal((await fetch(`${session.coreOrigin}/handshake`)).status,200);
    await session.close();await session.finished;session=undefined;
    console.log('PASS: default Cargo build discovers the executable from build artifacts');
  }

  blocker=http.createServer((_,response)=>response.end('TEST_ONLY_other_server'));
  const blockedPort=await listen(blocker);await configure(blockedPort);
  let output='';
  await assert.rejects(startDevelopment({projectRoot:fixture,coreBinary,uiPort:blockedPort,openBrowser:false,log:text=>{output=(output+text).slice(-16384);}}),/already in use/);
  assert.equal(await fetch(`http://127.0.0.1:${blockedPort}`).then(r=>r.text()),'TEST_ONLY_other_server');
  const match=output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
  assert.ok(match);await stopped(match[1]);
  console.log('PASS: occupied UI port preserves the existing server and cleans up the new core');
} finally {
  if(session) {await session.close();await session.finished.catch(()=>{});}
  if(blocker) await new Promise(resolve=>blocker.close(resolve));
  const relative=path.relative(tmpdir(),fixtureRoot);
  if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(fixtureRoot,{recursive:true,force:true,maxRetries:10,retryDelay:100});
}
