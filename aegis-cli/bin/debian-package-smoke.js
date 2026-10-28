#!/usr/bin/env node
import {mkdtemp,mkdir,cp,readFile,writeFile,rm,access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {randomBytes} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {withBenchmarkCore} from '../lib/benchmark-core.js';
if(process.platform!=='linux'||process.env.GITHUB_ACTIONS!=='true'||process.env.RUNNER_ENVIRONMENT!=='github-hosted') throw new Error('Debian system-install acceptance requires a disposable GitHub-hosted Linux runner');
const repo=process.cwd(),fixture=await mkdtemp(path.join(tmpdir(),'aegis-deb-acceptance-'));
const cli=path.join(repo,'aegis-cli/bin/aegis.js');
const run=(program,args)=>execFileSync(program,args,{encoding:'utf8',timeout:60000,maxBuffer:1024*1024});
const invoke=args=>JSON.parse(run(process.execPath,[cli,...args]));
let packageName,installed=false;
try {
  for(const key of ['HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME']) {process.env[key]=path.join(fixture,key);await mkdir(process.env[key]);}
  const appId='org.aegis.ci.deb.'+randomBytes(12).toString('hex');
  const artifacts=[];
  for(const version of ['1.0.0','2.0.0']) {
    const bundle=path.join(fixture,version);await mkdir(bundle);
    await cp(path.join(repo,'src-core/target/release/aegis-core'),path.join(bundle,'aegis-core'));
    await cp(path.join(repo,'src-ui/dist'),path.join(bundle,'ui'),{recursive:true});
    const config=JSON.parse(await readFile(path.join(repo,'aegis.config.json'),'utf8'));
    Object.assign(config,{appId,version,name:version==='1.0.0'?'Installer One':'Installer Renamed'});
    await writeFile(path.join(bundle,'aegis.config.json'),JSON.stringify(config));
    const pin=invoke(['package-manifest',bundle]).manifestSha256;
    artifacts.push(invoke(['package-deb',bundle,path.join(fixture,version+'.deb'),pin,'Aegis CI <ci@example.invalid>']));
  }
  packageName=artifacts[0].packageName;
  assert.equal(packageName,artifacts[1].packageName);
  assert.match(packageName,/^aegis-[a-f0-9]{32}$/);
  // Never replace a package that predates this randomly named fixture.
  let existing=false;
  try {run('dpkg-query',['-W',packageName]);existing=true;} catch(error) {if(error.status!==1) throw error;}
  assert.equal(existing,false,'Fixture package already exists');
  const binary=artifacts[0].installRoot+'/aegis-core';
  let expected;
  async function check(initialize=false) {
    await withBenchmarkCore({binary},async({base})=>{
      const {token}=await (await fetch(base+'/handshake',{signal:AbortSignal.timeout(5000)})).json();let seq=0;
      const call=async(command,payload)=>{
        const response=await fetch(base+'/ipc',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({id:String(++seq),command,payload,nonce:token}),signal:AbortSignal.timeout(15000)});
        const result=await response.json();assert.equal(result.success,true,command);return result.data;
      };
      const credentials={filename:'installer.vault',passphrase:'TEST_ONLY_debian_password'};
      if(initialize) {
        await call('vault_encrypt_store',{...credentials,content:'first'});
        await call('vault_encrypt_store',{...credentials,content:'second'});
      }
      const current=await call('fs_read_file',{path:'installer.vault'});
      const backup=await call('fs_read_file',{path:'installer.vault.bak'});
      assert.match(current.content,/AEGIS_VAULT_V2/);assert.match(backup.content,/AEGIS_VAULT_V2/);
      if(initialize) expected={current:current.content,backup:backup.content};
      else assert.deepEqual({current:current.content,backup:backup.content},expected);
      assert.equal((await call('vault_decrypt_retrieve',credentials)).decryptedContent,'second');
    });
  }
  // Mark ownership before dpkg: a failed install may leave a partially installed fixture.
  installed=true;run('sudo',['dpkg','--install',artifacts[0].outputFile]);await check(true);
  run('sudo',['dpkg','--install',artifacts[1].outputFile]);await check();
  run('sudo',['dpkg','--remove',packageName]);await assert.rejects(access(binary));
  run('sudo',['dpkg','--install',artifacts[1].outputFile]);await check();
  console.log('PASS: native Debian install, renamed upgrade, removal and reinstall preserve authenticated vault and backup bytes');
} finally {
  if(installed) run('sudo',['dpkg','--purge',packageName]);
  const relative=path.relative(tmpdir(),fixture);
  if(relative&&!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(fixture,{recursive:true,force:true});
}
