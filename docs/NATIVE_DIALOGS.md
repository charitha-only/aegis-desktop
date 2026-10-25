# Native message-dialog preview

Build with `--features native-dialogs` (also enabled by `file-dialog`). The default build returns an explicit unavailable error. `security.capabilities.system.allowDialogs` must be true; the permission is checked before input processing, spawning, polling or cancellation.

The typed bridge exposes `openMessageDialog(title, message, type)`, `pollMessageDialog(requestId)` and `cancelMessageDialog(requestId)`. Kinds are info, warning and error; title is limited to 1,024 UTF-8 bytes and message to 16 KiB, without NUL. The initial response is `NativeDialogSubmission`: requestId, status pending, submitted true and deliveryConfirmed false. Startup does not establish visible delivery or user acknowledgement.

Poll about once per second. A terminal successful result is `{status: 'acknowledged'}`. Provider nonzero exit, malformed output and timeout are errors; absence of a response is not acknowledgement. This is a UI acknowledgement, not user identity verification or authorization to a vault. Cancel reports that the runtime discarded its request and terminated its owned helper; it does not claim the user pressed Cancel.

Windows and macOS use rfd native message dialogs on an isolated helper's main thread. Linux replaces that helper with zenity, using literal command arguments, disabled Pango markup and a 120-second provider timeout. Zenity must be installed in the desktop session; missing providers return an asynchronous helper error. The runtime tracks the same Linux PID after replacement, avoiding an untracked zenity child. No shell or caller-provided executable is accepted. There is no native parent-window association, custom buttons or confirmation/Yes-No workflow yet.

File and message requests share the tested job implementation with separate slots. One message and one file selection may be outstanding per runtime. IDs contain 128 random bits, are profile-scoped, and terminal results are consumed once. Profile switching cancels pending requests. Timeout polling discards the job, and a subsequent open reclaims an abandoned expired slot (for example after a page reload). The core does not wait for a UI response while holding its dispatcher.

## Migration from v0.25.4

`window_dialog` and `showNativeDialog` remain aliases for message submission, now returning requestId/status in addition to their previous submission fields. They require the optional native-dialogs feature even on Windows; the previous unbounded PowerShell submission route is no longer used by IPC. Callers must poll to consume the result or cancel it before starting another message. The demo follows this lifecycle and reports submission separately from acknowledgement; its layout is unchanged.

## Verification and limits

Public-interface tests cover validation, result types, literal Linux command data, failed-provider rejection and shared job lifecycle. An exit-status negative control confirms that a provider returning plausible acknowledgement output with a failure status cannot succeed. The abandoned-request regression fails before timeout reclamation is applied.

Three-OS native CI compiles the real providers and rejects malformed helper input without opening UI. A separate disposable Linux X11 fixture opens the actual message window and presses its default button. Windows/macOS interactive message acceptance, physical desktops, screen readers, keyboard layouts, scaling and parent-window modality remain separate gates. API/build checks do not certify them.

Text is copied into provider/process memory and Linux command arguments. This API makes no comprehensive memory-wiping or same-account process-inspection guarantee. Avoid displaying secrets merely to confirm an action.

Provider references: https://docs.rs/rfd/0.17.2/rfd/struct.MessageDialog.html and https://gitlab.gnome.org/GNOME/zenity.
