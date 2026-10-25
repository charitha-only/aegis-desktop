# Independent security audit handoff

Status: pending external reviewer. This document and automated tests are project-authored evidence, not independent certification. No external audit was commissioned or completed here.

## Scope for a separate reviewer

- Vault V2: Argon2id parameter/version compatibility, XChaCha20-Poly1305 authentication, nonce/salt generation, password changes, plaintext/key buffer lifecycle and error paths.
- Loopback IPC: Host/Origin, process token vs vault-session token, parser limits, rate/cooldown behavior, CSP/no-store, bearer-token handling and browser threats.
- Session lifecycle: one vault/profile scope, forged token rejection, re-unlock/mutation/profile revocation, idle/absolute limits, in-flight Windows event revocation, failure of notification registration or monitor shutdown.
- Storage: traversal/symlink/device-name checks, quotas, atomic replacement and OS locks, encrypted backup recovery, legacy migration refusal, concurrency and crash/power-loss assumptions.
- Supply chain/distribution: dependency provenance, publisher certificate custody, trusted timestamps, installer payload signatures, upgrades/uninstall data retention and the disabled updater.

## Reproduce evidence

Use a disposable Windows machine, check out the exact commit under review and record its hash. Run `cargo test --locked --manifest-path src-core/Cargo.toml`, `npm ci --prefix src-ui`, `npm run bundle`, and both smoke suites against separate fresh isolated runtimes. Normal GitHub CI additionally compiles/exercises/uninstalls the installer. Never use real password vaults for destructive/fuzz/failure tests.

Review SECURITY_REVIEW.md, SECURITY_FIXES.md and WINDOWS_RELEASE.md. Test actual Windows lock/sleep/hibernate/remote-session transitions; synthetic window messages prove handler wiring only. Evaluate any noninteractive WTS behavior explicitly instead of bypassing it. Examine error paths and concurrently arriving requests/events, including monitor death while a KDF is running. Fuzz untrusted envelope/IPC/Wasm inputs in isolated processes and inspect for secret-bearing logs and crash artifacts.

## Required external report

Record reviewer identity/independence, reviewed commit hash, OS/build/device details, threat model, methods/tools, severity-ranked findings with reproductions, remediation commits and retest results. Record unresolved limitations and excluded scope. Distribution of real credentials remains gated on this review and real-device acceptance; a passing project test suite does not close that gate.
