# Managed native child windows — v0.29.18 experimental

Exact source `3a61048` passed [three-OS native GUI acceptance](https://github.com/charitha-only/aegis-desktop/actions/runs/41566723082), [three-OS portable/template checks](https://github.com/charitha-only/aegis-desktop/actions/runs/41566723062) and [Windows installer/upgrade/data-preservation/real-lock verification](https://github.com/charitha-only/aegis-desktop/actions/runs/41566723141). Source tag: v0.29.18. This preview adds explicit IPC commands for the optional native host; legacy window_spawn remains an external-browser launch and window_list remains capability discovery.

Enable security.capabilities.system.allowManagedWindows in aegis.config.json (default false), compile with native-desktop, and start with --native-window. Without permission commands are denied. Without an attached host they fail unavailable. No command opens a fallback browser.

| Command | Payload | Success data |
| --- | --- | --- |
| managed_window_create | Strict window options, same parent authority; omitted URL inherits parent | windowId, backend=NATIVE_EMBEDDED, documentReady=false |
| managed_window_list | Empty object | windowIds (managed children only) |
| managed_window_close | Only windowId | closed=true, windowId; unknown IDs error |

Creation preserves supported native resizing/decorations settings. Unsupported modal/topmost/kiosk/transparency/devtools/custom-protocol options are denied. Each child retains parent-authority navigation policy, denies popups, permissions and downloads; Linux also explicitly denies web file choosers. In-page application code and every child still share the same application IPC principal. IDs are lookup handles, not credentials. Child windows are not isolated vault/profile sessions.

Native objects are created, resized, enumerated and destroyed on their UI thread. Eight children and sixteen queued requests are the limits; failed native creation is not registered. Monotonic child IDs are not reused within a host run. OS child close removes its entry; primary close exits the native host and drops retained children. The list excludes the primary and does not assert document readiness. GTK/WebKit process crashes and every focus/minimize/event scenario are not yet covered.

The opt-in preview polls its request queue every 50 ms on the UI thread. Disabled permissions install no polling source. Per-request result waiting is five seconds; queued cancelled requests are skipped, but a timeout racing a dequeued operation has unknown outcome. Query the list to reconcile. This is not transactional cancellation or retry-idempotent creation. Application-wide commands are serialized by existing IPC dispatch.

The guarded native-smoke.js document requests two real children, lists them, rejects a different origin/unknown ID, closes them, reopens a child and verifies ID non-reuse. It runs only on disposable hosted GitHub runners, with a virtual display on Linux. It is not a substitute for physical devices, screen readers, OS/window-manager coverage or independent review. Local Windows GNU native test execution failed before the test harness; local native cargo check passed. Hosted Windows MSVC, macOS and Linux native suites and actual-window fixtures passed; Windows GNU native execution remains unvalidated. Full counts and remaining gates are in IMPLEMENTATION_STATUS.md.
