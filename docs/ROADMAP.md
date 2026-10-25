# Aegis Desktop: product direction and release roadmap

Recorded: 2026-10-26. Planning baseline: v0.23.0. Version assignments below are proposed milestones, not delivery promises or completed capabilities.

## Product intent

Build a general-purpose desktop application framework for Windows, Linux and macOS. Prioritize fast startup, low resource use, modern application interfaces, secure defaults and a straightforward developer experience. Support devices with limited CPU and memory through measured engineering rather than feature claims.

An offline password manager is one possible future application, not the framework's identity. Vaults, hardware authentication and other specialized features should be optional capabilities. The framework should support a broad range of desktop applications; suitability for a specific sensitive application requires its own threat model and validation.

Study weaknesses in existing approaches and measure improvements against reproducible workloads. Do not promise universal compatibility, zero vulnerabilities or superiority without evidence.

## Engineering rules

- Develop Windows, Linux and macOS support together. Every release records supported OS versions, CPU architectures, tested environments and known limitations.
- Keep shared behavior in a portable Rust core, with explicit platform adapters. Avoid Windows assumptions in shared paths, process handling, packaging and lifecycle logic.
- Expose capability detection and explicit unsupported errors. Never substitute a simulated success for missing functionality.
- Keep the default runtime small. Load optional features on demand, bound background work and give applications control over expensive operations.
- Fix confirmed defects with regression tests at agreed public interfaces. Keep security and performance checks in CI.
- Preserve the existing README presentation and demonstration UI unless a separate change is requested.

## Proposed milestones

### v0.24: finish the current hardening pass

- Complete review of IPC authorization, validation, resource limits, profile isolation and encrypted storage recovery.
- Retain regression coverage for profile storage collisions, including case-insensitive filesystems and saved-profile loading.
- Document trust boundaries and distinguish process handshake credentials from application/user authentication.
- Establish a capability/support matrix for all three operating systems; mark unimplemented features clearly.
- Publish a scoped security report with reproduced findings, fixes and unresolved limitations.

Release gate: confirmed critical/high findings within scope are resolved or the release is blocked; relevant tests pass. This does not certify the absence of undiscovered flaws.

### v0.25: portable core and three-OS CI

- Separate OS adapters for lifecycle events, data/config/cache locations, process launching, filesystem semantics and secure storage integration.
- Build and test the shared core on Windows, Linux and macOS on every change.
- Specify path handling, case sensitivity, symlinks, permissions and atomic-write behavior per platform.
- Publish the tested OS/architecture matrix; add native ARM64 validation where supported hardware/runners are available.

Release gate: the same core IPC, profile and storage behavior passes on all supported platforms; unsupported adapters fail explicitly.

### v0.26: real desktop windows and application APIs

- Provide an embedded-window backend for each OS, with a consistent lifecycle API and documented webview prerequisites.
- Implement essential APIs: windows, dialogs, file selection, clipboard, notifications and application lifecycle. Make each permission explicit.
- Test keyboard navigation, accessibility, DPI scaling, multiple displays and Unicode input.
- Specify Linux desktop/session coverage, including the supported display-server environments.
- Harden the UI-to-core bridge: command schemas, per-capability authorization, navigation restrictions and bounded/cancellable work.

Release gate: a representative application opens, operates and closes correctly on all three operating systems without requiring an external browser.

### v0.27: performance and constrained devices

- Establish repeatable release-build benchmarks before optimization: cold/warm startup, idle/peak memory, idle CPU, IPC latency, UI responsiveness, disk I/O and package size.
- Choose and publish a constrained-device baseline, including CPU, RAM, storage type, OS and workload. Validate on physical devices as well as CI.
- Define numeric budgets from that baseline, including measurement method and permitted regression. Track distributions rather than a single best run.
- Investigate lazy initialization, bounded queues, asynchronous I/O, cancellation and reduced UI work using profiles.
- Ensure background/minimized applications avoid unnecessary polling, timers and network activity.
- Keep password derivation and other security parameters justified by security requirements; do not silently weaken them to win performance benchmarks.

Release gate: the documented workload meets published budgets on each supported OS and the chosen constrained-device baseline.

### v0.28: packaging, distribution and updates

- Provide tested Windows, Linux and macOS packaging paths, with documented runtime dependencies and supported installation modes.
- Configure platform-appropriate signing and macOS notarization where credentials are available; keep unsigned development artifacts visibly separate.
- Verify update authenticity before installation; define rollback policy, interrupted-update recovery and data migration behavior.
- Test clean install, upgrade, uninstall and reinstall with user-data preservation on each OS.
- Publish checksums and dependency/SBOM information with release artifacts.

Release gate: clean-machine installation and upgrade/recovery tests pass for every advertised package. Production signing remains blocked until owner-provided credentials/services are configured.

### v0.29: developer experience and optional extensions

- Stabilize CLI commands for project creation, development, building, testing and packaging across platforms.
- Provide versioned, typed application APIs with clear errors and migration guidance.
- Supply small examples covering local storage, multiple windows and background work; include an optional secure-vault example separately.
- Define plugin capabilities, signature/trust policy, API compatibility and resource limits before enabling third-party plugins.
- Add actionable diagnostics that redact secrets and make telemetry an explicit product decision.

Release gate: a developer can create and package the same example on all supported platforms using the published instructions.

### v1.0: supported and maintainable release

- Stabilize the public API and define supported OS versions, architectures, upgrade compatibility and maintenance policy.
- Complete an independent assessment of a fixed source revision, remediate findings and obtain retest evidence.
- Complete physical-device lifecycle and recovery testing, including real lock/suspend/resume on each applicable OS.
- Publish release evidence, performance results, security limitations and a vulnerability-reporting process.

Release gate: agreed functionality, reliability, accessibility, security and performance criteria are met with evidence. External audit completion is not an absolute security guarantee.

## Immediate priority order

1. Finish and verify the current profile-isolation fix and remaining scoped audit work.
2. Establish three-OS builds/tests and a truthful support matrix.
3. Implement platform adapters and embedded windows.
4. Measure constrained-device performance and set budgets.
5. Complete packaging/signing/update validation before general production distribution.

## Current boundaries

The planning baseline is not evidence that Linux/macOS packaging or runtime parity is complete. Current validation has focused on Windows. Existing signing preparation is not a publisher-signed release. Synthetic lifecycle tests and hosted Windows lock evidence do not replace physical sleep/resume testing. The current automated review is not a completed independent human audit.
