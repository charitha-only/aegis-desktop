# Managed-window TypeScript SDK

These helpers require the opt-in native host and security.capabilities.system.allowManagedWindows=true. See MANAGED_WINDOWS.md for setup, limits and platform evidence. They never fall back to window_spawn or a browser launch.

| Helper | Input | Success data |
| --- | --- | --- |
| createManagedWindow | ManagedWindowOptions (defaults to an empty object) | ManagedWindowCreated: native windowId, NATIVE_EMBEDDED backend, documentReady=false |
| listManagedWindows | None | windowIds, at most eight distinct managed child IDs |
| closeManagedWindow | ManagedWindowId | closed=true and the same requested windowId |

All helpers return IpcResponse. Always check success before using data. Backend permission/unavailable errors remain errors. A TIMEOUT or backend unknown-outcome error does not establish that native creation was cancelled; query the app-wide list to reconcile, rather than blindly creating again. IDs are lookup handles, not authorization credentials or per-window security principals.

The SDK runtime guards reject malformed successful receipts, foreign/empty IDs, oversized/duplicate listings, external-browser creation receipts and mismatched close IDs. Extra response fields remain compatible. Unknown custom IPC commands remain extensible. These checks validate a covered response contract; they do not authenticate a malicious backend or confirm renderer readiness.

The compilable src-ui/src/examples/managed-windows.ts example checks availability, opens a caller-supplied bundled same-origin page and closes only its retained handle. It performs no actions until its exported functions are called. It is copied with generated React applications as part of src-ui/src. The example does not poll document load events, which the preview does not expose.

SDK tests use an owned HTTP server at the public client boundary, including correct envelopes and malformed command data. They do not open native windows. Native actual-window evidence remains separately recorded in NATIVE_CAPACITY_ACCEPTANCE.md. Three new SDK tests and the UI build passed locally; exact-source hosted verification is pending.
