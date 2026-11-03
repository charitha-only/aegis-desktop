import {lstat,opendir} from 'node:fs/promises';
import path from 'node:path';
async function kind(filename){
 try{const info=await lstat(filename);return info.isSymbolicLink()?'link':info.isDirectory()?'directory':info.isFile()?'file':'other';}
 catch(error){if(error.code==='ENOENT')return 'absent';throw error;}
}
/** Observations only: does not determine lock ownership, package validity or crash durability. */
export async function inspectBundle({outputDirectory,name}){
 if(typeof name!=='string'||!/^[-A-Za-z0-9_.]+$/.test(name)||name==='.'||name.includes('..'))throw new Error('Invalid package name');
 if(typeof outputDirectory!=='string'||!outputDirectory)throw new Error('Output directory required');
 const directory=path.resolve(outputDirectory),rootType=await kind(directory);
 if(rootType!=='directory'&&rootType!=='absent')throw new Error('Output root must be an ordinary directory');
 const stages=[],possibleAliasStages=[];
 if(rootType==='directory'){
  let count=0;
  for await(const entry of await opendir(directory)){
   if(++count>4096)throw new Error('Bundle inspection exceeds 4096 entries');
   const suffix=entry.name.slice(('.aegis-stage-'+name+'-').length);
   if(/^[a-f0-9]{32}$/.test(suffix)){
    const prefix='.aegis-stage-'+name+'-';
    const exact=entry.name.startsWith(prefix);
    if(exact||entry.name.toLowerCase().startsWith(prefix.toLowerCase())){
     (exact?stages:possibleAliasStages).push({name:entry.name,type:await kind(path.join(directory,entry.name))});
    }
   }
  }
 }
 for(const list of [stages,possibleAliasStages])list.sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);
 return {schemaVersion:1,outputDirectory:directory,name,root:rootType,
  payload:await kind(path.join(directory,name)),lock:await kind(path.join(directory,'.aegis-bundle-'+name+'.lock')),
  installerSidecar:await kind(path.join(directory,name+'-installer.iss')),stages,possibleAliasStages,
  snapshotOnly:true,publisherAuthenticated:false};
}
