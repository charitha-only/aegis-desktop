#!/usr/bin/env node
// Run against a fresh, isolated test core. Does not log passwords or plaintext.
import {randomBytes} from 'node:crypto';
const base=process.env.AEGIS_BRIDGE_URL||'http://127.0.0.1:48291';
const assert=(ok,label)=>{if(!ok)throw Error('FAIL: '+label);console.log('PASS: '+label);};
try{
 const page=await fetch(base+'/');assert(page.ok&&page.headers.get('content-security-policy')?.includes("connect-src 'self'"),'Strict packaged CSP');
 const dev=await fetch(base+'/handshake',{headers:{Origin:'http://localhost:5173'}});assert(dev.status===403,'Development origin rejected in package');
 const hs=await fetch(base+'/handshake');assert(hs.headers.get('cache-control')==='no-store','Handshake is not cached');const {token}=await hs.json();let seq=0;
 async function call(command,payload={},nonce=token){const r=await fetch(base+'/ipc',{method:'POST',headers:{'Content-Type':'application/json',Origin:base},body:JSON.stringify({id:String(++seq),command,payload,nonce})});assert(r.headers.get('cache-control')==='no-store','IPC response is not cached');return r.json();}
 const file='sensitive-smoke-'+randomBytes(8).toString('hex')+'.vault';const password='test-only-master-'+randomBytes(12).toString('hex');const replacement='rotated-master-'+randomBytes(12).toString('hex');
 assert((await call('vault_encrypt_store',{filename:file,content:'FIRST',passphrase:password})).success,'Initial vault creation');
 assert(!(await call('vault_encrypt_store',{filename:file,content:'WRONG',passphrase:'wrong-password'})).success,'Unauthenticated overwrite rejected');
 assert((await call('vault_decrypt_retrieve',{filename:file,passphrase:password})).data?.decryptedContent==='FIRST','Failed overwrite preserves data');
 assert(!(await call('fs_write_file',{path:file,content:'CORRUPT'})).success,'Plaintext API cannot overwrite vault');
 assert((await call('vault_encrypt_store',{filename:file,content:'SECOND',passphrase:password})).success,'Authenticated replacement');
 assert(!(await call('vault_restore_backup',{filename:file,passphrase:'wrong-password'})).success,'Unauthenticated restore rejected');
 assert((await call('vault_restore_backup',{filename:file,passphrase:password})).success,'Authenticated encrypted backup restore');
 assert((await call('vault_decrypt_retrieve',{filename:file,passphrase:password})).data?.decryptedContent==='FIRST','Restored contents match');
 assert((await call('vault_rotate_passphrase',{filename:file,oldPassphrase:password,newPassphrase:replacement})).success,'Password rotation');
 assert(!(await call('vault_decrypt_retrieve',{filename:file,passphrase:password})).success,'Old password rejected after rotation');
 assert((await call('vault_decrypt_retrieve',{filename:file,passphrase:replacement})).data?.decryptedContent==='FIRST','Rotated vault decrypts');
 assert(!(await call('fs_write_file',{path:'CON.txt',content:'x'})).success,'Reserved Windows device name rejected');
 const plain=await fetch(base+'/ipc',{method:'POST',headers:{'Content-Type':'text/plain'},body:'{}'});assert(plain.status===415,'Non-JSON body rejected');
 assert((await call('secrets_store',{key:'smoke-'+randomBytes(8).toString('hex'),value:'TEST_ONLY',masterPassphrase:password})).success,'Ephemeral secret created');
 assert((await call('profile_switch',{profileId:'private_vault'})).success,'Profile switch');
 assert((await call('secrets_list')).data.secrets.length===0,'Ephemeral secrets cleared on profile switch');
 await call('profile_switch',{profileId:'default'});
 const untrusted=await fetch(base+'/ipc',{method:'OPTIONS',headers:{Origin:'https://untrusted.example'}});assert(untrusted.status===403,'Untrusted origin rejected');
}catch(e){console.error(e.message);process.exitCode=1;}
