# 🛡️ Aegis Desktop Framework

<div align="center">

[![Version](https://img.shields.io/badge/version-v0.10-blue.svg?style=flat-square&logo=git)](CHANGELOG.md)
[![Changelog](https://img.shields.io/badge/Changelog-What's%20New%20in%20v0.10-purple?style=flat-square)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18%20%7C%2019-blue?logo=react)](https://react.dev/)
[![Vue](https://img.shields.io/badge/Vue.js-3.x-4FC08D?logo=vuedotjs)](https://vuejs.org/)
[![Svelte](https://img.shields.io/badge/Svelte-5.x-FF3E00?logo=svelte)](https://svelte.dev/)
[![Tailwind](https://img.shields.io/badge/Tailwind-CSS-38B2AC?logo=tailwind-css)](https://tailwindcss.com/)
[![Security](https://img.shields.io/badge/Security-Zero--Trust-success)](#-zero-trust-capability-security-model)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**A high-security, ultra-lightweight alternative to Electron for building modern desktop applications.**  
Bridging a high-performance **Rust systems engine** with **any modern web frontend (React, Vue, Svelte, Tailwind)** via zero-trust capability-based IPC.

[What is Aegis?](#-what-is-aegis-desktop) • [⚡ Quick Install](#-quick-install--run-in-30-seconds) • [📜 What's New in v0.10 (Changelog)](CHANGELOG.md) • [Use Cases](#-what-can-you-build-with-aegis) • [Architecture](#-architecture) • [Frontend Compatibility](#-frontend-frameworks--ui-libraries)

</div>

---

## 💡 What is Aegis Desktop?

**Aegis Desktop** is a modern, open-source desktop framework designed to replace heavy, insecure desktop setups. It allows you to build **ultra-fast, native desktop software** using familiar web frontend technologies (**React, Vue, Svelte, Solid, or Tailwind**) backed by a memory-safe, impenetrable **Rust systems core**.

Traditional desktop frameworks like Electron bundle an entire Chromium browser and Node.js runtime with your app. This leads to bloated binary sizes (**150MB+**), heavy memory consumption (**300MB–500MB+ RAM**), and severe security risks—where an XSS vulnerability can grant an attacker full access to run shell commands (`cmd.exe`) on the user's computer.

**Aegis solves these issues by design:**
* 🪶 **Ultra Lightweight:** Minimal ~8MB binary footprint and under 50MB RAM usage.
* 🛡️ **Zero-Trust Security:** Shell execution is permanently blocked, file access is locked inside an isolated sandbox, and every IPC request is cryptographically authenticated.
* ⚡ **Frontend Agnostic:** Use React, Vue, Svelte, or plain HTML/CSS with seamless type-safe IPC communication to Rust.

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

## 📜 Version & Release Notes (`v0.10`)

The current active release is **`v0.10` (Alpha)**.

| Version | Status | Highlights & Changelog |
| :---: | :---: | :--- |
| **`v0.10`** | 🟢 **Current Release** | Initial public open-source release with Rust Core, Zero-Trust Sandbox, Nonce Authentication & Attack Simulator. 👉 [**View Full v0.10 Changelog**](CHANGELOG.md#-v0100---2026-10-25) |
| **`v0.11`** | 🟡 *Next Roadmap* | One-click standalone native `.exe` bundler, multi-window dialog APIs, and Mac/Linux WebKit support. 👉 [**View Upcoming Features**](CHANGELOG.md#-upcoming-roadmap---v0110--v0200) |

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
