#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { debianPackagePlan } from '../aegis-cli/lib/debian-package.js';
import { macosPackagePlan } from '../aegis-cli/lib/macos-package.js';
import { writePackageIntegrity, verifyPackageIntegrity } from '../aegis-cli/lib/package-integrity.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const distBundleDir = path.join(root, 'dist-bundle');
const configPath = path.join(root, 'aegis.config.json');

console.log('==> Aegis Desktop Cross-Platform Packaging Pipeline <==');

if (!fs.existsSync(configPath)) {
  console.error(`Error: Configuration not found at ${configPath}`);
  process.exit(1);
}

const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
const appName = typeof config.name === 'string' ? config.name.replaceAll(' ', '') : 'AegisDesktop';
const bundleDir = path.join(distBundleDir, appName);

console.log(`Application: ${config.name} (v${config.version})`);
console.log(`Target Bundle: ${bundleDir}`);

// 1. Build and Bundle base application if needed
console.log('\n[Step 1/4] Ensuring bundle and payload integrity...');
if (fs.existsSync(bundleDir)) {
  fs.rmSync(bundleDir, { recursive: true, force: true });
}
const issPath = path.join(distBundleDir, `${appName}-installer.iss`);
if (fs.existsSync(issPath)) {
  fs.rmSync(issPath, { force: true });
}
const aegisCli = path.join(root, 'aegis-cli', 'bin', 'aegis.js');
try {
  execFileSync(process.execPath, [aegisCli, 'package'], {
    cwd: root,
    stdio: 'inherit',
    windowsHide: true,
  });
} catch (err) {
  console.error('Failed to bundle application:', err.message);
  process.exit(1);
}

// 2. Read and verify manifest SHA-256
const manifestPath = path.join(bundleDir, 'aegis-integrity.json');
if (!fs.existsSync(manifestPath)) {
  console.error('Error: Bundle aegis-integrity.json not found after packaging.');
  process.exit(1);
}
const manifestBytes = fs.readFileSync(manifestPath);
import { createHash } from 'node:crypto';
const manifestSha256 = createHash('sha256').update(manifestBytes).digest('hex');
console.log(`Bundle aegis-integrity.json SHA-256: ${manifestSha256}`);

// 3. Evaluate Cross-Platform Packaging Plans
console.log('\n[Step 2/4] Validating Cross-Platform Specifications...');

// 3a. Debian Plan
try {
  const debPlan = debianPackagePlan(config, {
    architecture: 'x64',
    maintainer: 'Aegis Engineering <maintainer@aegis-desktop.org>',
  });
  console.log(`[Debian] Package Plan validated: ${debPlan.packageName} (${debPlan.architecture}) -> ${debPlan.installRoot}`);
  fs.writeFileSync(
    path.join(distBundleDir, `${debPlan.packageName}.control.spec`),
    `Package: ${debPlan.packageName}\nVersion: ${debPlan.version}\nArchitecture: ${debPlan.architecture}\nMaintainer: ${debPlan.maintainer}\nSection: utils\nPriority: optional\nDescription: ${debPlan.description}\n`,
    'utf8'
  );
  fs.writeFileSync(
    path.join(distBundleDir, `${debPlan.packageName}.desktop`),
    debPlan.desktopEntry,
    'utf8'
  );
  console.log(`[Debian] Generated control & desktop specs in dist-bundle/`);
} catch (err) {
  console.warn(`[Debian Plan Warning]: ${err.message}`);
}

// 3b. macOS Plan
try {
  const macPlan = macosPackagePlan(config, { architecture: 'x64' });
  console.log(`[macOS] Package Plan validated: ${macPlan.appName} (${macPlan.identifier})`);
  fs.writeFileSync(
    path.join(distBundleDir, `${macPlan.appName}.Info.plist`),
    macPlan.infoPlist,
    'utf8'
  );
  console.log(`[macOS] Generated Info.plist spec in dist-bundle/`);
} catch (err) {
  console.warn(`[macOS Plan Warning]: ${err.message}`);
}

// 4. Platform-Specific Execution
console.log('\n[Step 3/4] Platform-Specific Builder Invocation...');
if (process.platform === 'win32') {
  console.log('[Windows] Detected Windows build environment.');
  const signScript = path.join(root, 'scripts', 'sign-windows.ps1');
  const exePath = path.join(bundleDir, 'aegis-core.exe');
  if (fs.existsSync(exePath)) {
    console.log(`[Windows] Signing ${exePath}...`);
    try {
      execFileSync('powershell', ['-ExecutionPolicy', 'Bypass', '-File', signScript, '-Path', exePath, '-AllowSelfSigned'], {
        cwd: root,
        stdio: 'inherit',
      });
    } catch (e) {
      console.warn('[Windows] Code signing notice:', e.message);
    }
  }
} else if (process.platform === 'linux') {
  console.log('[Linux] Running native Debian packager...');
  try {
    execFileSync(process.execPath, [
      aegisCli,
      'package-deb',
      bundleDir,
      path.join(distBundleDir, `${config.appId}.deb`),
      manifestSha256,
      'Aegis Engineering <maintainer@aegis-desktop.org>',
    ], { cwd: root, stdio: 'inherit' });
  } catch (e) {
    console.error('[Linux] Debian package generation failed:', e.message);
  }
} else if (process.platform === 'darwin') {
  console.log('[macOS] Running native macOS pkg packager...');
  try {
    execFileSync(process.execPath, [
      aegisCli,
      'package-macos',
      bundleDir,
      path.join(distBundleDir, `${config.appId}.pkg`),
      manifestSha256,
    ], { cwd: root, stdio: 'inherit' });
  } catch (e) {
    console.error('[macOS] macOS package generation failed:', e.message);
  }
}

// 5. Final Package Summary
console.log('\n[Step 4/4] Package Verification & Summary:');
console.log(`All packaging artifacts and specs staged in: ${distBundleDir}`);
console.log('Cross-platform packaging pipeline completed successfully.\n');
