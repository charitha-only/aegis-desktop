#!/usr/bin/env node
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { withBenchmarkCore } from '../lib/benchmark-core.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const binaryCandidates = [
  path.join(root, 'dist-bundle/AegisDesktopApplication', process.platform === 'win32' ? 'aegis-core.exe' : 'aegis-core'),
  path.join(root, 'src-core/target/release', process.platform === 'win32' ? 'aegis-core.exe' : 'aegis-core'),
  path.join(root, 'src-core/target/debug', process.platform === 'win32' ? 'aegis-core.exe' : 'aegis-core'),
];

const binary = binaryCandidates.find(fs.existsSync);
if (!binary) {
  console.error('Error: aegis-core binary not found. Run "cargo build" or "npm run build:release" first.');
  process.exit(1);
}

console.log(`==> Running Live Smoke Suite against ephemeral cores: ${binary} <==\n`);

async function runSmoke(label, scriptRelativePath) {
  console.log(`--- Running ${label} ---`);
  await withBenchmarkCore({ binary, args: ['--port=0'], startupTimeoutMs: 15000 }, async ({ base }) => {
    console.log(`[Core Ready] Bound to ephemeral origin: ${base}`);
    const scriptPath = path.join(root, scriptRelativePath);
    const res = spawnSync(process.execPath, [scriptPath], {
      cwd: root,
      env: { ...process.env, AEGIS_BRIDGE_URL: base },
      stdio: 'inherit',
    });
    if (res.status !== 0) {
      throw new Error(`${label} failed with exit code ${res.status}`);
    }
  });
  console.log(`[PASS] ${label}\n`);
}

try {
  await runSmoke('Sensitive Operations Smoke Test', 'aegis-cli/bin/sensitive-smoke.js');
  await runSmoke('Session Lifecycle Smoke Test', 'aegis-cli/bin/session-smoke.js');

  console.log('✅ All live smoke and security tests passed successfully.');
} catch (err) {
  console.error('\n❌ Smoke suite failed:', err.message);
  process.exit(1);
}
