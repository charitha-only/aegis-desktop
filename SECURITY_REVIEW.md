# Sensitive-data framework review gates (v0.23.0)

This version hardens the framework. It does not create a password manager, banking application, hardware provider or a security certification. The original README presentation is retained.

## Protected operations

V2 vaults use Argon2id v0x13 with 19,456 KiB memory, 2 iterations, 1 lane and 32-byte keys, plus a 16-byte random salt and a 24-byte random XChaCha20-Poly1305 nonce. Parameters are pinned so existing V2 files remain readable. New encryption requires at least 12 characters; old V2 files still decrypt with their original password. V1 remains unsupported and is never silently rewritten.

All existing-vault replacement requires the existing passphrase. Atomic vault writes authenticate again while holding an OS file lock, write and sync a same-directory temporary file, preserve the prior encrypted generation as .bak, then replace the active file. Failed authentication does not mutate the vault. Writers using this helper cannot interleave. The backup is one prior generation, not a substitute for offline backup media. Quota checks conservatively include space for the previous generation.

Framework APIs available through the existing bridge:

- vault_encrypt_store: create or replace a V2 vault using its current passphrase.
- vault_decrypt_retrieve: authenticate and return plaintext to the requesting application.
- vault_rotate_passphrase: filename, oldPassphrase, newPassphrase; keeps a backup encrypted with the previous password.
- vault_restore_backup: filename ending in .vault and the backup's passphrase; authenticates the backup before replacing the active file. Restoring can deliberately revert the master password and data to the backup's generation.

The TypeScript bridge exposes vaultRestoreBackup and vaultRotatePassphrase. No new app screens or application workflow are added.

## Memory and transport boundaries

Derived-key arrays, raw IPC request bodies, parsed string payload values and owned sensitive response buffers are cleared on drop using zeroize. Debug formatting redacts payloads. Caller-supplied crash traces are omitted. This reduces retention; it does not guarantee that every allocator, OS, browser, network or JavaScript copy is erased. Plaintext returned by library APIs remains the caller's responsibility. Browser demo state is cleared on blur/hidden and after a 60-second display window; this is not an application vault-lock implementation.

The bridge binds to loopback and checks Host/Origin and JSON body sizes. Packaged UI CSP restricts script/connect sources to the application origin, and responses carrying IPC data are no-store. Development origins are excluded from a packaged runtime. Expensive vault/secret operations are limited to 20/minute per process; this shared budget may affect other local clients. Failed authentication can trigger a cooldown. Loopback HTTP is not TLS. The process-level handshake token is not user authentication and must not be used as an unlocked vault session credential.

## Threat model

Covered: malformed requests, unauthorized browser origins, unauthenticated vault overwrite, ciphertext tampering, password guessing rate within the running framework, linked/traversing application paths, partial application writes and cooperating concurrent writers.

Not covered: malware or administrators with equivalent OS-account access, process memory extraction, malicious browser extensions, compromised application code, a malicious filesystem actor racing path resolution, OS crashes/power-loss guarantees on all filesystems, secure clipboard handling, rollback attacks against whole disk snapshots, signed application distribution or a compromised supply chain. Atomic rename/flush behavior must be verified on the target filesystem and storage device; only a bounded helper failure/recovery test is automated here.

Hardware authentication, TPM, HSM/PKCS#11, biometrics, GPU, P2P, native non-Windows packaging and encrypted IPC transport remain unavailable. These features are optional for a master-password-encrypted offline design; they must not be represented as protections without real providers and device tests.

## Before a future password-manager release

- Define the application threat model, master-password/key lifecycle, lock/idle policy, user authentication and client session authorization.
- Avoid plaintext persistence and logs; assess browser/extension risks and decide whether an embedded/native UI is required.
- Implement authenticated, tested backup/export/import and a recovery policy that clearly states that a forgotten master password cannot be recovered by this framework.
- Measure KDF costs on supported devices and design a versioned migration before changing V2 parameters.
- Review rollback, account isolation, clipboard, suspend/resume, concurrency, file ownership/ACLs and storage permissions on each OS.
- Independently audit application and framework cryptography/IPC, fuzz untrusted parsers, run real-device tests for enabled providers, and test signed installers on clean machines.
- Do not call this banking-ready: banking additionally needs a separate server/authorization/transaction system and applicable compliance review.

## Sources used for implementation choices

