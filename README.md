# 🛡️ Aegis Desktop Framework

<div align="center">

[![Version](https://img.shields.io/badge/version-v0.23-blue.svg?style=flat-square&logo=git)](CHANGELOG.md)
[![Changelog](https://img.shields.io/badge/Changelog-What's%20New%20in%20v0.23-purple?style=flat-square)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.89%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18%20%7C%2019-blue?logo=react)](https://react.dev/)
[![Vue](https://img.shields.io/badge/Vue.js-3.x-4FC08D?logo=vuedotjs)](https://vuejs.org/)
[![Svelte](https://img.shields.io/badge/Svelte-5.x-FF3E00?logo=svelte)](https://svelte.dev/)
[![Tailwind](https://img.shields.io/badge/Tailwind-CSS-38B2AC?logo=tailwind-css)](https://tailwindcss.com/)
[![Security](https://img.shields.io/badge/Security-Zero--Trust-success)](#-zero-trust-capability-security-model)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**A high-security, ultra-lightweight alternative to Electron for building modern desktop applications.**<br>
Bridging a high-performance **Rust systems engine** with **any modern web frontend (React, Vue, Svelte, Tailwind)** via zero-trust capability-based IPC.

[What is Aegis?](#-what-is-aegis-desktop) • [⚡ Quick Install](#-quick-install--run-in-30-seconds) • [📜 What's New in v0.23 (Changelog)](CHANGELOG.md) • [Use Cases](#-what-can-you-build-with-aegis) • [Architecture](#-architecture) • [Frontend Compatibility](#-frontend-frameworks--ui-libraries)

</div>

> **v0.23.0 status:** This README preserves the original project presentation. Current functionality and limitations are documented in [Security fixes and verification](SECURITY_FIXES.md) and the [v0.23 changelog](CHANGELOG.md). Hardware authentication, HSM/PKCS#11, TPM, biometrics, GPU, P2P, encrypted IPC and auto-updates are unavailable; native macOS/Linux webviews and packages remain unimplemented. Earlier descriptions and performance figures below describe prototype goals and are not verified production guarantees. Windows uses an external browser backend. Vault encryption now uses Argon2id + XChaCha20-Poly1305; legacy V1 migration needs the original version.


---

## 💡 What is Aegis Desktop?

**Aegis Desktop** is a modern, open-source desktop framework designed to replace heavy, insecure desktop setups. It allows you to build **ultra-fast, native desktop software** using familiar web frontend technologies (**React, Vue, Svelte, Solid, or Tailwind**) backed by a memory-safe, impenetrable **Rust systems core**.

Traditional desktop frameworks like Electron bundle an entire Chromium browser and Node.js runtime with your app. This leads to bloated binary sizes (**150MB+**), heavy memory consumption (**300MB–500MB+ RAM**), and severe security risks—where an XSS vulnerability can grant an attacker full access to run shell commands (`cmd.exe`) on the user's computer.

**Aegis solves these issues by design:**
* 🪶 **Ultra Lightweight:** Minimal ~8MB binary footprint and under 50MB RAM usage.
* 🛡️ **Zero-Trust Security:** Shell execution is permanently blocked, file access is locked inside an isolated sandbox, and every IPC request is cryptographically authenticated.
* 🔐 **Encrypted Vault at Rest:** Sensitive records encrypted with 10,000 PBKDF2/SHA-256 iterations & HMAC-SHA256 authentication tags.
* ⚡ **Anti-DoS Sliding Rate Limiter:** Protects the IPC bridge from local fuzzing and brute-force nonce floods.
* 📦 **One-Click Standalone Bundler:** Package into a portable `.exe` installer with InnoSetup — zero Rust/Node required on end-user machines.
* 🌐 **Frontend Agnostic:** Use React, Vue, Svelte, or plain HTML/CSS with seamless type-safe IPC communication to Rust.

---

## ⚡ Quick Install & Run (in 30 Seconds)

Getting started with Aegis is fast and straightforward:

```bash
# 1. Clone the repository
git clone https://github.com/charitha-only/aegis-desktop.git
cd aegis-desktop

# 2. Install all dependencies (Root + Frontend UI)
npm run setup

# 3. Launch the development environment (Rust Security Core + UI)
npm run dev
```

> 📋 **Prerequisites:** Make sure you have [Node.js](https://nodejs.org/) (v18+) and [Rust & Cargo](https://www.rust-lang.org/tools/install) (v1.75+) installed on your machine.

---

## 📜 Version & Release Notes (`v0.23`)

The current version is **`v0.23.0` (Experimental)**.

| Version | Status | Highlights & Changelog |
| :---: | :---: | :--- |
| **`v0.23`** | 🟢 **Current Version** | Previous-version upgrade and reinstall recovery checks, real Windows lock testing in a disposable hosted runner, and exact-commit validation evidence. External audit, signing credentials and physical-device suspend/resume remain gated. No application or hardware provider is added. See [security review gates](SECURITY_REVIEW.md). |
| **`v0.22`** | ⚪ **Previous Version** | Native Windows lifecycle revocation, signing setup and disposable installer checks. |
| **`v0.21`** | ⚪ **Previous Version** | Authenticated vault sessions, idle/absolute expiry, credential cleanup and profile-change revocation. |
| **`v0.20`** | ⚪ **Previous Version** | Authenticated atomic vault updates, encrypted backup restore, password rotation, secret-buffer cleanup and restricted packaged IPC. |
| **`v0.19`** | ⚪ **Previous Version** | Security and correctness fixes: bounded wasmi runtime, authenticated V2 vaults, Ed25519 manifest verification, hardened IPC/filesystem access, persistent profiles and repaired Windows portable packaging. Unimplemented integrations fail closed. See [verification and limitations](SECURITY_FIXES.md). |
| **`v0.18`** | ⚪ **Previous Prototype** | **Multi-OS Pipeline, P2P Marketplace & HSM PKCS#11 Release:** Automated Multi-OS Packaging Pipeline for macOS DMG, Linux AppImage/Debian, and Windows Portable; Distributed P2P Plugin Marketplace with content-addressed IPFS CIDs and decentralized DID multi-sig consensus; Hardware Security Module (HSM) native PKCS#11 driver for YubiHSM 2 and smart cards with non-extractable keys and on-chip signing. 👉 [**View Full v0.18 Changelog**](CHANGELOG.md) |
| **`v0.17`** | ⚪ *Previous Release* | Native Platform & GPU Acceleration Release: macOS & Linux native WebKit integration via Wry (0.47) + Tao (0.30); TPM 2.0 Remote Attestation Engine with PCR 0/7/10/11 cryptomarkers and AIK quote signing; WGPU-backed GPU-accelerated Wasm JIT compiler (12x–20x speedup). |
| **`v0.16`** | ⚪ *Previous Release* | Network Firewall Policy Engine with domain-pattern rules, Zero-Trust connection evaluation, Plugin Signature & Provenance verification with cryptographic manifest digests and trusted root authority registry. |
| **`v0.15`** | ⚪ *Previous Release* | Local Crash Reporter with structured crash entries and severity classification; Zero-Knowledge Secrets Manager with PBKDF2 key derivation, HMAC-authenticated secret storage, and master-password retrieval. |
| **`v0.14`** | ⚪ *Previous Release* | Hardware Keys & Virtual Machine Sandbox Release: Physical Security Key (FIDO2 / WebAuthn Level 3) challenge-response authentication, Multi-Profile isolated workspaces with directory sandboxes, and WebAssembly (Wasm) guest VM with gas metering. 👉 [**View v0.14 Notes**](CHANGELOG.md#-v0140---2026-10-25) |
| **`v0.13`** | ⚪ *Previous Release* | Extensibility & Capabilities Release: Custom Rust Plugin Engine with built-in telemetry & checksum tools, System Tray & OS Notifications via WinRT/PowerShell, Air-Gapped SHA-256 verified Auto-Updater, and runtime Capability Matrix. 👉 [**View v0.13 Notes**](CHANGELOG.md#-v0130---2026-10-25) |
| **`v0.12`** | ⚪ *Previous Release* | Standalone Bundler & Multi-Window Release: One-click native `.exe` bundler & InnoSetup packager, Multi-Window Manager with window registry, new `bundle_status` & `window_list` IPC commands. 👉 [**View v0.12 Notes**](CHANGELOG.md#-v0120---2026-10-25) |
| **`v0.11`** | ⚪ *Previous Release* | Cryptographic Hardening: Zero-Knowledge Encrypted Vault (PBKDF2/HMAC), Anti-DoS Sliding Window Rate Limiter, Air-Gap privacy shield. 👉 [**View v0.11 Notes**](CHANGELOG.md#-v0110---2026-10-25) |
| **`v0.10`** | ⚪ *Previous Release* | Initial open-source release with Rust core engine, sandbox filesystem, and React/Vite UI. 👉 [**View v0.10 Notes**](CHANGELOG.md#-v0100---2026-10-25) |
| **`v0.24`** | 🟡 *Next Roadmap (Q1 2027)* | Decentralized Zero-Knowledge Identity (DID/VC), Post-Quantum Lattice Cryptography (ML-KEM / ML-DSA), Ephemeral Memory Sanitization. 👉 [**View Upcoming Roadmap**](CHANGELOG.md) |

👉 **[Click here to view all Version Updates, Bug Fixes, and Roadmap in CHANGELOG.md](CHANGELOG.md)**

---

## 🎯 What Can You Build with Aegis?

With Aegis Desktop, developers can build any desktop application requiring **modern UI, instant responsiveness, and hardened data protection**:

| Application Type | Why Aegis is the Ideal Framework |
| :--- | :--- |
| 🔐 **Encrypted Vaults & Password Managers** | Sensitive data remains strictly quarantined inside an isolated sandbox folder that malicious external scripts cannot touch. |
| 🪙 **Crypto, Web3 & FinTech Desktop Tools** | The revoked shell execution policy and SHA-256 session handshake guarantee that malicious third-party dependencies cannot inject arbitrary commands. |
| 📝 **Private Local-First Notes & Documents** | Build responsive offline markdown editors and personal knowledge vaults with zero memory lag and instant startup times. |
| 📊 **Enterprise Dashboards & Admin Tools** | Safely display internal system diagnostics and data without exposing operating-system-level vulnerabilities. |
| 🛠️ **Developer Utilities & Native GUIs** | Fast, reactive tooling with a fraction of the CPU and memory footprint of Electron apps. |

---

## 🌟 Key Highlights

Aegis is an open-source enterprise desktop architecture engineered with **defense-in-depth**:

1. **Rust Core Engine (`src-core/`)**
   - High performance, memory-safe, minimal binary footprint (~8MB vs ~150MB Electron).
   - Fast compile times on Windows via the native GNU toolchain.
   - Built-in sandboxed file system manager.

2. **Zero-Trust Capability Security Model**
   - **No Arbitrary Execution:** Shell command execution (`cmd.exe`, `powershell`, `bash`) is permanently blocked by default policy.
   - **Path Traversal Defense:** Canonicalization algorithms prevent `../` directory climbs outside the isolated `./sandbox/` root.
   - **Cryptographic Nonce Handshake:** Every IPC request is authenticated with a 256-bit SHA-256 session token to eliminate CSRF and replay attacks.
   - **Origin Isolation & Strict CSP:** Only whitelisted origins (`http://localhost:5173`, `http://127.0.0.1:5173`) can communicate across the bridge.

3. **Frontend Freedom (`src-ui/`)**
   - **Agnostic Architecture:** Ships with React 18 + Vite by default, but completely framework-agnostic.
   - Works with **React, Vue, Svelte, Solid, Next.js (SSG), Tailwind, and shadcn/ui**.
   - Built-in **Threat Testing & Attack Simulator** showcasing defensive traps in real time.

4. **One-Click Native Bundler (`src-core/src/bundler.rs`)**
   - Packages Rust binary + React frontend + config into a portable distribution folder.
   - Auto-generates InnoSetup `.iss` script for native Windows `Setup.exe` installer.
   - New `npm run bundle` one-command packaging pipeline.

---

## 🏗️ Architecture

```
+-------------------------------------------------------------+
|                     Aegis Desktop App                       |
|                                                             |
|  +-------------------------------------------------------+  |
|  |       Any Frontend (React / Vue / Svelte / Solid)      |  |
|  +-------------------------------------------------------+  |
|                             │                               |
|               window.__IPC__ / aegis-bridge                 |
|                             │                               |
|        [ Cryptographic Nonce Handshake: SHA-256 ]           |
|        [ Origin Whitelist Verification           ]           |
|                             ▼                               |
|  +-------------------------------------------------------+  |
|  |                Rust Core Engine (src-core/)           |  |
|  |  • Capability Security Engine (Policy Enforcement)    |  |
|  |  • Sandboxed Storage Engine (./sandbox/)              |  |
|  |  • Cryptographic & System Diagnostics Services        |  |
|  +-------------------------------------------------------+  |
+-------------------------------------------------------------+
```

---

## 🎨 Supported Frontend Frameworks & UI Libraries

Aegis connects to frontend code over a standard, type-safe HTTP/WebSocket Loopback IPC bridge. This means **you can use any modern web framework and UI library of your choice**!

### 1. Framework Compatibility Matrix

| Framework / Tool | Status | Template / Support |
| :--- | :---: | :--- |
| **React 18 / 19** | :white_check_mark: **Default** | Pre-configured in `src-ui/` with Vite & TypeScript |
| **Vue 3** | :white_check_mark: Supported | Drop-in compatible with `@vitejs/plugin-vue` |
| **Svelte 5 / SvelteKit** | :white_check_mark: Supported | Compatible via `@sveltejs/vite-plugin-svelte` or static adapter |
| **Solid.js** | :white_check_mark: Supported | High-speed fine-grained reactivity via `vite-plugin-solid` |
| **Next.js** | :white_check_mark: Supported | Supported via Next.js Static HTML Export (`output: 'export'`) |
| **Vanilla HTML/TS** | :white_check_mark: Supported | Zero dependencies, ultra-lightweight Vite vanilla template |

### 2. UI Component Libraries & Styling

* **Tailwind CSS:** Fully compatible for rapid, responsive UI engineering.
* **shadcn/ui & Radix UI:** Accessible, customizable primitives with zero lock-in.
* **Lucide Icons / Heroicons:** Consistent, crisp SVG icons.
* **Mantine / Chakra UI / Ant Design / MUI:** Enterprise-grade component suites.

---

## 💻 How to Use Aegis with Different Frontends

The Aegis Bridge ([`aegis-bridge.ts`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-ui/src/aegis-bridge.ts)) is written in pure TypeScript, making it easily consumable in any framework:

### ⚛️ Using in React (Custom Hook)

```tsx
import React, { useEffect, useState } from 'react';
import { aegis } from './aegis-bridge';

export function SecureNotes() {
  const [content, setContent] = useState('');
  const [status, setStatus] = useState('Connecting...');

  useEffect(() => {
    // 1. Initialize Cryptographic Handshake
    aegis.init().then((res) => {
      if (res.success) setStatus('Core Connected');
    });
  }, []);

  const handleSave = async () => {
    // 2. Safe Sandboxed File Write
    const res = await aegis.writeSandboxFile('note.txt', content);
    if (res.success) alert('Saved securely to sandbox!');
  };

  return (
    <div className="p-6 bg-slate-900 text-white rounded-xl">
      <p className="text-sm text-emerald-400">{status}</p>
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder="Type private notes..."
        className="w-full p-3 rounded bg-slate-800 border border-slate-700"
      />
      <button onClick={handleSave} className="mt-4 px-4 py-2 bg-indigo-600 rounded">
        Save to Aegis Vault
      </button>
    </div>
  );
}
```

---

### 💚 Using in Vue 3 (Composition API)

```vue
<template>
  <div class="card">
    <h2>Aegis + Vue 3</h2>
    <p>Status: {{ isConnected ? 'Connected' : 'Offline' }}</p>
    <button @click="readVault">Read Sandbox File</button>
    <pre>{{ fileData }}</pre>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { aegis } from './aegis-bridge';

const isConnected = ref(false);
const fileData = ref('');

onMounted(async () => {
  const init = await aegis.init();
  isConnected.value = init.success;
});

async function readVault() {
  const res = await aegis.readSandboxFile('note.txt');
  if (res.success && res.data) {
    fileData.value = res.data.content;
  }
}
</script>
```

---

### 🧡 Using in Svelte 5

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { aegis } from './aegis-bridge';

  let hashInput = $state('Secret Message');
  let sha256 = $state('');

  onMount(async () => {
    await aegis.init();
  });

  async function calculateHash() {
    const res = await aegis.hashInput(hashInput);
    if (res.success && res.data) {
      sha256 = res.data.hash;
    }
  }
</script>

<main>
  <input bind:value={hashInput} />
  <button onclick={calculateHash}>Compute SHA-256</button>
  <p>Hash: {sha256}</p>
</main>
```

---

### 🌊 Adding Tailwind CSS & shadcn/ui

To use **Tailwind CSS** in your `src-ui/` directory:

```bash
cd src-ui
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

Add the template paths in `tailwind.config.js`:
```javascript
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx,vue,svelte}"],
  theme: { extend: {} },
  plugins: [],
}
```

Import Tailwind directives in `src-ui/src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

---

## 📋 Prerequisites

Before running Aegis, ensure you have the following installed:
* [Node.js](https://nodejs.org/) (v18.0 or newer)
* [Rust & Cargo](https://www.rust-lang.org/tools/install) (v1.75 or newer)

---

## 🚀 Getting Started

### 1. Clone & Setup Dependencies
```bash
git clone https://github.com/charitha-only/aegis-desktop.git
cd aegis-desktop

# Install both root and UI dependencies
npm run setup
```

### 2. Start Development Mode
Launches the Rust Security Core and Vite dev server simultaneously:
```bash
npm run dev
```

Alternatively, run each process in separate terminal windows:
* **Rust Core:** `npm run dev:core`
* **Frontend UI:** `npm run dev:ui`

### 3. Automated Security Penetration Tests
Verify that the Aegis core correctly catches and traps unauthorized attacks:
```bash
npm run test:security
```

---

## 🔒 Security Configuration (`aegis.config.json`)

You can configure origins (e.g. if running Next.js on port `3000` or Vue on port `5173`), capabilities, and window dimensions:

```json
{
  "name": "Aegis Desktop Application",
  "security": {
    "enableCapabilityCheck": true,
    "enforceOrigin": true,
    "allowedOrigins": [
      "http://localhost:5173",
      "http://127.0.0.1:5173"
    ],
    "capabilities": {
      "fs": {
        "allowRead": ["./sandbox/**"],
        "allowWrite": ["./sandbox/**"],
        "denyPaths": ["../**", "C:\\Windows\\**"]
      },
      "system": {
        "allowSystemInfo": true,
        "allowShellExecution": false
      }
    }
  }
}
```

---

## 📁 Repository Structure

```
aegis-desktop/
├── .github/              # Issue and PR templates for open-source contributors
├── aegis-cli/            # CLI launcher & automated security test suite
├── sandbox/              # Isolated, capability-protected storage folder
├── src-core/             # Rust systems backend (Server, Router, Security engine)
│   ├── src/
│   │   ├── ipc/          # IPC message routing
│   │   ├── security/     # Capabilities, nonce handshakes, origin validators
│   │   ├── main.rs       # Core entrypoint
│   │   └── server.rs     # Loopback IPC bridge
├── src-ui/               # Modern Frontend (React 18 + TypeScript + Vite)
│   └── src/
│       ├── aegis-bridge.ts # Type-safe TypeScript IPC client (usable across all frameworks)
│       └── App.tsx       # Live security dashboard & threat simulator
├── aegis.config.json     # Declarative security & window settings
├── CONTRIBUTING.md       # Contribution guidelines
├── SECURITY.md           # Vulnerability disclosure policy
└── LICENSE               # MIT License
```

---

## 🛡️ Security Vulnerabilities

Please review our [Security Policy](SECURITY.md) to report security issues or vulnerability findings responsibly.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE) - open and free for both personal and commercial use.
