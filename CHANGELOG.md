# 📜 Aegis Desktop Changelog & Release Notes

All notable changes, new features, security updates, and bug fixes for **Aegis Desktop** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## 🔮 [Upcoming Roadmap] - v0.11.0 / v0.20.0
> **Target Release:** Q4 2026

### 🚀 Planned Features & Enhancements
- [ ] **One-Click Standalone Bundler:** Automatic native `.exe` packaging using InnoSetup / MSI installers without requiring users to install Cargo/Node.
- [ ] **Native Webview Window Wrappers:** Integration with native OS webview APIs (`wry` / `tao` / WebKit on Linux & macOS).
- [ ] **Multi-Window Management:** Open secondary secure windows and modal dialogs via `aegis-bridge.ts`.
- [ ] **Custom Rust Plugin Engine:** Allow developers to easily expose their own custom Rust system functions through capabilities.
- [ ] **Tray & System Notifications:** Background daemon mode with system tray icon support.

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
| **[v0.10.0](#-v0100---2026-10-25)** | **2026-10-25** | Initial Open-Source Release: Rust core, React UI, Sandbox, Attack Simulator | **Alpha** |
| *v0.11.0* | *Upcoming* | Native standalone `.exe` bundler, Multi-window support, Mac/Linux webviews | *Planned* |

---

> 💡 **Have a feature request or found a bug?**  
> Feel free to [Open an Issue](https://github.com/charitha-only/aegis-desktop/issues) or submit a Pull Request following our [Contributing Guide](CONTRIBUTING.md).
