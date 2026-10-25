# Version-by-version implementation

This is the release ledger for the roadmap. A source version does not certify production suitability or completion of later milestones.

| Version | State | Evidence and remaining gates |
| --- | --- | --- |
| 0.24.0 | Tagged at 29ca319 | Three-OS core/UI checks and Windows installer/upgrade acceptance passed. Profile collision and restart coverage; window capability/URL validation; notification text separated from scripts; 41 Rust tests. Focused review in PROFILE_ISOLATION_REVIEW.md. External audit remains outside this release claim. |
| 0.25.0 | Tagged at 8e6b075 | Three-OS core/UI and native IPC/storage checks plus Windows acceptance passed. Portable data/config/cache module and typed platform IPC contracts; missing adapters explicitly unavailable; 42 Rust tests. |
| 0.25.1 | Source prepared; CI pending | Native portable directories and packaged boot checks; dynamic-port same-origin fix; core benchmark harness and CI artifacts. Local Rust tests 43, statistics test, UI build and isolated Windows package smoke pass. |
| 0.26 | In progress; dependency gate unresolved | Embedded-window prototype evaluated but not shipped because its Linux chain selected a RustSec-unsound GLib version. See NATIVE_BACKEND_DECISION.md. Full native APIs/GUI acceptance remain. |
| 0.27 | Tooling in progress | Core benchmark harness and nearest-rank statistics test implemented; CI measurements added. Full UI/process-tree workloads and physical constrained-device acceptance remain. |
| 0.28 | Portable tooling in progress | Native portable bundling and isolated packaged-runtime checks added. Native Linux/macOS installers, authenticated update recovery and owner signing/notarization credentials remain. |
| 0.29 | Planned | CLI/templates, typed APIs and restricted plugins. |
| 1.0 | External gates remain | Stable API evidence, physical device validation and independent reviewer report/retest required. |

Independent review cannot be self-certified. Missing physical hardware or signing credentials must not be replaced by fabricated passing results. Keep implementing independent work while recording these gates.
