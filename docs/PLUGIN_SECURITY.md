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

Handler unwind panics are caught inside dispatch. The affected registration is disabled and subsequent calls fail; IPC revokes retained vault sessions and ephemeral secrets on the caught panic. The router remains available to other commands. `plugin_list.disabledPlugins` returns a sorted list of disabled registration names. Explicitly registering a replacement creates a fresh registration; process restart also rebuilds the registry. Already in-flight native calls are not cancelled.

Dispatch pins the selected registration using shared ownership, then releases its registry lock before calling the handler. A handler can register another plugin without the former read/write-lock deadlock. This also keeps an old in-flight registration alive if trusted host code replaces its name.

`dispatch` returns `Result<Value, PluginError>` with `Rejected(String)`, `Panicked` and `Disabled` variants. Normal handler `Err` results do not disable it. IPC panic responses exclude the panic payload; Rust's configured panic hook still runs and may write the payload to host logs. Native handlers must not put secrets in panic messages.

This catches ordinary unwind panics; it does not roll back writes or native state, enforce execution timeouts, stop unsafe/FFI faults or contain process aborts. Panicking hooks/destructors and arbitrary native code remain host-process risks. Native handlers remain fully trusted.

Public regressions first reproduced the escaped panic and registration deadlock. The IPC fault test proves a fresh session token is revoked, ephemeral secrets are cleared, `sys_info` still works, a disabled handler is not retried and responses omit the test panic payload. Repeat with `cargo test --locked --manifest-path src-core/Cargo.toml ipc_plugin_panic_revokes_sessions_and_preserves_other_commands` and `cargo test --locked --manifest-path src-core/Cargo.toml plugin_handler_can_register_another_plugin_without_deadlock`.

`PluginRegistry::register` handlers now receive `(&serde_json::Value, &SecurityEngine)`. `dispatch` now requires `(name, payload, &SecurityEngine)`. Built-in handlers and IPC use that context. Custom handlers must authorize each actual resource they touch; declaring a grant does not automatically constrain arbitrary Rust filesystem/system calls. Native handlers run synchronously in the host and may block or crash it. Do not run untrusted native code through this interface. Untrusted extension loading, fault/process isolation, authenticated distribution and API stabilization remain roadmap work.
