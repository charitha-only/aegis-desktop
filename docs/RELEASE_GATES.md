# Remaining release acceptance

Keep one evidence record per source commit, OS, architecture, package format and physical test device. A passing API test cannot be substituted for a missing interactive/device test.

## v0.26: native application behavior

- Preserve the clean native dependency graph documented in NATIVE_BACKEND_DECISION.md; keep auditing the dependencies selected for supported platforms. The initial Linux dependency blocker is resolved.
- Extend the passing hosted embedded-window fixtures on Windows, Linux and macOS to physical-device acceptance. Record installed webview/runtime versions and tested Linux display sessions; the Linux hosted fixture uses a virtual display.
- Implement typed, permission-controlled window, dialog, file-selection, clipboard, notification and lifecycle interfaces. Return errors for unavailable behavior.
- Verify navigation/popups/downloads, clipboard data handling, keyboard-only use, screen readers, scaling, Unicode and multiple displays.
- Revoke retained sessions on real lock/suspend/resume; show fail-closed behavior if monitoring fails.

## v0.27: constrained-device performance

- Select physical baseline devices and publish CPU, RAM, storage, OS and workload. The current developer machine and hosted CI are not substitutes for an untested low-resource device class.
- Measure the entire application process tree, rendered UI responsiveness, startup/cache conditions and background activity.
- Set numeric budgets from repeated measurements, test regressions and retain raw results. Do not weaken encryption/KDF parameters to satisfy performance targets.

## v0.28: distribution and updates

- Complete native installer formats and clean install/upgrade/uninstall/reinstall/data migration on all advertised platforms. Portable folders do not establish installer parity.
- Configure owner-controlled signing identities, timestamping and macOS notarization. Production signing is unavailable until the owner supplies a real service/certificate configuration through secure secret storage.
- Authenticate update metadata and payloads against a provisioned trust root; test wrong publishers, altered metadata, rollback policy, interruption and failed migration. Hash equality alone is not publisher authentication.
- Exercise power-loss/recovery behavior on representative physical storage. Publish checksums/SBOM/provenance for artifacts actually produced.

## v0.29 and v1.0

- Validate the complete create/develop/build/package journey on the supported OS/architecture matrix; stabilize public types, errors and migration policy.
- Test plugin trust/capability/resource-limit behavior before enabling third-party code.
- Freeze a release-candidate revision for a separate qualified reviewer. Obtain a report, fix findings and obtain retest evidence; project-authored checks cannot complete this independent gate.
- Publish supported environments, limitations, maintenance/compatibility policy and vulnerability-reporting process.

The user has authorized public-interface testing for session/IPC, platform directories, windows, native services, CLI/build/package/update, plugins and performance reporting. These approvals do not turn unperformed physical tests or an unavailable external audit into passing evidence.

Linux Debian preview evidence: source `d872e4c` verifies native package installation, renamed upgrade, removal/reinstallation and authenticated vault/backup preservation on hosted Ubuntu. See [Debian packaging](DEBIAN_PACKAGING.md). This does not establish compatibility with all distributions or replace signing, macOS packaging and physical-device gates.
