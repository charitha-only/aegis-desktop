import {createHash} from 'node:crypto';
import {mkdtemp,cp,rename,mkdir,writeFile,readFile,lstat,chmod,opendir,link,rm,realpath} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {verifyPackageIntegrity} from './package-integrity.js';
const execute=promisify(execFile);
async function command(program,args,cwd) {return (await execute(program,args,{cwd,encoding:'utf8',timeout:60000,maxBuffer:1024*1024,windowsHide:true})).stdout.trim();}
const xml=value=>value.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&apos;');
const plist=body=>`<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0">${body}</plist>\n`;
export function macosPackagePlan(config,{architecture}) {
  if(!config || typeof config.appId!=='string' || !/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(config.appId)
    || typeof config.name!=='string' || !config.name.trim() || config.name.length>256 || /[\p{Cc}\p{Cf}\p{Cs}]/u.test(config.name)
    || typeof config.version!=='string' || !/^[0-9]{1,4}\.[0-9]{1,2}\.[0-9]{1,2}$/.test(config.version)
    || !['x64','arm64'].includes(architecture)) throw new Error('Invalid macOS package metadata or architecture; use a numeric major.minor.patch version');
  const suffix=createHash('sha256').update(config.appId).digest('hex').slice(0,32);
  const identifier='org.aegis.app.'+suffix;
  const appName='Aegis-'+suffix+'.app';
  return {identifier,appName,installRoot:'/Applications/'+appName,architecture,version:config.version,
    infoPlist:plist(`<dict><key>CFBundleIdentifier</key><string>${identifier}</string><key>CFBundleName</key><string>${xml(config.name)}</string><key>CFBundleDisplayName</key><string>${xml(config.name)}</string><key>CFBundleExecutable</key><string>launch</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleVersion</key><string>${config.version}</string><key>CFBundleShortVersionString</key><string>${config.version}</string></dict>`)};
}
/** Native build only: never install, elevate or launch the input executable. */
export async function buildMacosPackage({bundleDirectory,outputFile,expectedManifestSha256}) {
  if(process.platform!=='darwin') throw new Error('Package creation requires a native macOS build host');
  if(typeof outputFile!=='string'||!/^[A-Za-z0-9][A-Za-z0-9._-]*\.pkg$/.test(path.basename(outputFile))) throw new Error('Choose a .pkg output filename');
  const output=path.resolve(outputFile),parent=path.dirname(output);
  const relative=path.relative(await realpath(bundleDirectory),await realpath(parent));
  if(relative===''||(!relative.startsWith('..'+path.sep)&&relative!=='..'&&!path.isAbsolute(relative))) throw new Error('Package output must be outside the input bundle');
  const work=await mkdtemp(path.join(parent,'.aegis-macos-'));
  try {
    await verifyPackageIntegrity(bundleDirectory,expectedManifestSha256);
    const snapshot=path.join(work,'payload');
    await cp(path.resolve(bundleDirectory),snapshot,{recursive:true,force:false,errorOnExist:true,dereference:false});
    await verifyPackageIntegrity(snapshot,expectedManifestSha256);
    const configFile=path.join(snapshot,'aegis.config.json');
    if((await lstat(configFile)).size>1024*1024) throw new Error('Application config exceeds limit');
    const plan=macosPackagePlan(JSON.parse(await readFile(configFile,'utf8')),{architecture:process.arch});
    const binary=path.join(snapshot,'aegis-core');
    const expectedArch={x64:'x86_64',arm64:'arm64'}[process.arch];
    if(await command('/usr/bin/lipo',['-archs',binary],work)!==expectedArch) throw new Error('Expected a single native macOS architecture');
    const dependencies=await command('/usr/bin/otool',['-L',binary],work);
    for(const dependency of dependencies.split('\n').slice(1).filter(Boolean)) {
      if(!/^\s+\/(usr\/lib\/|System\/Library\/)/.test(dependency)) throw new Error('Non-system dynamic library dependency is not bundled');
    }
    const stage=path.join(work,'stage'),app=path.join(stage,'Applications',plan.appName),contents=path.join(app,'Contents');
    await mkdir(contents,{recursive:true});
    await rename(snapshot,path.join(contents,'MacOS'));
    await writeFile(path.join(contents,'Info.plist'),plan.infoPlist);
    const launch='#!/bin/sh\nBASE=$(CDPATH= cd "$(dirname "$0")" && pwd) || exit 1\nexec "$BASE/aegis-core" --spawn-window --port=0 "$@"\n';
    await writeFile(path.join(contents,'MacOS/launch'),launch,{flag:'wx'});
    async function normalize(directory) {
      await chmod(directory,0o755);
      for await(const entry of await opendir(directory)) {
        const filename=path.join(directory,entry.name),info=await lstat(filename);
        if(info.isSymbolicLink()) throw new Error('Linked staged entry rejected');
        if(info.isDirectory()) await normalize(filename);
        else if(info.isFile()) await chmod(filename,['aegis-core','launch'].some(name=>filename===path.join(contents,'MacOS',name))?0o755:0o644);
        else throw new Error('Non-file staged entry rejected');
      }
    }
    await normalize(stage);
    await command('/usr/bin/plutil',['-lint',path.join(contents,'Info.plist')],work);
    // Fixed destination; never chase a user's relocated copy or merge obsolete files.
    const component=plist(`<array><dict><key>RootRelativeBundlePath</key><string>Applications/${plan.appName}</string><key>BundleIsRelocatable</key><false/><key>BundleIsVersionChecked</key><true/><key>BundleOverwriteAction</key><string>upgrade</string></dict></array>`);
    const componentFile=path.join(work,'components.plist');await writeFile(componentFile,component);
    const artifact=path.join(work,'application.pkg');
    await command('/usr/bin/pkgbuild',['--root',stage,'--identifier',plan.identifier,'--version',plan.version,'--install-location','/','--ownership','recommended','--component-plist',componentFile,artifact],work);
    const hash=createHash('sha256');for await(const chunk of createReadStream(artifact)) hash.update(chunk);
    await link(artifact,output);
    return {...plan,outputFile:output,packageSha256:hash.digest('hex'),manifestSha256:expectedManifestSha256,publisherAuthenticated:false,format:'pkg',dependencies};
  } finally {
    const owned=path.relative(parent,work);
    if(owned&&!owned.startsWith('..')&&!path.isAbsolute(owned)) await rm(work,{recursive:true,force:true});
  }
}
