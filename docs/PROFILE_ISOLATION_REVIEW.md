# Profile isolation review — 2026-10-26

## Scope and result

Reviewed the public IPC profile creation/switching behavior and persisted-profile loading against the v0.23 baseline (`b6439da6ff25bd9dd99bd80a4bfaa7c60a3101b3`). Found and fixed one logical isolation defect. This is a focused project-authored review, not a completed independent audit or exhaustive penetration test.

## Finding: distinct profiles could share storage

Severity: Medium within the framework's logical profile-isolation boundary. The caller already requires access to local IPC. Profiles are not separate OS accounts and this finding does not demonstrate bypass of encrypted-vault passphrase authentication.

Affected code: `src-core/src/profile.rs`, `ProfileManager::create_profile` and saved-profile loading in `ProfileManager::new`.

Root cause: custom folder names were derived as `sandbox_<id>` while only exact profile IDs were checked for uniqueness. The built-in `private_vault` profile already uses `sandbox_vault`, so the permitted custom ID `vault` selected the same directory. `Work` and `work` also refer to the same directory on commonly used case-insensitive filesystems. Previously saved entries could recreate either collision at startup.

Impact: applications could present two profiles as isolated while sharing files, causing unintended disclosure of unencrypted profile data or conflicting writes. Ciphertext files still require their authenticated vault operations to decrypt; no cryptographic bypass was established.

Remediation: reject case-insensitive duplicate profile IDs and storage-directory assignments before creating a custom profile. Apply the same uniqueness rules while loading saved profiles, preserving the built-in entries and the first accepted distinct custom entry. Existing data is not deleted or automatically moved. A rejected legacy profile may need manual data reconciliation; directory ownership cannot safely be inferred after an alias has already been used.

## Regression evidence

- `ipc_profiles_reject_shared_storage_identities`: failed on the baseline when `vault` was accepted; passes after the fix. Also checks rejection of case-insensitive custom aliases.
- `ipc_restart_ignores_colliding_saved_profiles`: loads a legacy registry fixture through the real router, checks the profile list and switching behavior. Removing only the saved-entry collision guard yields five profiles instead of the expected three; restoring it passes.
- Tests use temporary fixture directories, the real IPC router and public commands. They do not access the user's application data or trigger real OS lock/suspend.

Verification:

```text
cargo test --locked --manifest-path src-core/Cargo.toml
npm --prefix src-ui run build
git diff --check
```

The portable-core workflow additionally runs shared tests, an optimized Rust build and the UI build on Windows, Linux and macOS. Exact-commit Actions results are required before recording those checks as passing.

## Remaining work

Complete the broader security review, platform adapters, embedded windows, physical-device performance/lifecycle tests, packaging parity, publisher signing and external review as tracked in `ROADMAP.md`. Core build success alone does not establish these capabilities.
