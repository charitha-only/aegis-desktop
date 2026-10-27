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


## Additional v0.23 evidence for reviewers

Download the exact run's `windows-validation-evidence` artifact and compare its `commit` with the source under review. Installer evidence includes actual v0.22 upgrade, uninstall/reinstall ciphertext preservation and authenticated backup restoration. Lock evidence comes from a real `LockWorkStation` call in the disposable hosted runner, not from injected messages. It explicitly records real suspend/resume as not run and independent audit as pending. Review the guarded test scripts before reproducing them; do not enable a workstation lock test on a developer or production machine.

Publisher-signature verification was checked against the immutable, digest-pinned, upstream-signed Inno Setup compiler installer as a positive fixture, plus unsigned/mismatched identity rejection. Aegis signing still needs its own publisher certificate; no synthetic identity was substituted. Actions are pinned, but reviewers should still examine dependency and signing-environment provenance.


## Known-advisory scan provenance

Local 2026-10-26 scan: cargo-audit 0.22.2 checked 94 lockfile dependencies against RustSec database commit `21d909e4c8bd4e14c7aea58bf980c881a0053ab1` (1,352 advisories), reporting zero known vulnerabilities and no warnings. This is a point-in-time database result, not proof that dependencies or the application have no flaws. CI repeats RustSec and npm checks with a digest-pinned RustSec executable and uploads their JSON results with commit and lockfile hashes; failed scans fail the job.


## v0.25.14 local HTTP transport patch

Review the local tiny_http fork in `src-core/vendor/tiny_http`, especially shared
socket-clone deadlines, permit lifetime through queued requests, response-close
semantics, header limits and rejected-body destruction. `AEGIS_PATCH.md` records
the upstream archive checksum and modified files. Known-advisory scanning does
not audit local patches. The regression `aegis-cli/bin/ipc-deadline-smoke.js`
uses an owned ephemeral process and tests incomplete/trickling input, rejected
oversized bodies, oversized headers and connection admission/recovery. Review
`docs/IPC_TRANSPORT_LIMITS.md` and source `39e3af6`. Long-running command deadlines,
sustained-flood availability, physical resource ceilings and an independent
review remain open; this project-authored fix does not close those gates.
