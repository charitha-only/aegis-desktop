# Dialog helper contract

`window_dialog` checks the configured `system.allowDialogs` capability before preparing or spawning a helper. Denied operations do not start a process. Titles are limited to 1024 bytes and messages to 16 KiB; null characters are rejected. Supported kinds are info, warning and error.

The current provider is a simple Windows message box launched through PowerShell. The script is constant; message/title/icon values travel as environment data rather than being interpolated into executable script. The process starts with its console hidden and is reaped after it exits. This is not an owned parent-window modal API or a mechanism for retrieving the user's response.

Success returns `submitted: true` and `deliveryConfirmed: false`. It means helper startup succeeded; it does not prove the operating system displayed the dialog or that the user read it. The bridge exposes this response as `NativeDialogSubmission`. macOS/Linux currently return an unavailable-provider error instead of a false delivered response. Real cross-platform dialog/file-selection adapters remain part of v0.26.

Regression coverage verifies command/data separation, input bounds, explicit unavailable providers and capability denial through public interfaces. A Windows-only parser check validates the prepared PowerShell syntax without executing it or opening a dialog, and checks that an invalid-script fixture is rejected. It does not replace interactive dialog acceptance.

Dialog strings are copied into the helper's environment/process memory; do not interpret this injection fix as a guarantee that every sensitive string copy is wiped.
