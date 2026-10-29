import {stat,realpath} from 'node:fs/promises';
import path from 'node:path';

async function file(location) {try {return (await stat(location)).isFile();} catch {return false;}}

/** Resolve trusted local developer tools without running or installing them. */
export async function resolveDeveloperTools(projectRoot) {
  const home=process.env.USERPROFILE||process.env.HOME||'';
  const cargoHome=process.env.CARGO_HOME?path.resolve(projectRoot,process.env.CARGO_HOME):path.resolve(projectRoot,home,'.cargo');
  const rustTool=async name=>{
    const candidate=path.join(cargoHome,'bin',process.platform==='win32'?`${name}.exe`:name);
    return await file(candidate)?candidate:name;
  };
  const nodeDir=path.dirname(process.execPath);
  let npmCli=null;
  for(const candidate of [path.join(nodeDir,'node_modules/npm/bin/npm-cli.js'),path.join(nodeDir,'../lib/node_modules/npm/bin/npm-cli.js')]) {
    if(await file(candidate)) {npmCli=candidate;break;}
  }
  return {node:process.execPath,cargo:await rustTool('cargo'),rustc:await rustTool('rustc'),npmCli};
}

/** Inspect the exact release commands; no tool execution, compilation or writes. */
export async function releaseBuildPlan(projectRoot, {bundle=false} = {}) {
  const root=await realpath(path.resolve(projectRoot));
  const tools=await resolveDeveloperTools(root);
  if(!tools.npmCli) throw new Error('Cannot locate npm-cli.js beside Node; install a Node distribution with npm and run aegis doctor');
  const coreExecutable=path.join(root,'src-core/target/release',process.platform==='win32'?'aegis-core.exe':'aegis-core');
  const commands=[
    {executable:tools.cargo,args:['build','--locked','--release','--manifest-path','src-core/Cargo.toml','--target-dir',path.join(root,'src-core/target')],cwd:root,shell:false},
    {executable:tools.node,args:[tools.npmCli,'run','build'],cwd:path.join(root,'src-ui'),shell:false},
  ];
  if(bundle) commands.push({executable:coreExecutable,args:['--bundle'],cwd:root,shell:false});
  return {coreExecutable,commands};
}
