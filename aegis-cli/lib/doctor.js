import {access,readFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';

/** Read-only portable-build diagnostics; native SDKs/signing are separate gates. */
export async function inspectProject(projectRoot) {
  const checks=[];
  const record=(id,ok,detail)=>checks.push({id,ok,detail});
  record('node',Number(process.versions.node.split('.')[0])>=22,process.versions.node);
  const home=process.env.USERPROFILE||process.env.HOME||'';
  let cargo=path.join(home,'.cargo/bin',process.platform==='win32'?'cargo.exe':'cargo');
  try {await access(cargo);} catch {cargo='cargo';}
  const rust=spawnSync(cargo,['--version'],{encoding:'utf8',timeout:10000,windowsHide:true});
  record('cargo',rust.status===0,rust.status===0?rust.stdout.trim():'Install Rust with rustup and a platform linker');
  const nodeDir=path.dirname(process.execPath);
  let npmCli;
  for(const candidate of [path.join(nodeDir,'node_modules/npm/bin/npm-cli.js'),path.join(nodeDir,'../lib/node_modules/npm/bin/npm-cli.js')]) {
    try {await access(candidate);npmCli=candidate;break;} catch {}
  }
  const npm=npmCli?spawnSync(process.execPath,[npmCli,'--version'],{encoding:'utf8',timeout:10000,windowsHide:true}):null;
  record('npm',npm?.status===0,npm?.status===0?npm.stdout.trim():'Install a Node distribution that includes npm');
  try {
    const config=JSON.parse(await readFile(path.join(projectRoot,'aegis.config.json'),'utf8'));
    record('configuration',typeof config.name==='string'&&typeof config.appId==='string','aegis.config.json requires name and stable appId');
  } catch {record('configuration',false,'Missing or invalid aegis.config.json');}
  for(const [id,file,detail] of [
    ['rust-lockfile','src-core/Cargo.lock','Restore the committed Cargo.lock'],
    ['ui-lockfile','src-ui/package-lock.json','Restore the committed UI package-lock.json'],
    ['ui-dependencies','src-ui/node_modules/vite/bin/vite.js','Run npm ci --prefix src-ui'],
  ]) {
    try {await access(path.join(projectRoot,file));record(id,true,file);} catch {record(id,false,detail);}
  }
  return {schemaVersion:1,platform:process.platform,architecture:process.arch,ok:checks.every(check=>check.ok),securityCertification:false,checks};
}
