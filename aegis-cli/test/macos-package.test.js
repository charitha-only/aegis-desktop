import test from 'node:test';
import assert from 'node:assert/strict';
import {macosPackagePlan} from '../lib/macos-package.js';
const config={appId:'org.example.app',name:'Example & App',version:'1.2.3'};
test('macOS package identity remains stable across display-name and version changes',()=>{
  const first=macosPackagePlan(config,{architecture:'arm64'});
  const next=macosPackagePlan({...config,name:'Renamed',version:'2.0.0'},{architecture:'arm64'});
  assert.equal(first.identifier,next.identifier);
  assert.equal(first.installRoot,next.installRoot);
  assert.match(first.installRoot,/^\/Applications\/Aegis-[a-f0-9]{32}\.app$/);
  assert.match(first.infoPlist,/<string>Example &amp; App<\/string>/);
  assert.match(first.infoPlist,/<key>CFBundleExecutable<\/key><string>launch<\/string>/);
});
test('macOS plans reject XML/control injection, unsafe identities, unsupported CPU and nonnumeric versions',()=>{
  for(const bad of [{...config,appId:'../escape'},{...config,name:'App\nInjected'},{...config,version:'1.0<script>'},{...config,version:'1.0.0-beta'},{...config,version:'1.2.3.4'},{...config,name:null}]) assert.throws(()=>macosPackagePlan(bad,{architecture:'arm64'}));
  assert.throws(()=>macosPackagePlan(config,{architecture:'ia32'}));
});
import {spawnSync} from 'node:child_process';
import path from 'node:path';
test('macOS packaging CLI rejects unsupported hosts before opening input',{skip:process.platform==='darwin'},()=>{
 const result=spawnSync(process.execPath,[path.resolve('aegis-cli/bin/aegis.js'),'package-macos','missing','app.pkg','a'.repeat(64)],{encoding:'utf8'});
 assert.equal(result.status,1);assert.match(result.stderr,/native macOS/);
});
