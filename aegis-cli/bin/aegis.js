#!/usr/bin/env node
import {spawn,spawnSync} from 'node:child_process';
import path from 'node:path';import fs from 'node:fs';import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const cargoPath=path.join(process.env.USERPROFILE||process.env.HOME||'','.cargo','bin',process.platform==='win32'?'cargo.exe':'cargo');
const cargo=fs.existsSync(cargoPath)?cargoPath:'cargo';
function start(cmd,args,cwd=root){return spawn(cmd,args,{cwd,stdio:'inherit',shell:process.platform==='win32'&&cmd==='npm.cmd'});}
function run(cmd,args,cwd=root){return new Promise((resolve,reject)=>{const p=start(cmd,args,cwd);p.on('error',reject);p.on('exit',code=>code===0?resolve():reject(new Error(cmd+' failed with exit '+code)));});}
const npm=process.platform==='win32'?'npm.cmd':'npm';
async function build(){await run(cargo,['build','--release','--manifest-path','src-core/Cargo.toml']);await run(npm,['run','build'],path.join(root,'src-ui'));}
async function securityTest(){const base=process.env.AEGIS_BRIDGE_URL||'http://127.0.0.1:48291';const handshake=await fetch(base+'/handshake');if(!handshake.ok)throw new Error('Handshake failed');const {token}=await handshake.json();let seq=0;
 const call=async(command,payload={},nonce=token)=>{const r=await fetch(base+'/ipc',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({id:String(++seq),command,payload,nonce})});return {status:r.status,...await r.json()};};
 const assert=(ok,name)=>{if(!ok)throw new Error('FAIL: '+name);console.log('PASS: '+name);};
 assert(!(await call('fs_read_file',{path:'../outside'})).success,'Traversal rejected');
 assert(!(await call('execute_shell',{command:'whoami'})).success,'Shell rejected');
 assert(!(await call('sys_info',{},'invalid')).success,'Invalid token rejected');
 assert(!(await call('hardware_key_enroll',{keyId:'fake',pin:'anything'})).success,'Fake hardware rejected');
 assert(!(await call('wasm_execute_gpu',{moduleId:'wasm-crypto-accelerator'})).success,'GPU unavailable');
 const file='smoke-'+Date.now()+'.vault';assert((await call('vault_encrypt_store',{filename:file,content:'TEST_ONLY',passphrase:'smoke-password-123'})).success,'Vault encrypted');
 const plain=await call('vault_decrypt_retrieve',{filename:file,passphrase:'smoke-password-123'});assert(plain.success&&plain.data.decryptedContent==='TEST_ONLY','Vault roundtrip');
 assert(!(await call('vault_decrypt_retrieve',{filename:file,passphrase:'wrong'})).success,'Wrong password rejected');
 const flood=await Promise.all(Array.from({length:35},()=>call('crypto_hash',{input:'test'})));assert(flood.some(r=>r.status===429),'Rate limiting');
}
try{const command=process.argv[2]||'dev';
 if(command==='dev'){const children=[start(cargo,['run','--manifest-path','src-core/Cargo.toml','--','--spawn-window']),start(npm,['run','dev'],path.join(root,'src-ui'))];const cleanup=()=>{for(const p of children){if(process.platform==='win32'&&p.pid)spawnSync('taskkill',['/pid',String(p.pid),'/t','/f'],{stdio:'ignore'});else p.kill();}process.exit();};process.on('SIGINT',cleanup);process.on('SIGTERM',cleanup);for(const p of children)p.on('error',e=>{console.error(e.message);cleanup();});}
 else if(command==='build'||command==='build-release')await build();
 else if(command==='bundle'){await build();await run(cargo,['run','--release','--manifest-path','src-core/Cargo.toml','--','--bundle']);}
 else if(command==='test-security')await securityTest();else throw new Error('Unknown command: '+command);
}catch(e){console.error(e.message);process.exitCode=1;}