- [OWASP password storage recommendations](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html).
- [zeroize and its limitations](https://docs.rs/zeroize/latest/zeroize/).
- [Rust OS file-lock API](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock).

## Verification

Run cargo test --locked --manifest-path src-core/Cargo.toml, npm --prefix src-ui run build and npm audit --prefix src-ui. Run npm run test:sensitive and npm run test:sessions each against a fresh isolated core; it creates encrypted test fixtures and must not be pointed at real user vault data. Live smoke checks cover authenticated overwrite, raw-write rejection, backup authentication/restore, password rotation, reserved names, JSON enforcement and origin rejection. Independent audit and clean-machine/hardware gates remain pending.

### Additional recovery regression coverage

Automated storage tests also exercise a backup target that cannot be replaced, a truncated active vault with a valid prior-generation backup, and an oversized replacement. Failed writes must preserve the existing generations and handled backup failure must leave no temporary file. These are deterministic failure tests; they do not simulate actual disk exhaustion, sudden power loss or a clean-machine installation.

### Outstanding release evidence

Application-wide user authorization and application UI lock/key lifecycle integration remain application work. The scoped session support below supplies vault-specific authorization only. Signing requires a publisher-controlled certificate or signing service; do not commit private signing keys. Installer verification needs a clean Windows environment. An independent audit requires a separate reviewer and cannot be replaced by this project's own test suite. None of these gates is marked complete by the added recovery tests.


## v0.21 vault-scoped unlocked sessions

`vault_session_unlock` takes an existing `.vault` filename and passphrase. Authenticated decryption is required before a fresh OS-random 256-bit token is issued. `vault_session_read` accepts that filename and `sessionToken`, with no password in subsequent read requests. `vault_session_lock` revokes the active session and clears ephemeral secrets. These interfaces do not provide a general OS user login or administrative authorization.

One unlocked vault is permitted per runtime, scoped to its resolved profile path. Successful re-unlock replaces the previous token. Every dispatched vault mutation attempt and profile switch attempt also revokes the session; this conservative behavior includes failed attempts. A token cannot authorize another vault. A 60-second idle limit and a 300-second absolute limit use a monotonic clock; successful reads refresh only the idle deadline. Reads check expiry before and after decryption. A background worker checks expiry approximately once per second without requiring traffic; OS scheduling can delay memory cleanup. Core requests are serialized against profile changes and explicit lock. Already returned or queued HTTP plaintext cannot be recalled by locking.

The retained password, session token, discarded unlock plaintext and session plaintext intermediates use zeroizing owned buffers. Lock, expiry, replacement and manager destruction drop retained credentials. This reduces retention and is not a guarantee against process-memory access or every allocator/OS/browser copy. The TypeScript bridge leaves token ownership with the caller: keep it in memory, never persist it, and clear application plaintext on lock. The demo layout is preserved and has not been converted into a session-based password-manager interface.

Existing password-per-operation vault and secret interfaces are retained. They still require the relevant passphrase; presenting a valid master password is fresh authentication even after lock. A lock revokes retained session access, not knowledge of the password. Initial vault creation is not enrollment of an OS user. Invalid session tokens never unlock a vault. Shared loopback handshake tokens are still transport credentials, distinct from vault-session tokens. Native Windows lock/suspend acceptance on supported devices, clipboard handling and application-wide idle policy remain required before release.

Verification includes authenticated unlock, forged/cross-vault token rejection, explicit lock, re-unlock revocation, idle/absolute expiry, cleanup without another IPC request, profile-change revocation and lock during rate limiting. Real power loss, disk exhaustion, signed installers and independent audit remain unverified.

Local v0.21 validation: 31 Rust tests, production UI build, release core build and both isolated portable smoke suites passed. This evidence does not complete the external release gates above.


## v0.22 Windows lifecycle and release evidence

Windows production routers require an active native WTS/power monitor for unlocked vault-session operations. It uses a hidden top-level window, registers only the current Windows session and revokes on every WTS transition and suspend/resume broadcasts. It also queries current WTS lock/connect flags before and after unlocking/reading; unknown/disconnected/locked state denies access. Registration failure, window destruction or message-loop exit leaves session access disabled. An atomic revocation generation is checked against retained credentials and after decryption to reject in-flight results when an OS event arrives. All retained sessions require fresh vault authentication after an event; Windows unlock does not unlock a vault.

Native tests register the real window and inject lock/power messages into that test process only. They do not actually suspend/lock the machine. Native integration is implemented, but supported-device Win+L, sleep/hibernate/Modern Standby and remote-session acceptance remain release gates. Legacy password-per-operation interfaces remain fresh-authentication paths and are not a global OS-user authorization system. Data already returned to the UI cannot be recalled.

Normal CI now compiles an explicitly unsigned test installer, installs it into a fresh temporary directory in a disposable GitHub-hosted Windows runner, exercises the installed runtime and lifecycle registration, uninstalls and checks that encrypted vault files are preserved. A hosted runner is not a pristine Windows installation. See WINDOWS_RELEASE.md for scope and publisher signing setup. No publisher certificate exists in this environment; the signed workflow refuses to upload without valid publisher signatures/timestamps. No external independent audit was performed; SECURITY_AUDIT_HANDOFF.md defines the required reviewer scope/evidence.


## v0.23 upgrade/reinstall and real Windows lock acceptance

The hosted Windows CI workflow now builds the actual pinned v0.22 source for upgrade testing. Vault and backup ciphertext hashes are compared before/after upgrade and uninstall/reinstall; separate IPC checks authenticate the retained vault contents and restore the prior encrypted generation after migration/reinstall. These are synthetic test vaults only.

The final explicitly enabled CI step locks the disposable hosted Windows workstation through the real OS API and checks the resulting WTS state, denied scoped session reads/unlocks, and ephemeral-secret cleanup. This script rejects local/self-hosted invocation. It never locks the developer PC. Earlier synthetic window-message tests remain useful handler coverage; real sleep/resume, hibernation, Modern Standby and supported physical-device acceptance remain unperformed.

CI uploads exact-commit, sanitized installer/lock evidence, with external audit and publisher signing clearly pending. The actual upstream-signed Inno Setup fixture was accepted by the publisher-signature verification helper; unsigned/wrong-publisher executables are rejected. This validates signature checking but does not create an Aegis publisher identity. Private certificate containers are ignored by Git. No independent human audit or signed Aegis release was completed.
