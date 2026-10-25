# 🛡️ Aegis Desktop Framework

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18-blue?logo=react)](https://react.dev/)
[![Vite](https://img.shields.io/badge/Vite-5.0-646CFF?logo=vite)](https://vitejs.dev/)
[![Security](https://img.shields.io/badge/Security-Zero--Trust-success)](#-zero-trust-capability-security-model)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**A high-security, lightweight cross-platform desktop framework.**  
Bridging a high-performance **Rust systems engine** with a reactive **React/Vite frontend** via zero-trust capability-based IPC.

[Key Highlights](#-key-highlights) • [Architecture](#-architecture) • [Getting Started](#-getting-started) • [Building Your App](#-building-your-own-app) • [Security Policy](SECURITY.md) • [Contributing](CONTRIBUTING.md)

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

3. **Modern Web Frontend (`src-ui/`)**
   - Built with **React 18**, **TypeScript**, and **Vite**.
   - Ultra-premium dark glassmorphic design system.
   - Built-in **Threat Testing & Attack Simulator** showcasing defensive traps in real time.

---

## 🏗️ Architecture

```
+-------------------------------------------------------------+
|                     Aegis Desktop App                       |
|                                                             |
|  +-------------------------------------------------------+  |
|  |             Frontend UI (React 18 + TS + Vite)         |  |
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

## 🎨 Building Your Own App

You can customize this starter framework to build any desktop software:

1. **Add Your UI Components:**  
   Replace the demo components in `src-ui/src/App.tsx` with your custom React pages and tools.
2. **Interact with the Core via `aegis-bridge.ts`:**
   ```typescript
   import { aegis } from './aegis-bridge';

   // Securely write to isolated sandbox
   await aegis.writeSandboxFile('user_config.json', JSON.stringify(data));

   // Read safely from sandbox
   const res = await aegis.readSandboxFile('user_config.json');

   // Compute SHA-256 hashes
   const hashRes = await aegis.hashInput('my-secret-token');
   ```
3. **Configure App Manifest (`aegis.config.json`):**
   Customize window dimensions, titles, allowed origins, and granular capabilities.

---

## 🔒 Security Configuration (`aegis.config.json`)

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
├── src-ui/               # React 18 + Vite frontend
│   └── src/
│       ├── aegis-bridge.ts # Type-safe TypeScript IPC client
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
