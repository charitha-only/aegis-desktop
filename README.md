# 🛡️ Aegis Desktop Framework

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18%20%7C%2019-blue?logo=react)](https://react.dev/)
[![Vue](https://img.shields.io/badge/Vue.js-3.x-4FC08D?logo=vuedotjs)](https://vuejs.org/)
[![Svelte](https://img.shields.io/badge/Svelte-5.x-FF3E00?logo=svelte)](https://svelte.dev/)
[![Tailwind](https://img.shields.io/badge/Tailwind-CSS-38B2AC?logo=tailwind-css)](https://tailwindcss.com/)
[![Security](https://img.shields.io/badge/Security-Zero--Trust-success)](#-zero-trust-capability-security-model)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**A high-security, lightweight cross-platform desktop framework.**  
Bridging a high-performance **Rust systems engine** with **any modern frontend framework** via zero-trust capability-based IPC.

[Key Highlights](#-key-highlights) • [Architecture](#-architecture) • [Frontend Compatibility](#-frontend-frameworks--ui-libraries) • [Getting Started](#-getting-started) • [Building Your App](#-building-your-own-app) • [Security Policy](SECURITY.md) • [Contributing](CONTRIBUTING.md)

</div>

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
