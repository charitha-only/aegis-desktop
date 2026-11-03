# Framework completion workstream

Scope confirmed 2026-11-03: improve Aegis as a general-purpose Windows/Linux/macOS framework. These are acceptance tracks, not a claim that any numbered release completes them. Existing exact-source evidence remains in IMPLEMENTATION_STATUS.md; no project test substitutes for independent review or an unperformed physical test.

| Track | Current boundary | Next acceptance deliverable |
| --- | --- | --- |
| Native experience | Optional primary native host; secondary window_spawn is external. Opt-in child create/list/close adapters passed hosted three-OS acceptance at source 3a61048; capacity/recovery/scripted live-child teardown passed at ef6d1a6. | Extend the verified preview with focus/modal/event behavior, user-operated native close/shutdown races and fault recovery acceptance. Permission checks for native services and actual keyboard/screen-reader acceptance. |
| Low-end performance | Existing core-process measurements do not represent the whole app. | Collect owned application process-tree RAM/CPU and rendered UI startup/response measurements, then repeat on named low-resource physical devices before setting supported budgets. |
| Safe updates | Offline verification, staging and recorded observation exist; automatic application is disabled. | Transactional install journal, interruption/restart recovery, authenticated rollback policy and application migration failure fixtures before enabling application. |
| Distribution | Hosted installer acceptance covers selected native formats. Production signing unavailable. | Explicit OS/CPU matrix, artifact provenance/checksums, owner-controlled certificates/notarization and clean supported-device acceptance. |
| Developer experience | CLI/templates/typed bridge exist with regression coverage. | Consistent public errors, versioned API compatibility/migration policy and full supported-matrix create/dev/build/package examples. |
| Plugin security | Plugins are trusted host code, not an untrusted process sandbox. | Document trust tiers; isolate untrusted code, enforce permissions/resource boundaries and demonstrate denial/crash containment before advertising safe third-party execution. |
| Real-device acceptance | Hosted fixtures and limited headless Windows evidence exist. | Device/OS/runtime records for Windows/Linux/macOS; interactive sleep/resume, scaling, accessibility and storage-interruption evidence. |
| v1.0 verification | Project tests and audit handoff do not complete independent review. | Freeze a candidate, obtain external review, resolve findings, retain retest evidence and publish support/security maintenance commitments. |

Implementation order: native lifecycle first, followed by whole-app measurement and transactional update installation. Distribution, developer/API documentation and plugin isolation progress alongside these where prerequisites are available. Preserve fail-closed unavailable features throughout. Certificates, macOS/Linux physical machines and an independent reviewer are external dependencies currently unavailable; prepare their integration and handoff without fabricating passing evidence.
