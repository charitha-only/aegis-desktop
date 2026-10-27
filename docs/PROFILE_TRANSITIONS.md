# Profile storage transitions (v0.26.7)

A profile is a logical workspace, not an OS sandbox or an authenticated user.
Creation and switching now propagate storage preparation errors rather than
report success after an ignored `create_dir_all` failure.

## Public behavior

- Creating a profile checks the existing ID/collision/quota rules, then prepares
  its directory before persisting registry metadata or publishing the new entry.
  A failed directory operation leaves its ID available for retry.
- Switching prepares the target directory before publishing its active identity.
  An existing regular file where the directory should be causes an error. The
  prior active profile and security storage root remain selected.
- After the blocking file is removed, normal creation/switching can be retried.
  A successful switch selects the new root; old profile files remain separate.
- The IPC router revokes retained vault sessions and pending native jobs before
  a switch attempt. That revocation remains in effect even when preparation
  fails. Do not restore old authorization automatically after an error.

The confirmed issue was incorrect success/state handling for an unusable
filesystem destination. No data disclosure or OS boundary escape is established
by these regressions.

## Verification and scope

The existing public IPC profile regression now arranges regular-file blockers
inside an owned temporary project. It observes unsuccessful create/switch
responses, absence of a failed registration in `profile_list`, the unchanged
active profile, and continued reads from the original root. It removes only
those owned fixture files and confirms retry and root separation. Both failure
assertions were observed red before their respective fixes.

```text
cargo test --locked --manifest-path src-core/Cargo.toml ipc_profiles_reject_shared_storage_identities
cargo test --locked --manifest-path src-core/Cargo.toml
```

These tests run natively on Windows, Linux and macOS in portable/native CI.
The simulated filesystem error is a deterministic non-directory destination;
it does not establish physical disk-full, permission-matrix or power-loss
acceptance. If later registry persistence fails, an empty prepared directory may
remain; its existence is not registration. Atomic registry writing retains its
existing behavior. This change does not add cross-process registry transactions,
bounded registry loading, symlink/race protection, or transactional rollback of
OS directory operations. Those remain separate review items.

See [ASVS_5_MAPPING.md](ASVS_5_MAPPING.md) and the exact-source release ledger in
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).
