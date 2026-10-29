import {access,readFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {resolveDeveloperTools} from './developer-tools.js';

/** Read-only portable-build diagnostics; native SDKs/signing are separate gates. */
export async function inspectProject(projectRoot) {
  const checks=[];
  const record=(id,ok,detail)=>checks.push({id,ok,detail});
  record('node',Number(process.versions.node.split('.')[0])>=22,process.versions.node);
  const tools=await resolveDeveloperTools(projectRoot);
  const options={cwd:projectRoot,encoding:'utf8',timeout:10000,windowsHide:true,env:{...process.env,RUSTUP_AUTO_INSTALL:'0'}};
  const rust=spawnSync(tools.cargo,['--version'],options);
  record('cargo',rust.status===0,rust.status===0?rust.stdout.trim():'Install Rust with rustup and a platform linker');
  const compiler=spawnSync(tools.rustc,['--version'],options);
  const version=compiler.stdout?.match(/^rustc (\d+)\.(\d+)\./);
  // Matches the current core Cargo.toml rust-version floor.
  const compilerOk=compiler.status===0 && !!version && (Number(version[1])>1 || (Number(version[1])===1 && Number(version[2])>=92));
  record('rustc',compilerOk,compilerOk?compiler.stdout.trim():'Rust compiler 1.92+ is required; install/update the selected rustup toolchain');
  const npm=tools.npmCli?spawnSync(tools.node,[tools.npmCli,'--version'],options):null;
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
  return {schemaVersion:1,platform:process.platform,architecture:process.arch,ok:checks.every(check=>check.ok),securityCertification:false,tools,checks};
}
