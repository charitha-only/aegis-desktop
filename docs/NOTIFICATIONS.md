# Notification submission contract

Notifications require `security.capabilities.system.allowNotifications: true`. The default is false, including when older configurations omit this property. Permission is checked before validation, provider startup, polling or cancellation. Capability metadata and the demo permission display report the configured grant rather than an unconditional true.

`notifySend(title, body, urgency = 'normal', timeoutMs?)` / `notify_send` return `{requestId, status: 'pending', submitted: true, deliveryConfirmed: false}` after starting an owned helper. `notifyPoll(requestId)` / `notify_poll` eventually return `{status: 'providerAccepted', deliveryConfirmed: false}` only after a successful helper exit and expected empty output. This does not establish a visible toast, user attention or read acknowledgement. OS privacy settings, quiet modes, notification permissions and desktop policy can suppress it. Failure exits and unexpected output are errors; text is not echoed in result/error payloads.

Poll roughly once per second. There is one pending/unconsumed notification request per runtime, a 30-second supervision timeout and 30 authorized submission attempts per sliding minute across all profiles. Malformed submissions consume this quota too; polling and cancellation do not. Consume or cancel a result before submitting another. New submissions reclaim abandoned expired slots. Profile switching invalidates requests, and IDs use 128 random bits.

`notifyCancel(requestId)` / `notify_cancel` discard the request and terminate/reap its owned helper. Cancellation does not withdraw an already accepted/displayed notification or clear OS notification history. Runtime termination by an external force may bypass ordinary cleanup; no crash-proof lifetime bound for external provider processes is claimed.

## Inputs and providers

Title/body are required strings, limited to 1,024/16,384 UTF-8 bytes, with no NUL. Unknown fields, unknown urgency values, invalid types and u32 timeout overflow are rejected. Custom icons are unavailable rather than silently ignored.

- Windows uses a hidden PowerShell helper calling WinRT with a fixed script and title/body environment data. Errors stop execution and fail the helper. A valid application identity/OS registration and permission may be required; real signed-app toast acceptance remains pending.
- macOS uses a constant AppleScript program with positional arguments and a ten-second Apple event timeout. Real notification permission/display acceptance remains pending.
- Linux uses notify-send/libnotify. Body backslashes are escaped before XML markup escaping, so notify-send's C-style escape decoding cannot recreate tags or corrupt literal paths. Urgency low/normal/critical is supported. Optional expiry is bounded to 1,000–60,000 ms, but notification servers can apply their own display/expiry policies.

Windows/macOS currently support normal urgency with default OS expiry. Unsupported urgency/timeout overrides return errors. No shell expression, executable path or image/link markup is accepted through this text API. Helpers must exist on the target machine. Provider availability is not asserted solely from the OS name. Tray status explicitly reports unverified provider availability, no delivery confirmation and no implemented tray icon.

## Validation and migration

Public request/provider/job and IPC tests cover default permission denial, parsing, bounded options, literal data, Linux markup/backslash handling, Windows error-stop behavior without showing a toast, truthful completion and a runtime-wide quota that survives profile switching. Shared job tests cover profile scoping, cancellation, timeouts and abandoned-request reclamation.

Linux CI starts an owned org.freedesktop.Notifications service on a private D-Bus session without visual UI, boots a disposable real core and exercises notify-send end to end. It checks Unicode/literal text, urgency, expiry and both accepted/rejected requests. This validates transport, not desktop rendering. Actual Windows/macOS notification and three-OS physical/accessibility acceptance remain separate release gates.

The previous notify_send response incorrectly claimed delivered true immediately after spawn and echoed the message. From v0.25.8 it is asynchronous and default-denied; applications must opt in and consume poll results. No automatic resend/retry is performed, to avoid duplicate notifications. The demo reports submission/provider acceptance separately while keeping its layout.

Notifications can expose text on lock screens and in OS history. This API is unsuitable for presenting passwords or other secrets without an application-specific disclosure policy; OS/provider copies are not guaranteed to be wiped.

References: https://specifications.freedesktop.org/notification/latest/ and https://github.com/GNOME/libnotify/blob/master/tools/notify-send.c.
