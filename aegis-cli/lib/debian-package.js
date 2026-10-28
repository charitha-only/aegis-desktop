import {createHash} from 'node:crypto';
import {mkdtemp,cp,rename,mkdir,writeFile,readFile,lstat,chmod,opendir,open,link,rm,realpath} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {verifyPackageIntegrity} from './package-integrity.js';
const execute=promisify(execFile);
async function command(program,args,cwd) {return (await execute(program,args,{cwd,encoding:'utf8',timeout:60000,maxBuffer:1024*1024,windowsHide:true})).stdout.trim();}
export function debianPackagePlan(config,{architecture,maintainer}) {
  const field=(value,max=256)=>typeof value==='string' && value.length>0 && value.length<=max && !/[\p{Cc}\p{Cf}]/u.test(value);
  if(!config || !field(config.appId,128) || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(config.appId)
    || !field(config.name) || !field(config.version,128) || !/^[0-9][A-Za-z0-9.+~:-]*$/.test(config.version)
    || (config.description!==undefined && !field(config.description,1024))
    || !field(maintainer) || !/^[^<>]+ <[^<>\s@]+@[^<>\s@]+>$/.test(maintainer)
    || !['x64','arm64'].includes(architecture)) throw new Error('Invalid Debian package metadata or architecture');
  const packageName='aegis-'+createHash('sha256').update(config.appId).digest('hex').slice(0,32);
  const installRoot='/opt/'+packageName;
  return {packageName,installRoot,architecture:{x64:'amd64',arm64:'arm64'}[architecture],version:config.version,
    maintainer,name:config.name,description:config.description ?? 'Desktop application built with Aegis',
    desktopEntry:`[Desktop Entry]\nType=Application\nName=${config.name.replaceAll('\\','\\\\')}\nExec=${installRoot}/aegis-core --spawn-window --port=0\nTerminal=false\nCategories=Utility;\n`};
}


/** Build only: never install, elevate privileges, or execute the supplied binary. */
export async function buildDebianPackage({bundleDirectory,outputFile,expectedManifestSha256,maintainer}) {
  if(process.platform!=='linux') throw new Error('Debian package creation requires a native Linux build host');
  if(typeof outputFile!=='string' || !/^[A-Za-z0-9][A-Za-z0-9._-]*\.deb$/.test(path.basename(outputFile))) throw new Error('Choose a .deb output filename');
  const output=path.resolve(outputFile),parent=path.dirname(output);
  const inputRelative=path.relative(await realpath(bundleDirectory),await realpath(parent));
  if(inputRelative==='' || (!inputRelative.startsWith('..'+path.sep)&&inputRelative!=='..'&&!path.isAbsolute(inputRelative))) throw new Error('Package output must be outside the input bundle');
  const work=await mkdtemp(path.join(parent,'.aegis-deb-'));
  try {
    await verifyPackageIntegrity(bundleDirectory,expectedManifestSha256);
    const snapshot=path.join(work,'payload');
    await cp(path.resolve(bundleDirectory),snapshot,{recursive:true,force:false,errorOnExist:true,dereference:false});
    await verifyPackageIntegrity(snapshot,expectedManifestSha256);
    const configFile=path.join(snapshot,'aegis.config.json');
    if((await lstat(configFile)).size>1024*1024) throw new Error('Application config exceeds limit');
    const config=JSON.parse(await readFile(configFile,'utf8'));
    const plan=debianPackagePlan(config,{architecture:process.arch,maintainer});
    if(await command('dpkg',['--print-architecture'],work)!==plan.architecture) throw new Error('Native Debian architecture does not match the build host');
    await command('dpkg',['--validate-version',plan.version],work);
    const binary=path.join(snapshot,'aegis-core');
    const file=await open(binary,'r');
    const header=Buffer.alloc(64);
    try {await file.read(header,0,64,0);} finally {await file.close();}
    const machine=header.readUInt16LE(18);
    if(header.subarray(0,4).toString('hex')!=='7f454c46' || header[4]!==2 || header[5]!==1
      || machine!==({amd64:62,arm64:183}[plan.architecture])) throw new Error('Expected a native 64-bit Linux ELF core');
    const stage=path.join(work,'stage'),installed=path.join(stage,plan.installRoot.slice(1));
    await mkdir(path.dirname(installed),{recursive:true});
    await rename(snapshot,installed);
    async function normalize(directory) {
      await chmod(directory,0o755);
      for await(const entry of await opendir(directory)) {
        const filename=path.join(directory,entry.name),info=await lstat(filename);
        if(info.isSymbolicLink()) throw new Error('Linked staged entry rejected');
        if(info.isDirectory()) await normalize(filename);
        else if(info.isFile()) await chmod(filename,filename===path.join(installed,'aegis-core') || filename===path.join(installed,'launch.sh')?0o755:0o644);
        else throw new Error('Non-file staged entry rejected');
      }
    }
    await normalize(stage);
    const sourceControl=`Source: ${plan.packageName}\nSection: utils\nPriority: optional\nMaintainer: ${plan.maintainer}\n\nPackage: ${plan.packageName}\nArchitecture: any\nDescription: ${plan.description}\n`;
    await mkdir(path.join(work,'debian'));
    await writeFile(path.join(work,'debian/control'),sourceControl);
    const shlibs=await command('dpkg-shlibdeps',['-O','-e',path.join(installed,'aegis-core')],work);
    const dependencyLine=shlibs.split('\n').find(line=>line.startsWith('shlibs:Depends='));
    if(dependencyLine===undefined) throw new Error('Cannot derive native shared-library dependencies');
    const libraries=dependencyLine.slice('shlibs:Depends='.length).trim();
    if(/[\r\n]/.test(libraries)) throw new Error('Invalid dependency metadata');
    const dependencies=[libraries,'xdg-utils'].filter(Boolean).join(', ');
    await mkdir(path.join(stage,'DEBIAN'));
    await writeFile(path.join(stage,'DEBIAN/control'),`Package: ${plan.packageName}\nVersion: ${plan.version}\nArchitecture: ${plan.architecture}\nMaintainer: ${plan.maintainer}\nDepends: ${dependencies}\nSection: utils\nPriority: optional\nDescription: ${plan.description}\n`,{mode:0o644});
    await mkdir(path.join(stage,'usr/share/applications'),{recursive:true});
    await writeFile(path.join(stage,'usr/share/applications',plan.packageName+'.desktop'),plan.desktopEntry,{mode:0o644});
    const artifact=path.join(work,'package.deb');
    await command('dpkg-deb',['--root-owner-group','--build',stage,artifact],work);
    const hash=createHash('sha256');
    for await(const chunk of createReadStream(artifact)) hash.update(chunk);
    // Same-filesystem exclusive publication: an existing output is never replaced.
    await link(artifact,output);
    return {...plan,outputFile:output,packageSha256:hash.digest('hex'),manifestSha256:expectedManifestSha256,
      publisherAuthenticated:false,format:'deb',dependencies};
  } finally {
    const relative=path.relative(parent,work);
    if(relative && !relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(work,{recursive:true,force:true});
  }
}
