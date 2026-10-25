# 📜 Aegis Desktop Changelog & Release Notes

All notable changes, new features, security updates, and bug fixes for **Aegis Desktop** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## 🔮 [Upcoming Roadmap] - v0.20.0
> **Target Release:** Q1 2027

### 🚀 Planned Features & Enhancements
- [ ] **macOS & Linux Native WebKit:** Full native webview integration via `wry` and `tao` (replacing Edge App Mode fallback).
- [ ] **Hardware Security Key Integration:** FIDO2 / WebAuthn and YubiKey challenge-response in VaultEngine.
- [ ] **Wasm Plugin Sandbox:** Running dynamically compiled WebAssembly modules inside Rust Core with memory isolation.
- [ ] **Multi-Profile Isolation:** Sandboxed runtime instances with separate keyrings and storage vaults.

---

## 🚀 [v0.13.0] - 2026-10-25
> **The Extensibility, Notifications, Air-Gapped Updater & Capability Matrix Release**

### ✨ Highlights & New Features
- **Extensible Rust Plugin Engine (`src-core/src/plugin.rs`):**
  - Thread-safe `PluginRegistry` allows registration of memory-safe Rust closures with fine-grained capability descriptors.
  - Three out-of-the-box native built-in plugins:
    - `system_memory` — RAM utilization, total capacity, and paging telemetry.
    - `file_checksum` — Streaming SHA-256 integrity verification inside the sandbox.
    - `process_uptime` — Host OS uptime and Aegis daemon process metrics.
  - New IPC commands: `plugin_list` and `plugin_dispatch`.
- **System Tray & Native OS Notifications (`src-core/src/tray.rs`):**
  - Zero-dependency OS notification dispatch using native subsystems:
    - **Windows:** PowerShell WinRT Toast integration (`Windows.UI.Notifications`).
    - **macOS:** AppleScript (`display notification`).
    - **Linux:** Freedesktop `notify-send`.
  - Input sanitization against command injection before invoking native OS hooks.
  - New IPC commands: `notify_send` and `tray_status`.
- **Air-Gapped Cryptographic Auto-Updater (`src-core/src/updater.rs`):**
  - Strict SemVer version comparison (`check_update_available`).
  - Streaming SHA-256 cryptographic integrity verification (`verify_integrity`).
  - Isolated atomic staging into `.aegis_updates/` sandbox before application restart.
  - Air-gapped offline support for air-gapped enterprise environments.
  - New IPC commands: `updater_status`, `updater_check`, and `updater_verify`.
- **Runtime Capability & Permission Guard (`src-core/src/ipc/router.rs`):**
  - Comprehensive capability matrix tracking `fs`, `shell`, `systemInfo`, `crypto`, `encryptedVault`, `bundler`, `multiWindow`, `plugins`, `notifications`, and `autoUpdater`.
  - Zero-trust enforcement: untrusted frontend cannot elevate capabilities at runtime.
  - New IPC command: `permissions_status`.
- **Frontend Dashboard Extensions (`src-ui/src/App.tsx` & `aegis-bridge.ts`):**
  - 4 new interactive tabs: **Plugin Engine**, **Tray & Notifications**, **Permission Manager**, and **Auto-Updater**.
  - TypeScript client helpers: `pluginList()`, `pluginDispatch()`, `notifySend()`, `trayStatus()`, `updaterStatus()`, `updaterCheck()`, `updaterVerify()`, `permissionsStatus()`.

### 🛡️ Security
- All notification parameters are strictly sanitized against shell escaping.
- Updater forbids automatic execution of unverified update files.
- Plugin handlers execute directly in Rust closures with zero shell breakout risk.

---

## 🚀 [v0.12.0] - 2026-10-25
> **The Standalone Bundler, Installer & Multi-Window Native API Release**

