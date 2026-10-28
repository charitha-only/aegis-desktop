import test from 'node:test';
import assert from 'node:assert/strict';
import {debianPackagePlan} from '../lib/debian-package.js';
const config={appId:'org.example.app',name:'Example App',version:'1.2.3',description:'Example desktop app'};
const options={architecture:'x64',maintainer:'Example Maintainer <maintainer@example.invalid>'};
test('Debian plans keep package/storage identity across display-name and version changes',()=>{
  const first=debianPackagePlan(config,options);
  const next=debianPackagePlan({...config,name:'Renamed App',version:'2.0.0'},options);
  assert.equal(first.architecture,'amd64');
  assert.equal(first.packageName,next.packageName);
  assert.equal(first.installRoot,next.installRoot);
  assert.match(first.installRoot,/^\/opt\/aegis-[a-f0-9]{32}$/);
  assert.ok(first.desktopEntry.includes('--spawn-window --port=0'));
  assert.equal(debianPackagePlan(config,{...options,architecture:'arm64'}).architecture,'arm64');
});

test('Debian metadata rejects injected fields, malformed identity and unsupported architectures',()=>{
  for(const bad of [{...config,appId:'../outside'},{...config,name:'App\nExec=evil'},{...config,version:'1.0\nDepends: bad'},{...config,description:null}]) {
    assert.throws(()=>debianPackagePlan(bad,options));
  }
  assert.throws(()=>debianPackagePlan(config,{...options,architecture:'ia32'}));
  assert.throws(()=>debianPackagePlan(config,{...options,maintainer:'Name <x@y>\nDepends: bad'}));
});
import {spawnSync} from 'node:child_process';
import path from 'node:path';
test('Debian CLI reports its native-host requirement before reading an input bundle',{skip:process.platform==='linux'},()=>{
  const result=spawnSync(process.execPath,[path.resolve('aegis-cli/bin/aegis.js'),'package-deb','missing','output.deb','a'.repeat(64),options.maintainer],{encoding:'utf8'});
  assert.equal(result.status,1);
  assert.match(result.stderr,/native Linux/);
});
test('Debian plans reject identities the cross-platform runtime cannot open',()=>{
 for(const appId of ['Example.','CON','nul.settings','COM1.app','lpt9']) assert.throws(()=>debianPackagePlan({...config,appId},options));
});
