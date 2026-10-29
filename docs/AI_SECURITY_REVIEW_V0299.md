# Scoped AI security review — v0.29.9

Date: 2026-10-30 (Asia/Colombo). Project-authored AI review, not an external independent audit or certification.

## Scope and method

A separate AI agent reviewed baseline 7ea2a195e038f625ba4ddb1dbe7c2a2343d5c223 read-only: filesystem capability resolution, listing/storage, IPC file commands, runtime SDK shapes and Windows signing/installer scripts/workflow. The primary agent reproduced the candidate through authenticated public IPC using only owned temporary data, then applied a regression-first fix. No third party was contacted, publisher credentials provisioned, or workstation lock/suspend performed.

## F-001: mixed-case internal storage names accepted

Priority: Medium (project triage, no independent severity assessment). Confirmed reserved-name policy bypass; concurrent corruption exploitation remains unverified.

Location: src-core/src/security/capabilities.rs:231, public regression src-core/src/security/regression_tests.rs:1940. The old case-sensitive starts_with check rejected .aegis- names but accepted .AEGIS-/.AeGiS- variants. On a case-insensitive filesystem these can alias the framework .aegis-write.lock file. A caller already holding the IPC token and file-write capability could address the internal name through ordinary file operations. This is not demonstrated remote unauthenticated execution or a vault decryption bypass.

Red: the public IPC regression first wrote ordinary.txt to initialize the framework lock, then attempted fs_write_file on .AEGIS-write.lock. Before the patch the request succeeded, failing the assertion that internal-name writes must be rejected. No real application data was used.

Green: check the ASCII-lowercased component prefix on every platform. The regression also seeds existing mixed-case internal files, verifies both read and write fail with a reserved-name error (including a nested component), and confirms ordinary file access still succeeds. The strengthened regression passes. The local Windows GNU Rust suite passed 98 tests, with one ignored. Exact-source portable/native/installer CI status is tracked separately in IMPLEMENTATION_STATUS.md.

Reproduce: cargo test --locked --manifest-path src-core/Cargo.toml ipc_filesystem_rejects_case_variants

Impact limits: no concurrent-writer data-loss reproduction, no proof about malicious equivalent-account filesystem actors, and no broad filesystem-race closure. The intentional portable policy now rejects ASCII case variants even on case-sensitive filesystems. Directory listing filtering is unchanged.

## Other observations and open gates

No additional confirmed defect was found in the bounded scope. This is not a claim of absence of vulnerabilities. Signing requires a real publisher identity; workflow checks cannot establish an unavailable Aegis signature. Physical Linux/macOS, interactive Windows lifecycle/UI, constrained-device performance and power-loss acceptance remain incomplete. A separate qualified reviewer must supply an identity, scope, report and retest evidence to complete the independent-audit gate.