### ✨ Highlights & New Features
- **One-Click Standalone Native `.exe` Bundler (`src-core/src/bundler.rs`):**
  - New `BundleEngine` module packages the compiled Rust binary + React `dist/` + `sandbox/` + `aegis.config.json` into a self-contained portable `dist-bundle/<AppName>/` directory.
  - Auto-generates a Windows **InnoSetup Installer Script** (`.iss`) for producing a native `Setup.exe` installer — no Rust or Node.js required on the end-user machine.
  - Auto-generates a Windows `.bat` launcher script for running the app directly.
  - New CLI flag: `cargo run -- --bundle` to trigger the full packaging pipeline.
  - New npm script: `npm run bundle` — builds Rust release binary + Vite frontend + runs bundler in one command.
- **Enhanced Multi-Window Manager (`src-core/src/window.rs`):**
  - Added `WindowRecord` registry for tracking all spawned secondary windows.
  - New `WindowConfig` fields: `always_on_top` and `kiosk` mode support.
  - Cross-platform launchers for Windows (Edge App Mode), macOS (Safari/open), and Linux (xdg-open).
  - Window ID generation for lifecycle management.
- **New IPC Commands (v0.12):**
  - `window_list` — Returns multi-window capability manifest and platform hint.
  - `bundle_status` — Queries the bundler engine state, existing bundles, and supported targets from inside the app.
- **New "Bundler & Packaging" UI Tab (React Frontend):**
  - 3-step visual bundling guide with interactive code blocks.
  - Live bundler engine status card (queries `bundle_status` IPC command).
  - InnoSetup installer documentation panel with bundle output structure preview.
  - Direct link to Inno Setup Compiler download.
- **`aegis-bridge.ts` v0.12 Updates:**
  - Added `call()` — a generic pass-through method for invoking any custom IPC command.
  - Added `getWindowCapabilities()` — queries multi-window support manifest.
  - Updated `SecurityStatus` type with `bundlerEngineActive` and `multiWindowApiActive` fields.

### 🛡️ Security
- Bundler strictly never exposes host system paths — all bundle paths are derived relative to `project_root`.
- InnoSetup script uses `PrivilegesRequired=lowest` — no admin escalation required.
- Bundler output includes the full `aegis.config.json` capability policy, preserving Zero-Trust settings in deployed apps.

### 🐛 Bug Fixes & Improvements
- `WindowConfig` now includes `always_on_top` and `kiosk` with proper `#[serde(default)]` annotations.
- `main.rs` now prints `v0.12.0` in the startup banner.
- `package.json` bumped to `0.12.0` with new `build:release` and `bundle` npm scripts.

---

## 🚀 [v0.11.0] - 2026-10-25
> **The Cryptographic Hardening, Rate Limiting & Privacy Shield Release**

### ✨ Highlights & New Features
- **Zero-Knowledge Encrypted Vault at Rest (`src-core/src/security/vault.rs`):**
  - Symmetric keystream encryption derived from user passphrases via **10,000 PBKDF2-style SHA-256 iterations**.
  - **HMAC-SHA256 Authentication Tags:** Detects and prevents file tampering, bit flips, or unauthorized file modifications.
  - Storage envelope format: `AEGIS_VAULT_V1:<HMAC>:<CIPHERTEXT>` ensuring on-disk confidentiality.
  - New IPC commands: `vault_encrypt_store` and `vault_decrypt_retrieve`.
- **Sliding-Window Anti-DoS Rate Limiter (`src-core/src/security/ratelimit.rs`):**
  - Protects loopback IPC port `48291` from local fuzzing and malicious burst flooding.
  - Enforces a strict 30 requests/second sliding window limit (triggers **HTTP 429** on violation).
  - **Brute-Force Nonce Lockout:** 5 consecutive invalid nonces activate a progressive 10-second penalty lockout.
- **Air-Gap & Zero-Leakage Privacy Policy Shield (`src-core/src/security/privacy.rs`):**
  - Mathematical zero-telemetry proof: Zero outbound sockets, zero diagnostic harvesting, strict `127.0.0.1` binding.
  - Accessible via `privacy_status` IPC command.
- **New UI Tab: "Privacy & Vault" in React Frontend:**
  - Zero-Knowledge Vault creator: Encrypt and decrypt local `.vault` records with live passphrase validation.
  - On-disk forensics inspector: Compare raw ciphertext with authenticated plaintext output.
  - Air-Gap compliance metrics display (GDPR, HIPAA, Zero-Knowledge verified).
