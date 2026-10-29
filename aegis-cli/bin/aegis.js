#!/usr/bin/env node
import {spawn} from 'node:child_process';
import {createProject} from '../lib/create.js';
import {writePackageIntegrity,verifyPackageIntegrity} from '../lib/package-integrity.js';
import {stageUpdate,inspectStagedUpdate} from '../lib/update-staging.js';
import {verifyUpdate} from '../lib/update-verification.js';
import {buildMacosPackage} from '../lib/macos-package.js';
import {buildDebianPackage} from '../lib/debian-package.js';
import {developmentPlan, parseDevelopmentArgs, startDevelopment} from '../lib/development.js';
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
 if(command==='dev') {
   const options=parseDevelopmentArgs(process.argv.slice(3));
   if(options.printPlan) console.log(JSON.stringify(developmentPlan(options)));
   else {
     const controller=new AbortController();
     const cancel=()=>controller.abort();
     process.on('SIGINT',cancel); process.on('SIGTERM',cancel);
     try {const session=await startDevelopment({projectRoot:root,...options,signal:controller.signal}); await session.finished;}
     catch(error) {if(!controller.signal.aborted || error.message !== 'Development startup cancelled') throw error;}
     finally {process.off('SIGINT',cancel);process.off('SIGTERM',cancel);}
   }
 }
 else if(command==='package-manifest') {
   if(process.argv.length!==4 || !process.argv[3]) throw new Error('Usage: aegis package-manifest <bundle-directory>');
   console.log(JSON.stringify(await writePackageIntegrity(process.argv[3])));
 }
 else if(command==='verify-package') {
   if(process.argv.length!==5 || !process.argv[3]) throw new Error('Usage: aegis verify-package <bundle-directory> <trusted-manifest-sha256>');
   console.log(JSON.stringify(await verifyPackageIntegrity(process.argv[3],process.argv[4])));
 }
 else if(command==='stage-update') {
   if(process.argv.length!==8) throw new Error('Usage: aegis stage-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json> <new-stage-directory>');
   console.log(JSON.stringify(await stageUpdate({artifactFile:process.argv[3],envelopeFile:process.argv[4],publicKeyFile:process.argv[5],policyFile:process.argv[6],stageDirectory:process.argv[7]})));
 }
 else if(command==='inspect-update') {
   if(process.argv.length!==6) throw new Error('Usage: aegis inspect-update <stage-directory> <trusted-public-key.pem> <trusted-policy.json>');
   console.log(JSON.stringify(await inspectStagedUpdate({stageDirectory:process.argv[3],publicKeyFile:process.argv[4],policyFile:process.argv[5]})));
 }
 else if(command==='verify-update') {
   if(process.argv.length!==7) throw new Error('Usage: aegis verify-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json>');
   console.log(JSON.stringify(await verifyUpdate({artifactFile:process.argv[3],envelopeFile:process.argv[4],publicKeyFile:process.argv[5],policyFile:process.argv[6]})));
 }
 else if(command==='package-macos') {
   if(process.argv.length!==6) throw new Error('Usage: aegis package-macos <bundle-directory> <output.pkg> <trusted-manifest-sha256>');
   console.log(JSON.stringify(await buildMacosPackage({bundleDirectory:process.argv[3],outputFile:process.argv[4],expectedManifestSha256:process.argv[5]})));
 }
 else if(command==='package-deb') {
   if(process.argv.length!==7) throw new Error('Usage: aegis package-deb <bundle-directory> <output.deb> <trusted-manifest-sha256> <maintainer>');
   console.log(JSON.stringify(await buildDebianPackage({bundleDirectory:process.argv[3],outputFile:process.argv[4],expectedManifestSha256:process.argv[5],maintainer:process.argv[6]})));
 }
 else if(command==='create')console.log('Project created: '+await createProject(root,process.argv[3]));
 else if(command==='--help'||command==='help')console.log('Aegis commands:\n  create <new-directory>\n  dev [--no-open] [--ui-port=<port>] [--core-binary=<path>] [--print-plan]\n  build\n  build-release\n  bundle\n  test-security\n  package-manifest <bundle-directory>\n  verify-package <bundle-directory> <trusted-manifest-sha256>\n  stage-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json> <new-stage-directory>\n  inspect-update <stage-directory> <trusted-public-key.pem> <trusted-policy.json>\n  verify-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json>\n  package-macos <bundle-directory> <output.pkg> <trusted-manifest-sha256>\n  package-deb <bundle-directory> <output.deb> <trusted-manifest-sha256> <maintainer>\n  --version');
 else if(command==='--version')console.log(JSON.parse(fs.readFileSync(path.join(root,'package.json'),'utf8')).version);
 else if(command==='build'||command==='build-release')await build();
 else if(command==='bundle'){await build();await run(cargo,['run','--release','--manifest-path','src-core/Cargo.toml','--','--bundle']);}
 else if(command==='test-security')await securityTest();else throw new Error('Unknown command: '+command);
}catch(e){console.error(e.message);process.exitCode=1;}
