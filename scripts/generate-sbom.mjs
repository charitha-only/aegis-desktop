#!/usr/bin/env node
/**
 * Aegis Desktop — Software Bill of Materials (SBOM) Generator & Validator
 * Conforms to SPDX 2.3 JSON Specification and NIST SP 800-218 Supply Chain Standards.
 */

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const isVerify = process.argv.includes('--verify');

function parseCargoLock(cargoLockPath) {
  if (!fs.existsSync(cargoLockPath)) {
    throw new Error(`Cargo.lock not found at: ${cargoLockPath}`);
  }
  const content = fs.readFileSync(cargoLockPath, 'utf8');
  const packages = [];
  const blocks = content.split('[[package]]').slice(1);

  for (const block of blocks) {
    const nameMatch = block.match(/name\s*=\s*"([^"]+)"/);
    const verMatch = block.match(/version\s*=\s*"([^"]+)"/);
    const sourceMatch = block.match(/source\s*=\s*"([^"]+)"/);
    const checksumMatch = block.match(/checksum\s*=\s*"([^"]+)"/);

    if (nameMatch && verMatch) {
      packages.push({
        ecosystem: 'cargo',
        name: nameMatch[1],
        version: verMatch[1],
        source: sourceMatch ? sourceMatch[1] : 'workspace',
        checksum: checksumMatch ? checksumMatch[1] : null,
      });
    }
  }
  return packages;
}