- **Interactive Threat Simulator Upgrade:**
  - Added **Test 4: Anti-DoS Burst Flood Simulation** in the UI dashboard.
- **Automated Penetration Test Suite Enhancements:**
  - Added Test 4: Rate limiter burst flood verification in `aegis-cli`.
  - Added Test 5: Vault tampering & wrong-passphrase rejection test.

---

## 🚀 [v0.10.0] - 2026-10-25
> **Initial Public Open-Source Release**

### ✨ Highlights & New Features
- **High-Performance Rust Core Engine (`src-core/`):**
  - Ultra-lightweight binary (~8MB) and low memory usage (<50MB RAM).
  - High-speed HTTP loopback IPC server running on port `48291`.
  - Windows Edge App Mode fallback window manager.
- **Zero-Trust Security & Capability Engine:**
  - **Canonical Path Traversal Traps:** Traps malicious `../` attempts to climb out of `./sandbox/`.
  - **Shell Execution Revocation:** Command execution (`cmd.exe`, `powershell`, `bash`) is permanently blocked.
  - **Cryptographic Nonce Handshake:** 256-bit SHA-256 session token required for every IPC action.
  - **Origin Whitelist Verification:** Strict validation for `localhost:5173` origins.
- **Modern UI Suite (`src-ui/`):**
  - Built with React 18, TypeScript, Vite, and Lucide icons.
  - Ultra-premium dark glassmorphic design system.
  - **Interactive Attack Simulator:** Live testing dashboard demonstrating blocked path traversal, blocked shell execution, and invalid nonce injections.
  - **Sandboxed File Explorer:** Safely create and read isolated documents in `./sandbox/`.
  - **SHA-256 Cryptographic Hasher:** Live hashing utility.
- **Multi-Framework Compatibility:**
  - Comprehensive integration guides for **React, Vue 3, Svelte 5, SolidJS, Next.js (SSG), and Tailwind CSS**.
- **Automated Security Penetration Testing Suite:**
  - `npm run test:security` CLI script testing attack vectors against the running core.

### 🛡️ Security Hardening
- Enforced Content-Security-Policy (CSP) headers across all IPC responses.
- Implemented isolated `./sandbox/` root directory to prevent access to sensitive host directories.
- Added responsible disclosure policy in [SECURITY.md](SECURITY.md).

### 🐛 Bug Fixes & Refactoring
- Added complete `.gitignore` preventing build artifacts (`target/`, `node_modules/`) from bloating git repositories.
- Updated `sandbox/.gitkeep` ensuring git clones preserve the sandboxed storage root.
- Cleaned up npm scripts and introduced `npm run setup` for one-step installation.

---

## 🏷️ Version History Summary

| Version | Release Date | Key Changes | Stability |
| :---: | :---: | :--- | :---: |
| **[v0.13.0](#-v0130---2026-10-25)** | **2026-10-25** | Rust Plugin Engine, System Tray & OS Notifications, SHA-256 Air-Gapped Updater, Capability Matrix | **Alpha** |
| **[v0.12.0](#-v0120---2026-10-25)** | **2026-10-25** | One-Click Native Bundler & .exe Installer, Multi-Window Manager, `bundle_status` & `window_list` IPC | **Alpha** |
| **[v0.11.0](#-v0110---2026-10-25)** | **2026-10-25** | Zero-Knowledge Encrypted Vault, Anti-DoS Rate Limiter, Air-Gap Privacy Shield | **Alpha** |
| **[v0.10.0](#-v0100---2026-10-25)** | **2026-10-25** | Initial Open-Source Release: Rust core, React UI, Sandbox, Attack Simulator | **Alpha** |
| *v0.20.0* | *Upcoming (Q1 2027)* | macOS WebKit, Hardware Keys, Wasm Sandbox, Multi-Profile Isolation | *Planned* |

---

> 💡 **Have a feature request or found a bug?**  
> Feel free to [Open an Issue](https://github.com/charitha-only/aegis-desktop/issues) or submit a Pull Request following our [Contributing Guide](CONTRIBUTING.md).
