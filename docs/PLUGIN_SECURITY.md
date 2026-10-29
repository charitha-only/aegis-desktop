# Native plugin contract and security fixes

Native Rust plugins are compiled, trusted host code. They are not dynamically loaded third-party code or a CPU/memory sandbox. Signing-manifest verification does not load a plugin or grant its permissions. The separate Wasm interpreter has its own no-import, memory and fuel restrictions; those restrictions do not apply to native handlers.

## v0.25.11 findings and remediation

The previous IPC plugin dispatcher invoked handlers without enforcing descriptor capabilities. The checksum demo additionally read `./sandbox/<filename>` directly, allowing an absolute path to select another process-readable file, traversal outside that directory, and hashing the wrong profile. It returned SHA-256 and size, not file plaintext. Reads were unbounded. A valid process handshake was required, but filesystem/hash grants and profile selection were bypassed.

Classification: **Medium — confirmed local capability/profile bypass requiring a valid process handshake**. The affected reviewed revision was `6f51377`: [unchecked dispatch](https://github.com/charitha-only/aegis-desktop/blob/6f51377/src-core/src/plugin.rs#L61), [direct file read](https://github.com/charitha-only/aegis-desktop/blob/6f51377/src-core/src/plugin.rs#L130), and [IPC entry](https://github.com/charitha-only/aegis-desktop/blob/6f51377/src-core/src/ipc/router.rs#L951). A caller could obtain a checksum/size oracle for a file outside its authorized profile. Unbounded reads also created an availability risk; no memory-exhaustion attack was run against a user process.

The dispatcher now receives the active `SecurityEngine` and checks every declared capability before invoking a handler. Supported native descriptor grants are `system_info`, `fs_read` and `crypto`; unsupported grants fail closed. The checksum handler separately authorizes its exact path against the active sandbox, allow/deny rules and existing link/path validation. It accepts only a regular file up to the shared storage limit (2 MiB + 256 bytes), reads with an 8 KiB zeroizing buffer and also caps bytes consumed if the file grows. Existing filesystem-actor/path-resolution race limitations remain; no stronger OS sandbox is claimed.

Public regression tests reproduced the denied-system-info dispatch and absolute-file checksum bypass before remediation. Tests now cover those denials, active-profile selection, the known SHA-256 digest of `abc`, oversized files and missing read/hash grants. Native memory statistics now return unavailable instead of fabricated figures. The uptime handler uses monotonic elapsed seconds and explicitly labels its scope as time since built-in registration.

Repeat the focused checks with `cargo test --locked --manifest-path src-core/Cargo.toml checksum_plugin` and `cargo test --locked --manifest-path src-core/Cargo.toml ipc_plugin_dispatch_enforces_declared_capabilities`. After building the release core, `node aegis-cli/bin/portable-smoke.js` exercises actual IPC and profile switches under owned temporary platform roots.

## Native API compatibility

### v0.25.12 fault handling

Handler unwind panics are caught inside dispatch. The affected registration is disabled and subsequent calls fail; IPC revokes retained vault sessions and ephemeral secrets on the caught panic. The router remains available to other commands. `plugin_list.disabledPlugins` returns a sorted list of disabled registration names. As of v0.29.1, a duplicate registration is rejected, including after a handler fault. Process restart rebuilds the registry. Already in-flight native calls are not cancelled.

Dispatch pins the selected registration using shared ownership, then releases its registry lock before calling the handler. A handler can register another plugin without the former read/write-lock deadlock. Dispatch holds shared ownership of its registration while it executes; registration no longer replaces an existing name.

`dispatch` returns `Result<Value, PluginError>` with `Rejected(String)`, `Panicked` and `Disabled` variants. Normal handler `Err` results do not disable it. IPC panic responses exclude the panic payload; Rust's configured panic hook still runs and may write the payload to host logs. Native handlers must not put secrets in panic messages.

This catches ordinary unwind panics; it does not roll back writes or native state, enforce execution timeouts, stop unsafe/FFI faults or contain process aborts. Panicking hooks/destructors and arbitrary native code remain host-process risks. Native handlers remain fully trusted.

Public regressions first reproduced the escaped panic and registration deadlock. The IPC fault test proves a fresh session token is revoked, ephemeral secrets are cleared, `sys_info` still works, a disabled handler is not retried and responses omit the test panic payload. Repeat with `cargo test --locked --manifest-path src-core/Cargo.toml ipc_plugin_panic_revokes_sessions_and_preserves_other_commands` and `cargo test --locked --manifest-path src-core/Cargo.toml plugin_handler_can_register_another_plugin_without_deadlock`.

`PluginRegistry::register` handlers now receive `(&serde_json::Value, &SecurityEngine)`. `dispatch` now requires `(name, payload, &SecurityEngine)`. Built-in handlers and IPC use that context. Custom handlers must authorize each actual resource they touch; declaring a grant does not automatically constrain arbitrary Rust filesystem/system calls. Native handlers run synchronously in the host and may block or crash it. Do not run untrusted native code through this interface. Untrusted extension loading, fault/process isolation, authenticated distribution and API stabilization remain roadmap work.

## v0.29.1 registration contract

`PluginRegistry::register` returns `Result<(), RegistrationError>`. The variants are `InvalidDescriptor`, `DuplicateName`, `RegistryFull` and `RegistryUnavailable`. No rejected registration changes the existing entry or resets its faulted state. Duplicate detection and the capacity check occur under the same write lock as insertion. There is no hot replacement/unregister interface; restart the process to rebuild registrations.

Names use 1–64 ASCII bytes, begin with a lowercase letter and contain lowercase letters, digits, underscore or hyphen. Version labels use 1–64 ASCII bytes, begin with a digit and contain letters, digits, dot, plus or hyphen; these are bounded labels, not SemVer compatibility validation. Description and author allow Unicode up to 1024 and 256 UTF-8 bytes respectively, without control characters. Empty description/author are allowed. Grants must be unique and drawn from `system_info`, `crypto`, `fs_read`. Unsupported grants now fail at registration as well as dispatch. The registry permits at most 128 registrations, including the three built-ins.

Migration: use `registry.register(descriptor, handler)?` and propagate or deliberately handle the result. `register_builtin_plugins` also returns `Result`; framework startup propagates it before serving requests. Built-in registration is a sequence of individually checked registrations, not an all-or-nothing batch. Existing code that intentionally overwrote names must choose distinct names or rebuild the registry at process startup. Descriptor validation restricts metadata, not arbitrary native handler behavior.

Repeat with `cargo test --locked --manifest-path src-core/Cargo.toml plugin_registration`. Tests exercise public registration/list/dispatch behavior, preserving the original handler after duplicates, rejected metadata and registry exhaustion while an existing handler remains callable. Existing capability, panic revocation, checksum and reentrant-registration tests remain required. This is framework contract hardening, not evidence of a remote attacker-controlled plugin loader or an independent audit.
