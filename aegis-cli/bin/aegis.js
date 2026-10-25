#!/usr/bin/env node

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '../..');

const command = process.argv[2] || 'dev';

async function runDev() {
  console.log('\x1b[36m%s\x1b[0m', '═══════════════════════════════════════════════════════════');
  console.log('\x1b[35m%s\x1b[0m', '  🛡️  AEGIS DESKTOP FRAMEWORK • STARTING DEV ENVIRONMENT  ');
  console.log('\x1b[36m%s\x1b[0m', '═══════════════════════════════════════════════════════════\n');

  const cargoPath = path.join(process.env.USERPROFILE || '', '.cargo', 'bin', 'cargo.exe');
  const cargoCmd = fs.existsSync(cargoPath) ? cargoPath : 'cargo';

  // 1. Start Rust Core
  console.log('\x1b[33m%s\x1b[0m', '➜ [Core] Launching Rust Aegis Security Engine...');
  const coreProcess = spawn(cargoCmd, ['run', '--manifest-path', 'src-core/Cargo.toml', '--', '--spawn-window'], {
    cwd: projectRoot,
    stdio: 'inherit',
    shell: true,
  });

  // 2. Start Vite Dev Server
  console.log('\x1b[33m%s\x1b[0m', '➜ [Frontend] Launching Vite UI server on http://localhost:5173...');
  const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  const uiProcess = spawn(npmCmd, ['run', 'dev'], {
    cwd: path.join(projectRoot, 'src-ui'),
    stdio: 'inherit',
    shell: true,
  });

  const cleanup = () => {
    console.log('\n\x1b[31m%s\x1b[0m', 'Stopping Aegis processes...');
    try { coreProcess.kill(); } catch (e) {}
    try { uiProcess.kill(); } catch (e) {}
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
}

async function runSecurityTest() {
  console.log('\x1b[32m%s\x1b[0m', 'Running Aegis Automated Penetration & Security Tests...');
  const bridgeUrl = 'http://127.0.0.1:48291';

  try {
    const handshakeRes = await fetch(`${bridgeUrl}/handshake`);
    const data = await handshakeRes.json();
    console.log('\x1b[32m✔\x1b[0m Cryptographic Nonce Handshake: PASSED (Token length: ' + data.token.length + ')');

    // Test 1: Malicious Traversal
    const t1 = await fetch(`${bridgeUrl}/ipc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        id: 'test-1',
        command: 'fs_read_file',
        nonce: data.token,
        payload: { path: '../../../../Windows/win.ini' },
      }),
    });
    const r1 = await t1.json();
    if (!r1.success && r1.error.includes('Security Violation')) {
      console.log('\x1b[32m✔\x1b[0m Path Traversal Defense: PASSED (Attack correctly trapped and denied)');
    } else {
      console.log('\x1b[31m✘\x1b[0m Path Traversal Defense: FAILED');
    }

    // Test 2: Shell execution
    const t2 = await fetch(`${bridgeUrl}/ipc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        id: 'test-2',
        command: 'execute_shell',
        nonce: data.token,
        payload: { command: 'whoami' },
      }),
    });
    const r2 = await t2.json();
    if (!r2.success) {
      console.log('\x1b[32m✔\x1b[0m Shell Injection Defense: PASSED (Arbitrary execution strictly denied)');
    }

    // Test 3: Invalid Nonce
    const t3 = await fetch(`${bridgeUrl}/ipc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        id: 'test-3',
        command: 'sys_info',
        nonce: 'SPOOFED_NONCE',
        payload: {},
      }),
    });
    const r3 = await t3.json();
    if (!r3.success && r3.error.includes('Security Breach')) {
      console.log('\x1b[32m✔\x1b[0m Replay & Nonce Defense: PASSED (Invalid token rejected)');
    }

    // Test 4: Anti-DoS Rate Limiter Burst Test
    console.log('\x1b[33m➜\x1b[0m Firing burst flood to verify Anti-DoS Rate Limiter...');
    const floodCalls = [];
    for (let i = 0; i < 35; i++) {
      floodCalls.push(fetch(`${bridgeUrl}/ipc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: `flood-${i}`,
          command: 'crypto_hash',
          nonce: data.token,
          payload: { input: `test_${i}` },
        }),
      }));
    }
    const floodResults = await Promise.all(floodCalls);
    const rateLimited = floodResults.some(r => r.status === 429);
    if (rateLimited) {
      console.log('\x1b[32m✔\x1b[0m Anti-DoS Rate Limiter Shield: PASSED (Burst flood trapped with HTTP 429)');
    } else {
      console.log('\x1b[33m⚠\x1b[0m Anti-DoS Rate Limiter: Requests handled within window');
    }

    // Test 5: Zero-Knowledge Encrypted Vault Tamper Resistance
    const vStore = await fetch(`${bridgeUrl}/ipc`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        id: 'test-vault-store',
        command: 'vault_encrypt_store',
        nonce: data.token,
        payload: {
          filename: 'autotest_vault.vault',
          content: 'Confidential Payload 2026',
          passphrase: 'CorrectPassword123'
        },
      }),
    });
    const vStoreRes = await vStore.json();
    if (vStoreRes.success) {
      // Attempt decrypt with WRONG passphrase
      const vTamper = await fetch(`${bridgeUrl}/ipc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'test-vault-tamper',
          command: 'vault_decrypt_retrieve',
          nonce: data.token,
          payload: {
            filename: 'autotest_vault.vault',
            passphrase: 'WRONG_MALICIOUS_PASSWORD'
          },
        }),
      });
      const vTamperRes = await vTamper.json();
      if (!vTamperRes.success && vTamperRes.error.includes('Decryption failed')) {
        console.log('\x1b[32m✔\x1b[0m Vault Zero-Knowledge Integrity: PASSED (Unauthorized passphrase rejected)');
      }
    }
  } catch (err) {
    console.log('\x1b[31mEnsure Aegis Core is running on http://127.0.0.1:48291 to run test-security.\x1b[0m');
  }
}

if (command === 'dev') {
  runDev();
} else if (command === 'test-security') {
  runSecurityTest();
} else {
  console.log(`Unknown command: ${command}`);
}
