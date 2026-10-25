# Native text clipboard preview

`clipboard_write_text` accepts `{ "text": "..." }` and returns `{ "written": true }` only after the native provider accepts the write. The typed client is `writeClipboardText(text)`. Enable `security.capabilities.system.allowClipboardWrite` in the application configuration to grant access; omitted/false denies access before any native clipboard work. This permission is separate from dialogs and shell execution.

Only UTF-8 text writes are exposed. Input is limited to 65,536 bytes and embedded NUL is rejected to avoid platform-dependent truncation. There is no clipboard read API, HTML/image support, history control or automatic secret expiry. Responses and provider errors do not echo clipboard text.

The provider is arboard with image dependencies disabled. Windows and macOS use native clipboard providers. Linux uses X11 (including suitable XWayland environments); native Wayland data-control support is not enabled. Missing display/clipboard access returns an error. The provider is opened lazily and retained in the router because X11 clipboard ownership requires a live instance. Quitting may remove X11 clipboard availability.

Clipboard data is shared with other applications and may be retained by OS clipboard history, remote desktop or clipboard managers. Vault locking does not retract copied text; this preview does not claim secure password-copy lifecycle or memory erasure of provider/OS copies. Application authors must disclose this when offering copy actions for secrets.

The default test suite checks permission denial and input validation without accessing the user's clipboard. An ignored integration test writes a synthetic Unicode fixture, verifies it through a separate native clipboard instance and clears it. The native desktop workflow runs it only on disposable GitHub-hosted runners (Linux under Xvfb). The test refuses to run outside that environment. This establishes a hosted text roundtrip, not physical desktop-manager or Wayland acceptance.

Provider behavior reference: https://docs.rs/arboard/3.7.0/arboard/struct.Clipboard.html
