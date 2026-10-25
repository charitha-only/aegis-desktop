# Security corrections

## Implemented

- Removed fabricated hardware keys, token slots, PIN authentication, TPM proofs, biometric success, GPU benchmarks and marketplace installation acknowledgements. Unavailable endpoints fail closed.
- Replaced custom Wasm parsing with wasmi validation/execution, bounded stores and fuel. GPU requests return unavailable rather than changing program semantics.
- Replaced deterministic XOR vault encryption with randomized Argon2id + XChaCha20-Poly1305 V2 envelopes. Wrong passwords and modified envelopes fail authentication; V1 remains untouched and is explicitly unsupported.
- Removed hard-coded default secrets. In-memory secret values use the authenticated vault format.
- Replaced publisher-name signature acceptance with strict Ed25519 verification over a domain-separated plugin identity and exact manifest bytes.
- Session tokens use OS randomness. IPC validates Host and Origin, rejects untrusted preflights and limits request bodies to 1 MiB.
- Filesystem paths reject absolute paths, traversal, alternate-stream syntax, ambiguous names and linked components. Profile quotas are checked before file/vault writes. This is an application boundary; it is not protection from another process with the same OS account changing files concurrently.
- Profile IDs are validated; custom profiles persist on disk. No filesystem glob outside the active sandbox is granted.
- Runtime status and unavailable feature pages no longer advertise fabricated providers.
- Windows bundling requires release/UI/config assets, includes a local static UI server, rejects output reuse and uses per-user installation/data locations. CLI failures must be treated as errors.

## Verification

Run cargo test --manifest-path src-core/Cargo.toml and npm --prefix src-ui run build. Regression tests cover bytecode validation, branch continuation, unreachable/fuel/memory traps, import rejection, unavailable providers, signature tampering, password authentication, randomized encryption, path rejection, quotas and profile persistence.

## Remaining integration work

Real WebAuthn registration/assertion verification, vendor PKCS#11 providers, platform TPM/biometric APIs, native macOS/Linux webviews and packages, GPU computation, P2P transport and publisher provisioning require separate implementations. They are disabled, not claimed complete. Encrypted transport and updater are unavailable. A clean-machine installer test and real-device tests remain required before production release. This change is not an independent security audit or certification.

## v0.19.0 validation record

- 16 Rust regression tests passed locally.
- Production UI and optimized core builds passed.
- Nine live API security smoke checks passed against a packaged binary.
- Packaged HTML and JavaScript assets served without Node/Vite; unauthorized-origin preflight rejected.
- File and custom profile persisted after process restart; cross-profile file read rejected.
- npm audit reported zero known vulnerabilities after the Vite/React-plugin upgrade.
- Generated Inno Setup script was not compiled or tested on a clean Windows installation. Hardware and non-Windows providers remain unavailable.

## v0.20.0 follow-up

See [sensitive-data framework review gates](SECURITY_REVIEW.md) for atomic authenticated vault replacement, encrypted backup/restore, password rotation, secret-buffer cleanup, strict packaged IPC and the remaining production-release requirements. This remains a framework, not a password-manager application.

Local v0.20 validation: 24 Rust regression tests passed; optimized core and production UI builds passed; the isolated portable runtime passed the sensitive smoke suite (authenticated overwrite/backup/rotation, no-cache responses, strict CSP/origins, reserved names and profile secret clearing). Windows CI now repeats the portable sensitive smoke suite. This does not replace the pending independent review described in SECURITY_REVIEW.md.


## v0.21.0 scoped session lifecycle

Added authenticated vault-scoped unlock/read/lock interfaces, random tokens, idle/absolute expiry, background credential cleanup and revocation on vault mutations/profile switches. Core dispatch is serialized against lock/profile changes. Password-per-operation interfaces remain fresh-authentication paths. This provides a framework building block, not native OS login or a completed password manager. See SECURITY_REVIEW.md for limits and release gates.
