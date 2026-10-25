# Native backend evaluation: unresolved release gate

Evaluated Wry 0.57.1 and Tao 0.37.2 for an optional main-thread embedded window. A local prototype used a waiting event loop, local-origin navigation restrictions, and denied popups/downloads/new permission requests. The prototype is not included in the released source.

The resolved Linux dependency chain selected glib 0.18.5. RustSec reports RUSTSEC-2024-0429, an unsound VariantStrIter implementation in versions below 0.20.0. It also reported the unmaintained proc-macro-error package (RUSTSEC-2024-0370). The audit policy rejects these warnings; it was not relaxed to admit the prototype. The default framework lockfile remains free of these proposed dependencies.

Reference: [RustSec GLib advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html). This is dependency-risk evidence, not a demonstrated Aegis exploit or proof that the proposed application called the affected function.

The local Windows MSVC check also failed because link.exe/Visual C++ Build Tools are absent. A GNU core build is available; that is not equivalent to native WebView2 feature validation.

Before v0.26 completion, choose and validate a maintained Linux webview dependency path (for example, evaluate a GTK4/WebKit6 adapter), or maintain and test a narrowly reviewed upstream fix with an explicit dependency policy. Then implement the full native application APIs and run interactive launch, accessibility, DPI, multi-display and session tests on the supported platforms. Do not mark v0.26 complete merely because an optional backend compiles.
