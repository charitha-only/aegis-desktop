# Platform support and validation

The product targets Windows, Linux and macOS. This table describes implementation and validation scope, not a blanket support guarantee.

| Capability | Windows | Linux | macOS |
| --- | --- | --- | --- |
| Shared Rust core tests and release build | Native CI | Native CI added; result must be checked per commit | Native CI added; result must be checked per commit |
| TypeScript/UI build | Native CI | Native CI added | Native CI added |
| Packaged app-data directory resolution | `LOCALAPPDATA/AegisDesktop/<app-id>` | `XDG_DATA_HOME/AegisDesktop/<app-id>`; fallback `~/.local/share` | `~/Library/Application Support/AegisDesktop/<app-id>` |
| Embedded application webview | Not implemented | Not implemented | Not implemented |
| Retained-session OS lock monitoring | WTS/power adapter; hosted lock test | Not implemented | Not implemented |
| Installer acceptance | Disposable hosted runner tests | Not implemented | Not implemented |
| Publisher-signed Aegis distribution | Setup available; credentials needed | Not implemented | Not implemented; notarization needed |
| Physical low-resource device benchmarks | Pending | Pending | Pending |

The portable-core workflow tests the native architecture selected by each hosted runner; this is not coverage of every CPU architecture or OS version. Green core/UI builds do not imply desktop integration or installer parity. Inspect the exact-commit workflow result before claiming a platform check passed.

App-data resolution is independent of the working directory and rejects unsafe application IDs and relative roots. Linux ignores a relative `XDG_DATA_HOME` and uses its home-directory fallback. The resolver performs no filesystem writes; packaged startup creates the returned directory. Development-mode data remains in the project directory. Windows keeps its existing data location. The current app ID still comes from the executable's parent folder name, so renaming that folder changes the selected app-data location; stable configured application identities and migrations remain planned work.

Window requests are validated before invoking a browser: only local HTTP URLs with a valid port are accepted, and unsupported modal/kiosk/always-on-top/transparent requests return errors. Validation tests do not launch a browser and do not establish successful native window creation. The existing backend still launches an external browser.

Retained vault sessions outside Windows currently do not receive OS lock/suspend revocation. Applications requiring that guarantee must not treat them as protected by lifecycle monitoring. Platform adapters, supported OS versions, architecture coverage and acceptance criteria must be established before advertising production support.

See [the roadmap](ROADMAP.md) for planned milestones. Existing Windows release checks remain separate so portable checks cannot silently replace installer/signature/lifecycle coverage.
