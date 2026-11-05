# Managed native child windows — v0.29.18 experimental

The [typed TypeScript SDK and compiled example](MANAGED_WINDOW_SDK.md) provide explicit create/list/close helpers; their follow-up hosted verification is recorded separately.

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


## Follow-up acceptance expansion — verified 2026-11-04

The GUI fixture now fills all eight child slots, checks that a ninth creation is rejected without altering the existing IDs, closes a child, creates a replacement with a fresh ID and leaves one live child when the primary fixture completes. Native smoke-only diagnostics confirm exactly one retained registry entry was released on host shutdown. This records owned native-object release and process exit, not exhaustive descendant-process cleanup, physical close-button behavior or crash/power-loss recovery. Requests are paced to keep this capacity test independent of the transport rate limiter. The guarded fixture deadline is 45 seconds inside the runtime and 60 seconds in its owned launcher. Exact source ef6d1a6 passed native workflow 41735933346, portable workflow 41735933330 and Windows installer/upgrade/real-lock workflow 41735933316. See the [OS acceptance matrix](NATIVE_CAPACITY_ACCEPTANCE.md). The v0.29.18 tag remains on its original source; these follow-up assertions apply to ef6d1a6.


## v0.29.19 focus and minimize requests — pending hosted verification

managed_window_focus accepts only windowId. managed_window_set_minimized accepts only windowId and boolean minimized; false requests unminimize. Both require the managed-window permission and an attached native host, target only registered children and reject unknown IDs. Wrong types, nulls and extra options are rejected. There is no primary-window target or external-browser fallback.

Windows/macOS dispatch focus_window/set_minimized on their winit window. GTK dispatches present/minimize/unminimize on its application window. OS focus-stealing and window-manager policies can ignore or alter the requested behavior. Responses contain requested=true, stateConfirmed=false, the matching windowId and action (focus or setMinimized); minimize receipts also echo requestedMinimized. These are acceptance receipts, not focus/minimized-state events. Do not use them as user-presence or authorization evidence. Unminimize does not promise reset geometry or maximized state.

The paired SDK adds focusManagedWindow and setManagedWindowMinimized, with typed results and request-ID/state echo validation. Use a matching v0.29.19 core for these new commands; older managed-window hosts can report an unsupported command. The compiled example shows explicit request methods. Native event delivery, observed focus/minimize state, modal parenting and physical/WM acceptance remain pending.

The guarded actual-GUI fixture issues focus, minimize and unminimize requests, checks truthful receipts, rejects an unknown focus target and malformed minimize payloads, then retains its existing capacity/teardown checks. It does not assert physical focus or observed minimization. Windows still requires all three fresh-fixture runs. Local default Rust 108 passed plus one ignored, SDK 25 passed and Windows native compile passed; hosted exact-source checks remain pending.


### macOS content-view regression

The new real-GUI control fixture aborted on macOS at e9f62a7 and 7ead998. The diagnostic backtrace identifies winit's window_did_resign_key callback. In winit 0.30.13 that callback casts the window contentView to WinitView; Wry 0.57.1 full-view build replaces it with WryWebViewParent. Both primary and managed macOS webviews now use build_as_child to retain winit's view and attach WebKit as a subview. Wry derives initial child bounds from the parent; existing resize handlers maintain bounds. Windows retains its existing full-view build. The same focus/minimize/unminimize GUI fixture remains the regression check; verification of the fix is pending. Temporary operation/backtrace probes were removed after diagnosis.
