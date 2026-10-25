# Version-by-version implementation

This is the release ledger for the roadmap. A source version does not certify production suitability or completion of later milestones.

| Version | State | Evidence and remaining gates |
| --- | --- | --- |
| 0.24.0 | Source prepared; exact-commit CI required before tagging | Profile collision and restart coverage; window capability/URL validation; notification text separated from scripts; 41 local Rust tests and UI build pass. Focused review in PROFILE_ISOLATION_REVIEW.md. Broader unknown vulnerabilities and external audit remain outside this release claim. |
| 0.25.0 | Source prepared; exact-commit CI required before tagging | Portable data/config/cache module, platform capability/typed IPC contracts and real native-binary smoke tests. Lifecycle/keychain/installers without adapters are explicitly unavailable. Shared tests and filesystem contracts documented; 42 local Rust tests and UI build pass. |
| 0.26 | Planned | Embedded desktop windows and native application APIs, with real GUI acceptance. |
| 0.27 | Planned | Repeatable benchmarks and measured constrained-device budgets. Physical baseline hardware needed for final acceptance. |
| 0.28 | Planned | Packaging parity and authenticated updates. Owner signing identities/notarization credentials needed for signed production distribution. |
| 0.29 | Planned | CLI/templates, typed APIs and restricted plugins. |
| 1.0 | External gates remain | Stable API evidence, physical device validation and independent reviewer report/retest required. |

Independent review cannot be self-certified. Missing physical hardware or signing credentials must not be replaced by fabricated passing results. Keep implementing independent work while recording these gates.
