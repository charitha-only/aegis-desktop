import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,readFile,rm,mkdir,symlink} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {writePackageIntegrity,verifyPackageIntegrity} from '../lib/package-integrity.js';

test('package verification checks every file against a separately pinned manifest digest',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'aegis-integrity-'));
  try {
    await writeFile(path.join(root,'asset.txt'),'abc');
    const result=await writePackageIntegrity(root);
    const manifest=JSON.parse(await readFile(result.manifestPath,'utf8'));
    assert.equal(manifest.files[0].sha256,'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
    const verified=await verifyPackageIntegrity(root,result.manifestSha256);
    assert.equal(verified.valid,true);
    assert.equal(verified.publisherAuthenticated,false);
    await writeFile(path.join(root,'asset.txt'),'abd');
    await assert.rejects(verifyPackageIntegrity(root,result.manifestSha256),/mismatch/);
  } finally {
    const relative=path.relative(tmpdir(),root);
    if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
  }
});

test('extra/missing files and replaced descriptors fail; a hash alone cannot authorize descriptor paths',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'aegis-integrity-'));
  try {
    const asset=path.join(root,'asset.txt');
    await writeFile(asset,'abc');
    const pin=await writePackageIntegrity(root);
    await assert.rejects(verifyPackageIntegrity(root,''),/trusted manifest/);
    await writeFile(path.join(root,'extra.txt'),'unexpected');
    await assert.rejects(verifyPackageIntegrity(root,pin.manifestSha256),/mismatch/);
    await rm(path.join(root,'extra.txt'));
    await rm(asset);
    await assert.rejects(verifyPackageIntegrity(root,pin.manifestSha256),/mismatch/);
    const malformed=JSON.stringify({schemaVersion:1,files:[{path:'../outside',size:0,sha256:'0'.repeat(64)}]});
    await writeFile(pin.manifestPath,malformed);
    await assert.rejects(verifyPackageIntegrity(root,pin.manifestSha256),/manifest digest mismatch/);
    const trustedBadHash=createHash('sha256').update(malformed).digest('hex');
    await assert.rejects(verifyPackageIntegrity(root,trustedBadHash),/inventory mismatch/);
  } finally {
    const relative=path.relative(tmpdir(),root);
    if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
  }
});

test('linked directories are rejected and existing descriptors are preserved',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'aegis-integrity-'));
  try {
    const bundle=path.join(root,'bundle'),outside=path.join(root,'outside');
    await mkdir(bundle);await mkdir(outside);
    await writeFile(path.join(outside,'private.txt'),'TEST_ONLY fixture');
    await symlink(outside,path.join(bundle,'linked'),process.platform==='win32'?'junction':'dir');
    await assert.rejects(writePackageIntegrity(bundle),/Linked/);
    await rm(path.join(bundle,'linked'));
    await writeFile(path.join(bundle,'asset.txt'),'abc');
    const result=await writePackageIntegrity(bundle);
    await assert.rejects(writePackageIntegrity(bundle),{code:'EEXIST'});
    assert.equal((await verifyPackageIntegrity(bundle,result.manifestSha256)).valid,true);
  } finally {
    const relative=path.relative(tmpdir(),root);
    if(!relative.startsWith('..')&&!path.isAbsolute(relative)) await rm(root,{recursive:true,force:true});
  }
});
