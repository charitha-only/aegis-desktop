#!/usr/bin/env node
// Use a fresh isolated runtime only; creates encrypted test fixtures, logs no credentials.
import {randomBytes} from 'node:crypto';
const base=process.env.AEGIS_BRIDGE_URL||'http://127.0.0.1:48400';
const assert=(ok,label)=>{if(!ok)throw Error('FAIL: '+label);console.log('PASS: '+label);};
try {
 const {token}=await (await fetch(base+'/handshake')).json();let seq=0;
 const call=async(command,payload={})=>(await fetch(base+'/ipc',{method:'POST',headers:{'Content-Type':'application/json',Origin:base},body:JSON.stringify({id:String(++seq),command,payload,nonce:token})})).json();
 const file='session-'+randomBytes(8).toString('hex')+'.vault';const other='other-'+randomBytes(8).toString('hex')+'.vault';
 const pass='session-test-'+randomBytes(16).toString('hex');const next='rotated-test-'+randomBytes(16).toString('hex');
 const rejected=r=>!r.success&&/session.*(locked|expired|invalid)/i.test(r.error||'');
 assert((await call('vault_encrypt_store',{filename:file,content:'SESSION_FIXTURE',passphrase:pass})).success,'Create primary vault');
 assert((await call('vault_encrypt_store',{filename:other,content:'OTHER_FIXTURE',passphrase:pass})).success,'Create second vault');
 assert(!(await call('vault_session_unlock',{filename:file,passphrase:'wrong-password'})).success,'Wrong password cannot unlock');
 const unlocked=await call('vault_session_unlock',{filename:file,passphrase:pass});assert(unlocked.success&&unlocked.data?.sessionToken?.length===64,'Authenticated session issued');
 const sessionToken=unlocked.data.sessionToken;
 assert(rejected(await call('vault_session_read',{filename:file,sessionToken:'forged'})),'Forged session rejected');
 assert(rejected(await call('vault_session_read',{filename:other,sessionToken})),'Existing other vault rejects session');
 assert((await call('vault_session_read',{filename:file,sessionToken})).data?.decryptedContent==='SESSION_FIXTURE','Scoped session returns authenticated plaintext');
 assert((await call('secrets_store',{key:'temporary',value:'TEST_ONLY',masterPassphrase:pass})).success,'Temporary secret created');
 assert((await call('vault_session_lock')).success,'Explicit lock succeeds');
 assert(rejected(await call('vault_session_read',{filename:file,sessionToken})),'Locked session rejected');
 assert((await call('secrets_list')).data?.secrets?.length===0,'Lock clears ephemeral secrets');
 const reopened=await call('vault_session_unlock',{filename:file,passphrase:pass});assert(reopened.success,'Fresh password reauthenticates after lock');
 assert((await call('profile_switch',{profileId:'private_vault'})).success,'Switch away');
 assert((await call('profile_switch',{profileId:'default'})).success,'Switch back');
 assert(rejected(await call('vault_session_read',{filename:file,sessionToken:reopened.data.sessionToken})),'Profile switch revokes token');
 const before=await call('vault_session_unlock',{filename:file,passphrase:pass});assert(before.success,'Unlock before password rotation');
 assert((await call('vault_rotate_passphrase',{filename:file,oldPassphrase:pass,newPassphrase:next})).success,'Rotate password');
 assert(rejected(await call('vault_session_read',{filename:file,sessionToken:before.data.sessionToken})),'Rotation revokes retained credential');
 const rotated=await call('vault_session_unlock',{filename:file,passphrase:next});assert(rotated.success,'New password unlocks');
 assert((await call('vault_session_read',{filename:file,sessionToken:rotated.data.sessionToken})).data?.decryptedContent==='SESSION_FIXTURE','New session reads rotated vault');
 assert((await call('vault_session_lock')).success,'Final lock');
}catch(e){console.error(e.message);process.exitCode=1;}
