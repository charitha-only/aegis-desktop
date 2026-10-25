# Aegis Desktop

Experimental Rust + React desktop application foundation. The supported window backend is an external browser (Edge app mode on Windows), not an embedded Wry/Tao webview. No security certification or performance multiplier is claimed.

## Supported functionality

- React/TypeScript UI with authenticated loopback IPC, origin checks and request limits.
- Relative sandbox file access, separate profile directories, persistent profile definitions and storage quotas.
- Explicit vault encryption using Argon2id and XChaCha20-Poly1305. Ordinary sandbox files are plaintext.
- Wasmi WebAssembly execution with validation, no host imports, 16 MiB memory and 100,000 fuel limits.
- Ed25519 verification of plugin manifests against locally provisioned public keys.
- Windows portable bundles containing the release binary, UI and configuration. The binary serves the packaged UI without Node or Vite.

## Unavailable integrations

WebAuthn/FIDO2, native biometrics, HSM/PKCS#11, TPM attestation, GPU execution, P2P distribution, encrypted IPC transport and automatic updates are **disabled**. Their endpoints return unavailable instead of synthesizing hardware proofs or successful installs. macOS/Linux native packaging and embedded webviews are not implemented. The network policy screen is an advisory evaluator, not an OS firewall.

## Development

Install a current stable Rust toolchain and Node.js 22 or newer. Run:

~~~sh
npm run setup
npm run dev
~~~

Tests:

~~~sh
cargo test --manifest-path src-core/Cargo.toml
npm --prefix src-ui run build
npm run test:security
~~~

The security smoke test needs a running core. Rust unit tests do not require hardware. Hardware integrations cannot be enabled by changing a status flag.

## Windows distribution

~~~sh
npm run bundle
~~~

This builds a release binary and UI, then creates a fresh portable directory under dist-bundle. Existing bundle directories are rejected to avoid stale assets. Run its Launch batch file. Inno Setup can compile the generated .iss into an installer; generating the script alone does not create Setup.exe. Packaged data is stored under LOCALAPPDATA/AegisDesktop/<bundle-name>, separate from the installation. Do not rename a bundle after using it unless you also migrate its data.

Only Windows packaging is supported. Missing assets, unsupported targets and unsafe package names fail with an error. Validate the generated installer on a clean Windows machine before distributing it.

## Data and compatibility

Vaults use AEGIS_VAULT_V2 with a fresh random salt and nonce per encryption. Legacy V1 envelopes are rejected, never silently rewritten. Preserve backups and use the original version to export old plaintext before encrypting it with the new version. In-memory secrets are ephemeral; use vault files for data that must survive restarts. Profile switching isolates file directories, not operating-system accounts or access by the user running the process.

## Trusted plugin keys

An administrator may provision trusted-signers.json beside the application (never inside the sandbox). It is a JSON array containing key_id, vendor, trusted, and public_fingerprint (the 32-byte Ed25519 public key as hex). There are no default trusted publishers. Sign the exact UTF-8 bytes AEGIS_PLUGIN_V1 + NUL + plugin_id + NUL + manifest; submit key_id:signature_hex. Keep private signing keys outside this repository.

See [SECURITY_FIXES.md](SECURITY_FIXES.md) for verification and remaining limitations. MIT license.
