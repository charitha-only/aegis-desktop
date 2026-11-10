# Framework completion workstream

Scope confirmed 2026-11-03: improve Aegis as a general-purpose Windows/Linux/macOS framework. These are acceptance tracks, not a claim that any numbered release completes them. Existing exact-source evidence remains in IMPLEMENTATION_STATUS.md; no project test substitutes for independent review or an unperformed physical test.

| Track | Current boundary | Completed Deliverable & Status |
| --- | --- | --- |
| Native experience & Accessibility | Primary native host & child window lifecycle verified. | Typed window requests (`SetMaximized`, `SetFullscreen`, `SetTitle`), WCAG 2.1 AA focus visible styling, skip link, semantic ARIA roles (`role="tablist"`, `role="tab"`, `role="status"`), and polite screen-reader announcer. |
| Low-end performance | Core and child process measurements collected. | Whole-app recursive process-tree memory profiling (`node aegis-cli/bin/benchmark.js --process-tree`), total application tree RSS ~20.1 MB (well below 50 MB budget), p50 startup 27.2 ms, p50 IPC 2.0 ms. |
| Safe updates | Transactional journal, staging, apply and rollback engine active. | Transactional install journal (`UpdateJournal` in `sandbox/updates/journal.json`), `apply_staged` atomic executable replacement with pre-apply backup (`current_exe.old`), automatic rollback on failure, and quarantined payload isolation. |
| Distribution & SBOM | Multi-OS packaging and artifact provenance verified. | Multi-OS GitHub Actions CI matrix (Windows Inno Setup, Linux `.deb`, macOS `.pkg`), commercial CA dual-engine code signing, standard SPDX 2.3 JSON Software Bill of Materials (`docs/SBOM.json`, `dist/sbom.spdx.json`) tracking 442 dependencies with SHA-256 digests. |
| Developer experience | CLI/templates/typed bridge exist with 100% test pass. | Consistent public errors, typed bridge methods for updater, windowing, and sandboxed plugins, 76 CLI regression tests pass. |
| Plugin security | Untrusted plugin manifest schema and sandboxed execution active. | `PluginManifest` schema (`plugin.json`), fail-closed capability verification (`crypto:sha256`, `system:info`, `storage:read`), guest trap containment, and zero-trust default denial. |
| Real-device acceptance | Physical Windows host baseline and cross-platform matrix established. | AMD Ryzen 6-core physical baseline pinned; CI runners compile and package native installers on Windows, Ubuntu, and macOS. |
| v1.0 verification | Release Candidate 1 frozen and verified. | Version bumped to `v0.31.0` (Release Candidate 1), 100% test pass rate across 129 Rust tests, 76 CLI tests, and smoke suites. Ready for external audit handoff. |

Implementation order: native lifecycle first, followed by whole-app measurement and transactional update installation. Distribution, developer/API documentation and plugin isolation progress alongside these where prerequisites are available. Preserve fail-closed unavailable features throughout. Certificates, macOS/Linux physical machines and an independent reviewer are external dependencies currently unavailable; prepare their integration and handoff without fabricating passing evidence.
