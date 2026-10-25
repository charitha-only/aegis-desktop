# Version-by-version implementation

This is the release ledger for the roadmap. A source version does not certify production suitability or completion of later milestones.

| Version | State | Evidence and remaining gates |
| --- | --- | --- |
| 0.24.0 | Tagged at 29ca319 | Three-OS core/UI checks and Windows installer/upgrade acceptance passed. Profile collision and restart coverage; window capability/URL validation; notification text separated from scripts; 41 Rust tests. Focused review in PROFILE_ISOLATION_REVIEW.md. External audit remains outside this release claim. |
| 0.25.0 | Tagged at 8e6b075 | Three-OS core/UI and native IPC/storage checks plus Windows acceptance passed. Portable data/config/cache module and typed platform IPC contracts; missing adapters explicitly unavailable; 42 Rust tests. |
| 0.25.1 | Tagged at 614e718 | Three-OS native packaged boot, core/UI and benchmark checks plus Windows installer acceptance passed. Dynamic-port same-origin fix; 43 Rust tests and benchmark statistics test. |
| 0.25.2 | Tagged at 47f05fd | Three-OS core/UI, native IPC, package boot, CLI/statistics and clean-template UI build checks plus Windows installer/upgrade/lock acceptance passed. Exact-commit workflows: portable 40241118112 and Windows 40241118106. |
| 0.25.3 | Tagged at 700b72a | Native fixture-window load/exit/cleanup checks passed on Windows, macOS and Linux (virtual display), alongside three-OS core/UI and Windows installer acceptance. Native workflow 40242908973, portable 40242908741, Windows acceptance 40242908807. Full v0.26 remains incomplete. |
| 0.25.4 | Source prepared; CI pending | Dialog permission/data separation, explicit unavailable-provider errors, typed submission response and native-capability metadata correction. Local Rust tests 47, CLI tests 3 and UI build pass. |
| 0.26 | In progress | Optional embedded-window preview implemented; dependency-selection gate addressed. Full multi-window/dialog/clipboard/file-selection interfaces and accessibility/device acceptance remain. See NATIVE_DESKTOP.md. |
| 0.27 | Tooling in progress | Core benchmark harness and nearest-rank statistics test implemented; CI measurements added. Full UI/process-tree workloads and physical constrained-device acceptance remain. |
| 0.28 | Portable tooling in progress | Native portable bundling and isolated packaged-runtime checks added. Native Linux/macOS installers, authenticated update recovery and owner signing/notarization credentials remain. |
| 0.29 | Tooling in progress | Source-only create command, help/version, typed platform APIs and clean UI-template build checks added. Plugin restrictions/API stabilization and complete three-OS developer acceptance remain. |
| 1.0 | External gates remain | Stable API evidence, physical device validation and independent reviewer report/retest required. |

Independent review cannot be self-certified. Missing physical hardware or signing credentials must not be replaced by fabricated passing results. Keep implementing independent work while recording these gates.

Detailed remaining criteria are in RELEASE_GATES.md. Published tags identify validated source milestones; they are not publisher-signed installer releases or completion of every roadmap row.