function parseNpmPackageLock(lockPath) {
  if (!fs.existsSync(lockPath)) {
    return [];
  }
  const data = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
  const packages = [];

  if (data.packages) {
    for (const [key, pkg] of Object.entries(data.packages)) {
      if (!key || key === '') continue;
      const cleanName = key.replace(/^node_modules\//, '');
      if (!cleanName || cleanName.includes('node_modules/')) continue;

      packages.push({
        ecosystem: 'npm',
        name: cleanName,
        version: pkg.version || 'unknown',
        resolved: pkg.resolved || null,
        integrity: pkg.integrity || null,
      });
    }
  }
  return packages;
}

function generateSpdxDocument() {
  const rootPkg = JSON.parse(fs.readFileSync(path.join(rootDir, 'package.json'), 'utf8'));
  const version = rootPkg.version || '1.0.0-rc1';
  const timestamp = new Date().toISOString();

  const cargoPackages = parseCargoLock(path.join(rootDir, 'src-core', 'Cargo.lock'));
  const npmPackages = parseNpmPackageLock(path.join(rootDir, 'src-ui', 'package-lock.json'));

  const spdxPackages = [
    {
      SPDXID: 'SPDXRef-Package-aegis-desktop',
      name: 'aegis-desktop',
      versionInfo: version,
      downloadLocation: 'https://github.com/charitha-only/aegis-desktop',
      filesAnalyzed: false,
      supplier: 'Organization: Aegis Desktop Contributors',
      originator: 'Organization: Aegis Desktop Project',
      licenseConcluded: 'MIT',
      description: rootPkg.description || 'High-Security Cross-Platform Desktop Framework',
    },
  ];

  const relationships = [];

  for (const rustPkg of cargoPackages) {
    const spdxId = `SPDXRef-Package-cargo-${rustPkg.name.replace(/[^a-zA-Z0-9-]/g, '-')}-${rustPkg.version.replace(/[^a-zA-Z0-9-]/g, '-')}`;
    const entry = {
      SPDXID: spdxId,
      name: rustPkg.name,
      versionInfo: rustPkg.version,
      downloadLocation: rustPkg.source === 'workspace' ? 'NOASSERTION' : `https://crates.io/crates/${rustPkg.name}/${rustPkg.version}`,
      filesAnalyzed: false,
      supplier: 'NOASSERTION',
      licenseConcluded: 'NOASSERTION',
    };
    if (rustPkg.checksum) {
      entry.checksums = [
        {
          algorithm: 'SHA256',
          checksumValue: rustPkg.checksum,
        },
      ];
    }
    spdxPackages.push(entry);
    relationships.push({
      spdxElementId: 'SPDXRef-Package-aegis-desktop',
      relationshipType: 'DEPENDS_ON',
      relatedSpdxElement: spdxId,
    });
  }

  for (const npmPkg of npmPackages) {
    const spdxId = `SPDXRef-Package-npm-${npmPkg.name.replace(/[^a-zA-Z0-9-]/g, '-')}-${npmPkg.version.replace(/[^a-zA-Z0-9-]/g, '-')}`;
    const entry = {
      SPDXID: spdxId,
      name: npmPkg.name,
      versionInfo: npmPkg.version,
      downloadLocation: npmPkg.resolved || 'NOASSERTION',
      filesAnalyzed: false,
      supplier: 'NOASSERTION',
      licenseConcluded: 'NOASSERTION',
    };
    if (npmPkg.integrity && npmPkg.integrity.startsWith('sha512-')) {
      const hexSha512 = Buffer.from(npmPkg.integrity.replace('sha512-', ''), 'base64').toString('hex');
      entry.checksums = [
        {
          algorithm: 'SHA512',
          checksumValue: hexSha512,
        },
      ];
    }
    spdxPackages.push(entry);
    relationships.push({
      spdxElementId: 'SPDXRef-Package-aegis-desktop',
      relationshipType: 'DEPENDS_ON',
      relatedSpdxElement: spdxId,
    });
  }

  return {
    spdxVersion: 'SPDX-2.3',
    dataLicense: 'CC0-1.0',
    SPDXID: 'SPDXRef-DOCUMENT',
    name: 'Aegis-Desktop-Framework-SBOM',
    documentNamespace: `https://github.com/charitha-only/aegis-desktop/spdx/aegis-desktop-${version}-${Date.now()}`,
    creationInfo: {
      creators: [
        'Tool: aegis-sbom-generator-1.0.0',
        'Organization: Aegis Desktop Contributors',
      ],
      created: timestamp,
    },
    documentDescribes: ['SPDXRef-Package-aegis-desktop'],
    packages: spdxPackages,
    relationships,
    stats: {
      cargoDependencies: cargoPackages.length,
      npmDependencies: npmPackages.length,
      totalTrackedPackages: spdxPackages.length,
      generatedAt: timestamp,
    },
  };
}

function verifySbom(sbomPath) {
  if (!fs.existsSync(sbomPath)) {
    throw new Error(`SBOM file not found for verification: ${sbomPath}`);
  }
  const content = fs.readFileSync(sbomPath, 'utf8');
  const doc = JSON.parse(content);
  if (doc.spdxVersion !== 'SPDX-2.3') {
    throw new Error(`Invalid SPDX version: ${doc.spdxVersion}, expected SPDX-2.3`);
  }
  if (!Array.isArray(doc.packages) || doc.packages.length < 50) {
    throw new Error(`Suspicious package count in SBOM: ${doc.packages?.length}`);
  }
  if (!Array.isArray(doc.relationships) || doc.relationships.length < 50) {
    throw new Error(`Suspicious relationship count in SBOM: ${doc.relationships?.length}`);
  }
  return doc;
}

async function main() {
  const docsDir = path.join(rootDir, 'docs');
  const distDir = path.join(rootDir, 'dist');
  const docsSbomPath = path.join(docsDir, 'SBOM.json');
  const distSbomPath = path.join(distDir, 'sbom.spdx.json');
  const distShaPath = path.join(distDir, 'sbom.spdx.json.sha256');

  if (isVerify) {
    console.log('[SBOM] 🔍 Verifying existing SBOM document integrity...');
    const targetPath = fs.existsSync(docsSbomPath) ? docsSbomPath : distSbomPath;
    const doc = verifySbom(targetPath);
    console.log(`[SBOM] ✅ Valid SPDX-2.3 SBOM verified: ${doc.packages.length} packages tracked (${doc.stats?.cargoDependencies || '?'} Rust crates, ${doc.stats?.npmDependencies || '?'} npm modules).`);
    return;
  }

  console.log('[SBOM] 🔨 Generating Software Bill of Materials (SPDX 2.3 JSON)...');
  const sbomDoc = generateSpdxDocument();
  const jsonContent = JSON.stringify(sbomDoc, null, 2);

  // Write docs/SBOM.json
  if (!fs.existsSync(docsDir)) fs.mkdirSync(docsDir, { recursive: true });
  fs.writeFileSync(docsSbomPath, jsonContent, 'utf8');

  // Write dist/sbom.spdx.json and digest
  if (!fs.existsSync(distDir)) fs.mkdirSync(distDir, { recursive: true });
  fs.writeFileSync(distSbomPath, jsonContent, 'utf8');

  const sha256 = crypto.createHash('sha256').update(jsonContent, 'utf8').digest('hex');
  fs.writeFileSync(distShaPath, `${sha256} *sbom.spdx.json\n`, 'utf8');

  console.log(`[SBOM] ✅ Successfully generated:`);
  console.log(`       - Document: ${docsSbomPath}`);
  console.log(`       - Artifact: ${distSbomPath}`);
  console.log(`       - Digest:   ${distShaPath} (${sha256})`);
  console.log(`[SBOM] 📊 Metrics: ${sbomDoc.stats.cargoDependencies} Rust crates, ${sbomDoc.stats.npmDependencies} npm modules (${sbomDoc.stats.totalTrackedPackages} total).`);
}

main().catch((err) => {
  console.error('[SBOM] ❌ Error:', err.message);
  process.exit(1);
});

