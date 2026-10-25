# File-selection preview

Build with `cargo build --release --manifest-path src-core/Cargo.toml --features file-dialog` (combine with `native-desktop` when needed). Rust 1.92 or newer is the declared minimum. The default build has no file-picker provider and returns an explicit unavailable error. Set `security.capabilities.system.allowFileDialogs` to true to opt in; omitted/false denies open/poll/cancel before provider work.

The typed bridge exposes:

1. `openFileDialog(title)` returns `{requestId, status: 'pending'}` after helper startup, not after selection.
2. `pollFileDialog(requestId)` returns `pending`, `selected` with a UTF-8 absolute `path`, or `noSelection`. Poll about twice per second. A terminal result is consumed once.
3. `cancelFileDialog(requestId)` discards the request and terminates its owned helper.

The IPC commands are `file_dialog_open`, `file_dialog_poll`, and `file_dialog_cancel`. Only the title (up to 1,024 UTF-8 bytes, no NUL) is configurable. No arbitrary executable, initial path, shell command or script is accepted from IPC. Request IDs use 128 random bits and are scoped to the active profile. Profile switching invalidates outstanding requests. One pending/unconsumed request is allowed per runtime; poll or cancel before starting another.

The separate helper runs the platform picker on its main thread, leaving the core request dispatcher available for lock/session operations. Native providers are Windows shell dialogs, macOS AppKit and Linux XDG Desktop Portal. Linux deliberately disables executable fallback lookup in the helper, so zenity is not spawned; a working portal backend/session is required. Dialogs are standalone, without a native parent-window association. The helper has a 120-second lifetime ceiling; polling after the request deadline returns a timeout and discards it. Ordinary cancellation/runtime shutdown terminates and reaps the owned helper. An external portal may manage the visible dialog independently; no universal remote-dialog dismissal guarantee is claimed.

The provider returns the same empty result for cancellation and some provider failures. Consequently `noSelection` deliberately makes no claim about why no path was returned. Helper startup, malformed output, nonzero exits and timeout are errors. Output is bounded to 64 KiB; paths must be absolute UTF-8, no NUL, at most 32 KiB. Non-UTF-8 Unix filenames return an error rather than being silently changed.

Selection returns a path only. It does not open/read/write the file, issue a filesystem capability or bypass the existing sandbox. Import/export grants and safe handle-based I/O are separate future work. A path can change after selection, so it must not be treated as an authenticated file handle. No save, folder, multi-select, extension filtering or retained macOS security-scoped bookmark support is provided yet.

Tests exercise the public request/result and job interfaces using real controlled non-GUI child processes: validation, nonblocking polling, single-slot bounds, profile scoping, single consumption and timeout cleanup. Three-OS native CI compiles the real providers and checks malformed helper input before opening UI. Interactive file selection, cancellation, parent ownership, keyboard/screen-reader behavior and portal desktop combinations still require acceptance on real desktop sessions. These tests do not certify that interactive picker behavior.

Provider reference: https://docs.rs/rfd/0.17.2/rfd/struct.FileDialog.html
