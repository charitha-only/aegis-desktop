# 📖 Aegis Desktop Domain Glossary

Ubiquitous vocabulary and architectural definitions shared between Rust systems core and UI frontends.

---

### Core Domain Concepts

| Term | Domain Definition | Code Anchor |
| :--- | :--- | :--- |
| **Capability** | Explicit, immutable privilege granted to the application runtime via `aegis.config.json`. Untrusted scripts cannot elevate privileges dynamically. | [`src-core/src/security/capabilities.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/security/capabilities.rs) |
| **Session Nonce** | 256-bit cryptographic one-time token negotiated during startup handshake; required on all IPC invocations to prevent CSRF and replay attacks. | [`src-core/src/security/handshake.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/security/handshake.rs) |
| **Encrypted Vault** | Zero-knowledge encrypted-at-rest container using 10,000 PBKDF2/SHA-256 rounds with HMAC-SHA256 authentication tags. | [`src-core/src/security/vault.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/security/vault.rs) |
| **Hardware Key (FIDO2)** | Physical security authenticator adhering to WebAuthn / CTAP2.1 challenge-response semantics requiring physical touch and PIN. | [`src-core/src/security/hardware_key.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/security/hardware_key.rs) |
| **Profile Isolation** | Strictly quarantined sandbox filesystem boundaries per user or workspace (`./sandbox_<id>/`) preventing cross-profile leakage. | [`src-core/src/profile.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/profile.rs) |
| **Wasm VM Sandbox** | Memory-capped (8–16MB), gas-metered guest virtual machine executing untrusted bytecode with strictly isolated host imports. | [`src-core/src/wasm.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/wasm.rs) |
| **Biometric Enclave** | Native OS hardware security module (Windows Hello TPM / Apple Secure Enclave) managing cryptographic keypairs without software exposure. | [`src-core/src/security/enclave.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/security/enclave.rs) |
| **Encrypted IPC Stream** | Authenticated symmetric cipher channel (ChaCha20-Poly1305 / HMAC) wrapping loopback communication against host process sniffing. | [`src-core/src/ipc/encrypted_stream.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/ipc/encrypted_stream.rs) |
| **Local Crash Reporter** | Air-gapped, zero-telemetry panic and assertion recorder storing encrypted crash telemetry strictly in the isolated sandbox. | [`src-core/src/telemetry/crash_report.rs`](file:///c:/Users/user/.gemini/antigravity-ide/scratch/aegis-desktop/src-core/src/telemetry/crash_report.rs) |
