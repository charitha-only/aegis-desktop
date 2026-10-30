import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {executeReleasePlan} from '../lib/release-build.js';

// Trusted tool boundary fixtures; no real compiler/UI process is mocked internally.
async function fixture(run) {
  const root=await mkdtemp(path.join(tmpdir(),'aegis-release-diagnostic-'));
  try {
    const marker=path.join(root,'downstream.txt');
    const tool=path.join(root,'tool.cjs');
    const downstream=path.join(root,'downstream.cjs');
    await writeFile(downstream,'require("node:fs").writeFileSync('+JSON.stringify(marker)+',"RAN")');
    const plan={commands:[
      {executable:process.execPath,args:[tool],cwd:root,captureCoreArtifact:true},
      {executable:process.execPath,args:[downstream],cwd:root},
    ]};
    await run({root,marker,tool,plan});
  } finally {await rm(root,{recursive:true,force:true});}
}

test('release build rejects a complete UTF-8 diagnostic above the byte limit before downstream work',()=>fixture(async({tool,plan,marker})=>{
  await writeFile(tool,`console.log(JSON.stringify({reason:'compiler-message',message:{rendered:'é'.repeat(524288)}}));
console.log(JSON.stringify({reason:'compiler-artifact',target:{name:'aegis-core',kind:['bin']},executable:process.execPath}));`);
  await assert.rejects(executeReleasePlan(plan,{log:()=>{}}),/diagnostic line exceeds 1 MiB/);
  await assert.rejects(readFile(marker),{code:'ENOENT'});
}));


test('release build rejects an unterminated oversized UTF-8 diagnostic',()=>fixture(async({tool,plan,marker})=>{
  await writeFile(tool,`process.stdout.write('é'.repeat(524289));`);
  await assert.rejects(executeReleasePlan(plan,{log:()=>{}}),/diagnostic line exceeds 1 MiB/);
  await assert.rejects(readFile(marker),{code:'ENOENT'});
}));

test('release build accepts exact-limit lines independently and continues after a valid artifact',()=>fixture(async({tool,plan,marker})=>{
  await writeFile(tool,`const empty=JSON.stringify({reason:'fixture',text:''});
const line=JSON.stringify({reason:'fixture',text:'a'.repeat(1048576-Buffer.byteLength(empty))});
if(Buffer.byteLength(line)!==1048576) throw new Error('Invalid boundary fixture');
console.log(line);console.log(line);
console.log(JSON.stringify({reason:'compiler-artifact',target:{name:'aegis-core',kind:['bin']},executable:process.execPath}));`);
  await executeReleasePlan(plan,{log:()=>{}});
  assert.equal(await readFile(marker,'utf8'),'RAN');
}));

test('release build stops before UI work when a successful tool reports no core artifact',()=>fixture(async({tool,plan,marker})=>{
  await writeFile(tool,`console.log(JSON.stringify({reason:'build-finished',success:true}));`);
  await assert.rejects(executeReleasePlan(plan,{log:()=>{}}),/did not report an aegis-core executable/);
  await assert.rejects(readFile(marker),{code:'ENOENT'});
}));

test('release build stops before UI work when a tool fails after reporting an artifact',()=>fixture(async({tool,plan,marker})=>{
  await writeFile(tool,`console.log(JSON.stringify({reason:'compiler-artifact',target:{name:'aegis-core',kind:['bin']},executable:process.execPath}));process.exitCode=7;`);
  await assert.rejects(executeReleasePlan(plan,{log:()=>{}}),/failed with exit 7/);
  await assert.rejects(readFile(marker),{code:'ENOENT'});
}));
