# Platform support and validation

The product targets Windows, Linux and macOS. This table describes implementation and validation scope, not a blanket support guarantee.

| Capability | Windows | Linux | macOS |
| --- | --- | --- | --- |
| Shared Rust core tests and release build | Native CI | Native CI added; result must be checked per commit | Native CI added; result must be checked per commit |
| TypeScript/UI build | Native CI | Native CI added | Native CI added |
| Packaged app-data directory resolution | `LOCALAPPDATA/AegisDesktop/<app-id>` | `XDG_DATA_HOME/AegisDesktop/<app-id>`; fallback `~/.local/share` | `~/Library/Application Support/AegisDesktop/<app-id>` |
| Optional embedded application webview | Wry/WebView2 preview; hosted fixture passed | GTK4/WebKit6 preview; virtual-display fixture passed | Wry/WKWebView preview; hosted fixture passed |
| Opt-in text clipboard writes | Native provider preview | X11 provider preview; native Wayland not enabled | Native provider preview |
| Optional single-file picker | Windows shell helper preview | XDG Portal helper preview | AppKit helper preview |
| Optional native message dialogs | rfd native helper preview | zenity helper preview; requires installed provider | rfd AppKit helper preview |
| Retained-session OS lock monitoring | WTS/power adapter; hosted lock test | Not implemented | Not implemented |
| Installer acceptance | Disposable hosted runner tests | Not implemented | Not implemented |
| Native portable directory | Supported | Implementation added; native CI acceptance required | Implementation added; native CI acceptance required |
| Publisher-signed Aegis distribution | Setup available; credentials needed | Not implemented | Not implemented; notarization needed |
| Physical low-resource device benchmarks | Pending | Pending | Pending |

The portable-core workflow tests the native architecture selected by each hosted runner; this is not coverage of every CPU architecture or OS version. Green core/UI builds do not imply desktop integration or installer parity. Inspect the exact-commit workflow result before claiming a platform check passed.

App-data resolution is independent of the working directory and rejects unsafe application IDs and relative roots. Linux ignores a relative `XDG_DATA_HOME` and uses its home-directory fallback. The resolver performs no filesystem writes; packaged startup creates the returned directory. Development-mode data remains in the project directory. Windows keeps its existing data location. The current app ID still comes from the executable's parent folder name, so renaming that folder changes the selected app-data location; stable configured application identities and migrations remain planned work.

Window requests are validated before invoking a browser: only local HTTP URLs with a valid port are accepted, and unsupported modal/kiosk/always-on-top/transparent requests return errors. The default backend launches an external browser. Builds with `native-desktop` can select `--native-window`; v0.25.3 passed disposable hosted fixture creation, document readiness, exit and cleanup on all three runner platforms (native workflow 40242908973). This does not establish physical-device, accessibility, multi-display or full interactive acceptance. See [native desktop setup](NATIVE_DESKTOP.md).

Retained vault sessions outside Windows are unavailable until real OS lifecycle monitors exist; fresh-passphrase encrypted-file operations remain available. Platform adapters, supported OS versions, architecture coverage and acceptance criteria must be established before advertising production support.

Clipboard writes require their own default-denied capability and bounded text input. There is no clipboard-read API or guarantee of secret expiry/history removal. See [the clipboard contract and test scope](CLIPBOARD.md).

The optional file picker uses default-denied permission and profile-scoped asynchronous requests. Native provider builds and non-GUI helper tests are distinct from interactive selection/accessibility acceptance, which remains pending. A selected path grants no filesystem access. See [the file-picker contract](FILE_DIALOGS.md).

See [the roadmap](ROADMAP.md) for planned milestones. Existing Windows release checks remain separate so portable checks cannot silently replace installer/signature/lifecycle coverage.

Portable bundles use the host release binary, built UI, configuration and a launcher. Linux/macOS output is an unsigned directory, not an AppImage, Debian package, notarized .app or DMG. No cross-compilation is implied. Execute `node aegis-cli/bin/package-smoke.js` after building the core/UI to bundle and boot a disposable fixture without mixing existing bundle output or application data.
