# Sensitive-data framework review gates (v0.20.0)

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

Run cargo test --locked --manifest-path src-core/Cargo.toml, npm --prefix src-ui run build and npm audit --prefix src-ui. Run npm run test:sensitive only against a fresh isolated core; it creates encrypted test fixtures and must not be pointed at real user vault data. Live smoke checks cover authenticated overwrite, raw-write rejection, backup authentication/restore, password rotation, reserved names, JSON enforcement and origin rejection. Independent audit and clean-machine/hardware gates remain pending.
