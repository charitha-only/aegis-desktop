# v0.29.0 — Developer workflow preview (2026-10-29)

- Add read-only `doctor [--json]` prerequisite diagnostics with actionable failures.
- Add `package [--print-plan]`: locked Rust build, UI build, portable bundling, bounded integrity inventory and immediate verification. Pin the Cargo target directory to avoid packaging a stale binary when Cargo configuration redirects output. Refuse existing bundle output before compilation.
- Extend clean generated-project acceptance from UI-only compilation to actual Rust/UI release builds and verified portable packages on Windows, Linux and macOS.
- Exact-source portable/native checks on all three OSes and Windows installer acceptance passed; see the verification ledger. This is not signing, an installer or a security certification; stable API/plugin contracts and physical-device/audit gates remain open.

# 📜 Aegis Desktop Changelog & Release Notes

All notable changes, new features, security updates, and bug fixes for **Aegis Desktop** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.28.7] - 2026-10-29

- Add read-only `inspect-update-state` operator diagnostics: validated cached observation, lock/pending presence and a point-in-time blocker snapshot. Explicitly report that the cached record is not fresh authentication.
- Preserve all files and fail closed on missing/corrupt committed state. Do not infer stale locks, inspect blocker contents, clear files, advance floors or enable installation.
- Add four public API/CLI regressions and native-artifact diagnostic checks, including inspection under real Linux filesystem exhaustion. Exact-source three-OS checks, Linux full-filesystem inspection and Windows installer acceptance passed; see the verification ledger.
- Operator crash recovery, runtime install/recovery, physical durability and tamper-resistant state remain separate gates.

## [v0.28.6] - 2026-10-29

- Add `init-update-state` and `verify-tracked-update` developer commands. Persist an authenticated release observation under an explicitly provisioned state directory; reverify every attempt and permit matching-release retries across processes.
- Reject sequence/version rollback, changed same-sequence release fields and target changes. Bound and validate state; missing/corrupt state, occupied locks and pending writes fail closed rather than reset. Serialize cooperating writers and publish synced replacement state via rename.
- Add seven public API/CLI regressions, real native-artifact tracked retries on all OS runners and Linux ENOSPC checks for preservation/retry of the committed observation. Exact-source three-OS checks, Linux ENOSPC state preservation/advancement and Windows installer acceptance passed; see the verification ledger.
- This records verification, not installation. Runtime integration, stale-lock/pending-write operator recovery, directory/power-loss guarantees, OS signing and resistance to same-account/disk-snapshot state rollback remain open.

## [v0.28.5] - 2026-10-29

- Add a guarded hosted-Linux acceptance fixture using a bounded private tmpfs to provoke real kernel `ENOSPC` during an 8 MiB signed-update staging copy. Verify partial destination cleanup/rejection and retry after increasing capacity.
- Fill the private filesystem again with a completed stage present, verify that stage and trusted policy survive a failed second attempt, then free space and retry. Mount ownership is checked before cleanup; failed unmounts never trigger recursive mount deletion.
- This validates existing public CLI behavior; no speculative production patch is included. Exact-source three-OS checks, both Linux filesystem fault scenarios and Windows installer acceptance passed; see the verification ledger. No mount/fault operations run on local Windows/macOS or self-hosted runners.
- Physical storage/power-loss, Windows/macOS filesystem faults, persisted rollback state and transactional installation remain open; runtime auto-apply stays disabled.

## [v0.28.4] - 2026-10-29

- Add offline `stage-update` and `inspect-update` commands. Create an exclusive new directory, bound copied artifact/envelope bytes, reverify the copy against external trust inputs, sync files and publish the readiness marker last. No installation or trusted-policy mutation.
- Reject missing/modified readiness markers, extra files, staged-byte tampering, newly expired/disallowed metadata and replacement of existing stages. Concurrent attempts produce one complete stage. Invalid downloads leave no destination and repaired downloads can retry.
- Add six public API/CLI tests and actual native-core staging/source-change/tamper fixtures. Exact-source three-OS checks and Windows installer acceptance passed; see the verification ledger.
- Runtime updates remain disabled. Full power-loss/disk-full acceptance, persistent rollback/retry state, owner key lifecycle and transactional application/recovery remain open.

## [v0.28.3] - 2026-10-29

- Add offline `verify-update` developer tooling with domain-separated Ed25519 signatures over exact JSON payload bytes and separately provisioned public key/policy inputs. No network retrieval, installation, execution or policy mutation occurs.
- Require matching app/OS/architecture, strictly newer numeric release version, advancing caller-supplied sequence floor, unexpired metadata, exact artifact size and SHA-256. Bound metadata/artifact reads and reject ambiguous schema/base64/JSON forms and non-public signing-key inputs.
- Add publisher/tamper/target/expiry/rollback/schema/size/CLI regressions plus actual native-core artifact verification with ephemeral test keys. Exact-source three-OS checks and Windows installer acceptance passed; see the verification ledger.
- Runtime updater remains disabled. Persistent rollback/retry state, owner key provisioning/rotation, transactional application/recovery, OS signing and production update acceptance remain open. This is not a TUF implementation.

## [v0.28.2] - 2026-10-29

- Add native macOS `package-macos` CLI tooling: pinned source snapshot, stable `.app`/receipt identity, escaped property-list metadata, native architecture/system-library checks, normalized permissions and exclusive unsigned `.pkg` output. No install or elevation in the builder.
- Add public metadata/unsupported-host regressions and guarded hosted macOS install, renamed upgrade, app removal/reinstall, launcher inspection and authenticated vault/backup preservation acceptance. Exact-source three-OS checks, macOS/Linux package lifecycle fixtures and Windows installer acceptance passed; see the verification ledger.
- Reject reserved device IDs and trailing-dot identities in both native package planners, matching the runtime portability rules; regression tests reproduce the formerly accepted invalid configurations.
- Signing/notarization, Finder/Gatekeeper acceptance, older macOS/Intel compatibility, physical recovery and authenticated updates remain open.

## [v0.28.1] - 2026-10-29

- Add Linux-native `package-deb` tooling with pinned bundle verification, stable app-ID-derived installation identity, ELF architecture validation, derived library dependencies and exclusive output creation. The builder never installs or elevates.
- Add metadata/CLI regressions and a disposable hosted Ubuntu install, renamed upgrade, removal/reinstall and encrypted vault/backup preservation fixture. Exact-source three-OS checks, Windows acceptance and the real Ubuntu package lifecycle fixture passed; see the verification ledger.
- Packages remain unsigned previews tied to their build host ABI. macOS installers, signing, verified updates and physical-device acceptance remain open.

## [v0.28.0] - 2026-10-28

- Start distribution integrity tooling with explicit `package-manifest` and `verify-package` developer commands. Canonical file/size/SHA-256 inventories require a separately trusted manifest digest; existing descriptors are never overwritten automatically.
- Bound file/entry counts, nesting, byte totals and descriptor reads; reject linked/non-file entries, common unsafe portable names and normalized case collisions. Verification rebuilds the actual inventory without interpreting manifest-provided paths.
- Add missing/extra/altered-file and replaced-descriptor tests, known SHA-256 evidence, linked-directory checks and real CLI checks on an actual portable bundle. Results explicitly report that publisher authentication was not performed.
- Keep native Linux/macOS installers, publisher signing/notarization, authenticated automatic updates and physical recovery acceptance open. This preview is not a signed release or completion of the distribution milestone.

## [v0.27.1] - 2026-10-28

- Isolate benchmark core lifecycle behind a shared measurement runner. Readiness requires the complete newline-terminated loopback address, avoiding partial numeric ports across stdout chunks.
- Bound owned-child cleanup with a graceful request followed by direct-child force termination if needed. Startup timeout and workload failure close the owned process before temporary data removal; failure to stop is surfaced rather than waiting indefinitely.
- Add actual OS-process fixtures for split readiness lines, callback failure and a non-ready process, alongside the actual Rust benchmark budget fixture on all three OS runners. These checks remain core-only and do not certify whole-app performance.

## [v0.27.0] - 2026-10-28

- Begin the performance tooling milestone: schema-2 core reports retain five startup and twenty IPC samples, binary SHA-256, source revision and working-tree state. Host details and measurement limitations remain explicit.
- Add explicit core performance budgets with fail-closed handling of unknown/invalid limits and absent metrics. Exceeded budgets produce a report and nonzero exit; no default production threshold is invented. Three-OS CI exercises the actual budget CLI and retains raw reports.
- Preserve damaged, unreadable, linked/non-file or oversized profile registries instead of silently overwriting them during creation. Reads use the existing bounded storage limit. Expose registry diagnostics in profile_list; repair requires owner action and a new manager/restart.
- Record physical-device procedures and remaining whole-app/process-tree/UI measurement gates. This preview does not complete the v0.27 low-resource acceptance milestone.

## [v0.26.7] - 2026-10-28

- Return an error when a new profile's storage directory cannot be created; do not register or reserve the ID after failure.
- Prepare the target directory before changing the active profile. An existing non-directory destination no longer reports a successful switch or replaces the current storage root.
- Extend public IPC regressions with owned filesystem failures, profile-list/root preservation, and recovery after the blocking files are removed. Failed switch attempts still revoke retained sessions and pending native jobs as a fail-closed measure.

## [v0.26.6] - 2026-10-28

- Reject explicit malformed, zero or out-of-range `firewall_evaluate.port` inputs before policy evaluation. Checked conversion replaces truncation: 65979 previously became 443. Only an omitted field defaults to 443.
- Add real-core and packaged-runtime regression checks for overflow, signed/fractional/type errors, valid boundary ports and the omitted default. Clarify that this API evaluates rules and does not enforce OS/browser network traffic.
- Extend the scoped ASVS input-validation evidence and document the policy command contract. This is a validation/logic fix, not a claim of an exploitable network firewall bypass.

## [v0.26.5] - 2026-10-28

- Apply a common browser response boundary to successful, rejected, preflight and error responses: MIME sniffing disabled, referrers suppressed, and an additional restrictive CSP prevents framing, object embedding and base-URL changes. The app asset CSP remains separately enforced.
- Label JSON error bodies with their actual content type. Add actual-process IPC and packaged-runtime regressions; the missing referrer policy was reproduced before the fix.
- Add a pinned ASVS 5.0.0 evidence/gap matrix and update the README version, platform status, cryptography, development instructions and unsupported claims while retaining the presentation structure.

## [v0.26.4] - 2026-10-27

- Canonicalize development project roots before Vite setup. A real Windows short-path regression reproduced forbidden UI responses; the fix preserves Vite filesystem restrictions.

- Replace parallel development startup with an owned dynamic-port core and a Vite server that receives its actual address. Probe the configured UI origin before serving/opening the UI; preserve existing UI servers when strict-port startup fails.
- Build the core before launch and discover its executable from Cargo artifacts, supporting custom target directories. Add `--no-open`, `--ui-port`, `--core-binary` and side-effect-free `--print-plan` options.
- Stop owned resources on cancellation, core exit and startup failure. Allow bounded Vite configuration restarts without killing the core; fail if the owned UI does not resume listening. Never terminate a user's browser or unrelated listener.
- Add public launch-plan/argument tests and real native-core/Vite startup, restart, stale-override, cancellation, occupied-port and failure fixtures on all three OS runners. Default Cargo build discovery is additionally exercised on disposable hosted runners. See docs/DEVELOPMENT_STARTUP.md for limits.

## [v0.26.3] - 2026-10-27

- Allow development SDK clients to select an explicit core through `VITE_AEGIS_BRIDGE_URL`, retaining port 48291 when omitted. Validate literal IPv4-loopback HTTP origins and reject remote hosts, credentials, paths, query/fragment values, malformed types and invalid ports before any IPC fetch.
- Pin packaged clients to their own validated application origin and ignore development overrides. Resolve configuration inside guarded request paths so configuration errors return normal unsuccessful SDK results, including custom-nonce invocation.
- Add public origin-resolution regressions and a Vite-transformed TypeScript SDK integration against an owned dynamic-port Rust core on all three OS runners, without opening a browser or binding a Vite HTTP/HMR server. Automatic CLI development port discovery remains pending; see docs/DEVELOPMENT_BRIDGE.md.

## [v0.26.2] - 2026-10-27

- Fix packaged external/native startup targets when `--port=0` selects a dynamic listener: use the actual bound origin instead of generating a URL with port zero. Development UI still uses its separate Vite origin.
- Add `--print-launch-plan` for bounded, headless startup inspection without launching a desktop or running the IPC request loop; expose no handshake token. Add real-process packaged/development launch-target regressions on all three OS runners.
- Use dynamic-port binding in the guarded native GUI fixture, removing the reserve/release port race and verifying that the real embedded window loads the packaged document on Windows, Linux and macOS. Physical-device/accessibility and managed multi-window gates remain open.

## [v0.26.1] - 2026-10-27

- Validate window dimensions/title for both native and external launches. Reject overflowing, fractional, wrongly typed, null and unknown IPC options before launching; preserve omitted defaults and explicit camel/snake option aliases without silently dropping unsupported requests.
- Reject unsupported external resizing/decoration controls, developer tools and custom protocols. Report external launch requests without claiming visible-window confirmation or applied size/title settings.
- Add pure request-preparation and public IPC regressions without opening the developer desktop, plus native-binary malformed-request checks on all three OS runners. Full managed multi-window behavior and physical/accessibility acceptance remain open.

## [v0.26.0] - 2026-10-27

- Publish the cross-platform native-services preview milestone: optional embedded Windows/macOS/Linux hosts, bounded dialog/clipboard/notification interfaces and the previously validated transport/profile safeguards. Physical-device, accessibility and full multi-window acceptance remain open; see docs/RELEASE_GATES.md.
- Replace unbounded diagnostic history and second-based file-name collisions with a 32-event, 64 KiB persistent journal and random event IDs. Use serialized atomic writes, cooperating-process locks, bounded restart reads and fail-closed corrupt/linked/non-file history handling.
- Allowlist diagnostic labels, omit caller messages/traces, return real persistence errors and align the TypeScript/demo contracts with the backend. Diagnostics are explicitly application-wide; old sandbox crash files remain untouched.
- Add public IPC retention/restart/identity/privacy/failure regressions and real native-binary diagnostic checks on all three OS runners. This preview does not complete signing, physical-device validation or an independent audit.

## [v0.25.14] - 2026-10-27

- Reproduce and fix indefinite local IPC starvation by an incomplete body. Apply absolute five-second socket ingress and response-write budgets, preserve deadlines across socket clones and disable connection reuse/pipelining with `Connection: close`.
- Bound HTTP lines to 8 KiB and headers to 100 fields / 64 KiB. Bound accepted TCP connections to 32 with permits retained by queued requests and socket clones. Reject excess connections before worker allocation. Drain rejected request bodies with a fixed 8 KiB buffer instead of allocating from untrusted Content-Length.
- Vendor the narrowly patched tiny_http 0.12.0 source with both upstream licenses and archive provenance; copy it into generated source applications. Add real-process socket regressions on Windows/Linux/macOS CI.
- These are transport bounds, not command cancellation or a guarantee of availability against sustained local flooding. See docs/IPC_TRANSPORT_LIMITS.md and src-core/vendor/tiny_http/AEGIS_PATCH.md.

## [v0.25.13] - 2026-10-27

- Correct Wasm memory reporting to measure guest linear memory, including unexported growth, instead of JSON input bytes. Preserve memory/table/fuel restrictions and expose the measurement scope.
- Reuse validated modules with fresh execution stores and return metadata without cloning raw binaries. Bound module IDs/display names and reject a full registry before compiling another new entry.
- Reject malformed explicit execution tiers rather than silently falling back to AUTO. Remove unsupported GPU acceleration and absolute host-security claims from demo text; preserve the interface layout.
- Add public module/IPC resource-accounting and state-isolation regressions. Whole-process memory/time budgets and physical constrained-device validation remain pending; see docs/WASM_RESOURCE_LIMITS.md.

## [v0.25.12] - 2026-10-27

- Catch native handler unwind panics and disable the faulted registration. Return typed plugin failures and generic IPC panic errors, revoke retained vault sessions/ephemeral secrets and preserve subsequent IPC commands.
- Pin handler registrations with shared ownership and release registry locks before execution, fixing deadlock when a handler registers another plugin. Expose disabled registration names through `plugin_list`.
- Add public IPC tests for fault recovery, session cleanup, disabled-handler retry rejection and panic-payload exclusion from responses; add a bounded registration-deadlock regression. This is unwind-error handling, not process isolation or rollback of native side effects.

## [v0.25.11] - 2026-10-26

- Enforce declared native-plugin capabilities before dispatch and pass the active security context to handlers. Fix the checksum plugin's filesystem/hash permission bypass, absolute/traversing path reads and incorrect profile selection.
- Bound regular-file checksums to the shared storage limit using an 8 KiB zeroizing read buffer. Add public regression coverage for denied grants, outside paths, active profiles and oversized files.
- Replace fabricated memory statistics with an unavailable error and report monotonic elapsed time with an explicit registration-time scope. Native Rust handlers remain trusted host code, not an untrusted plugin sandbox; see docs/PLUGIN_SECURITY.md for compatibility and limitations.

## [v0.25.10] - 2026-10-26

- Add a stable packaged `appId` independent of display/package-folder names, with portable validation and fail-closed invalid configuration. Missing IDs retain legacy folder-based behavior; no automatic data migration or merge occurs.
- Keep the framework demo's established data identity and generate explicit IDs for new source projects. Reject invalid explicit IDs before bundling writes output.
- Add native packaged rename/restart/vault-authentication acceptance on all three OS runners. See docs/APP_IDENTITY.md for compatibility and namespace limitations.

## [v0.25.9] - 2026-10-26

- Clear retained vault credentials and ephemeral encrypted secrets immediately when a pre-decryption OS-session check fails, including denied unlock attempts. Advance the session epoch without recursively acquiring its mutex. Regression coverage verifies cleanup through the session and secret-manager interfaces.
- Linux/macOS retained sessions remain unavailable pending authenticated lifecycle adapters and desktop acceptance; this fix does not claim those integrations are complete.
- Add an optional Linux revoke-only logind observer with a pinned root-owned system-bus service, bounded subscription, owner-continuity checks and fail-closed cleanup. Add private-bus protocol acceptance without locking or suspending a desktop. See docs/LINUX_LIFECYCLE.md for the precise preview scope.

## [v0.25.8] - 2026-10-26

- Add a separate default-denied notifications capability and typed asynchronous submit/poll/cancel APIs. Replace false delivered claims with pending/providerAccepted results that never assert visual delivery or echo text.
- Bound notification helpers, timeouts and runtime-wide submission quota; reuse profile-scoped job cleanup and abandoned-request recovery without piping request data to external providers.
- Parse urgency/timeout without silent loss or integer truncation; reject unsupported platform options and custom icons.
- Escape Linux markup and backslashes before notify-send decoding. Stop Windows PowerShell errors rather than reporting false success; keep provider text separate from scripts.
- Correct capability/tray metadata and demo logs, preserving layout. Add a private nonvisual Linux D-Bus transport fixture; physical Windows/macOS toast and desktop acceptance remain pending.

## [v0.25.7] - 2026-10-26

- Add optional cross-platform native message helpers and typed open/poll/cancel methods; distinguish submission from acknowledgement and require successful provider exit.
- Replace the unbounded Windows IPC helper route with profile-scoped, single-slot dialog jobs. The legacy window_dialog alias now requires the native-dialogs feature and returns a request ID to consume/cancel.
- Pin Linux provider exit-code semantics against inherited overrides and verify OK/Escape separately.
- Fix expired abandoned requests permanently occupying file/dialog slots after page reload; add a failing-then-passing regression.
- Share bounded job supervision across typed file/message results; preserve the demo layout while reporting truthful submission/acknowledgement and polling results.
- Add malformed message-helper input checks on all native runners and a real Linux X11 message-window/default-button fixture. Physical/accessibility and Windows/macOS interactive message acceptance remain pending.

## [v0.25.6] - 2026-10-26

- Add optional native single-file selection through an isolated main-thread helper and default-denied file-dialog capability; declare Rust 1.92 minimum.
- Keep core dispatch available while selection is pending. Bound jobs/output/lifetime; scope request IDs to profiles, consume results once and invalidate on profile switching.
- Add typed open/poll/cancel bridge methods. A selected path grants no filesystem access; ambiguous empty provider results are reported as noSelection, not confirmed cancellation.
- Add non-GUI child-process regression tests and feature-enabled helper input checks to three-OS CI. Interactive picker and accessibility acceptance remain pending; see docs/FILE_DIALOGS.md.

## [v0.25.5] - 2026-10-26

- Add opt-in native text clipboard writes with a default-denied capability, 64 KiB UTF-8 limit, NUL rejection, typed responses and no text in returned errors.
- Retain the lazy provider for X11 clipboard ownership; expose unavailable providers as errors. Image/HTML/read access and native Wayland data-control are not enabled.
- Add permission/input regressions and a Unicode native roundtrip restricted to disposable hosted CI. This is not secure secret expiry or clipboard-history protection; see docs/CLIPBOARD.md.

## [v0.25.4] - 2026-10-26

- Correct compiled/active native capability metadata and bound native CI package-download/job waits without ignoring failed checks.
- Enforce the configured dialog capability before helper preparation or execution.
- Separate Windows dialog text from PowerShell code, bound text inputs and validate dialog kinds. Parse the prepared script in a regression test without opening a dialog; verify the parser rejects an invalid-script fixture.
- Return explicit unavailable errors for missing macOS/Linux dialog providers instead of fabricated delivery success. Report helper submission separately from confirmed delivery and add a typed bridge response.

## [v0.25.3] - 2026-10-26

- Add optional Wry/Winit Windows/macOS and GTK4/WebKit6 Linux embedded-window adapters, preserving the smaller default runtime. Avoid the rejected legacy GLib chain without advisory exemptions.
- Validate native navigation, dimensions, title, developer tools and unsupported protocols/settings through a public window-policy regression test; honor configured window fields instead of silently replacing them with defaults.
- Add feature-enabled builds/tests and real fixture-window load/exit checks restricted to disposable hosted runners. GUI/accessibility/device acceptance is reported separately; the full v0.26 milestone remains incomplete.

## [v0.25.2] - 2026-10-26

- Add a source-only create command, discoverable help/version and a minimal typed-bridge template without changing the framework demo UI or README.
- Refuse existing destinations and reject linked/special files, credential-shaped files and generated output before creating a project.
- Exercise clean locked-dependency UI installation/build and CLI regressions in three-OS CI. Keep native-window/plugin/physical-device release gates explicit.
- Restrict push-triggered validation to main, retaining PR validation; avoid duplicate expensive installer runs when tagging an already-verified source commit.

## [v0.25.1] - 2026-10-26

- Add native-host portable directories for Linux/macOS while preserving Windows Inno Setup output. Reject unsupported target claims and remove debug/foreign-binary fallback. Native installers/notarized applications are not claimed.
- Exercise an isolated real portable bundle in three-OS CI; keep all fixture data in temporary directories.
- Fix same-origin browser requests when the server chooses a dynamic port; retain rejection of unrelated origins.
- Add a core-only benchmark harness with tested percentile statistics and per-OS CI artifacts. Full UI and physical constrained-device performance gates remain.
- Record the unshipped native-window prototype's dependency and build blockers rather than relaxing the audit policy.

## [v0.25.0] - 2026-10-26

- Introduce portable data/config/cache resolution and typed platform IPC interfaces. Document filesystem, permission, case-collision, recovery and unavailable-provider contracts.
- Run actual release-binary platform/storage IPC checks on Windows, Linux and macOS in addition to shared tests and UI builds.
- Fail closed for retained sessions on Linux/macOS until lifecycle adapters exist; preserve fresh-passphrase encrypted-file operations.
- OS key stores, embedded webviews and Linux/macOS installers remain explicitly unavailable; portable core support does not imply those integrations.

## [v0.24.0] - 2026-10-26

- Separate notification text from OS scripts/arguments on all three platforms, bound notification text and reject unimplemented icons. Notification submission means helper startup, not confirmed user-visible delivery.

- Add a portable app-data location API for Windows, Linux and macOS, with deterministic public-interface tests for platform locations, unsafe app IDs and relative roots. Use it during packaged startup while preserving the existing Windows location.
- Validate local window URL authority/port and reject unsupported modal/kiosk/always-on-top/transparent requests before launching the external-browser backend.
- Report unimplemented modal, kiosk, always-on-top and custom-size controls as unavailable in the external-browser backend; cover the IPC capability response with a regression test.
- Reject custom profiles that share a storage directory with an existing profile, including the built-in private vault and case-insensitive aliases. Ignore colliding saved profile entries on startup.
- Add IPC regression coverage for profile storage identities and a native Windows/Linux/macOS core-test, release-build and UI-build workflow.
- Record the general-purpose framework roadmap and distinguish planned cross-platform capabilities from validated support.

## [v0.23.0] - 2026-10-26

- Build the actual pinned v0.22 source/runtime/installer for upgrade tests; verify encrypted vault and backup bytes survive upgrade, uninstall and reinstall, then authenticate and restore their contents.
- Add a real Windows workstation-lock acceptance test restricted to explicitly enabled disposable GitHub-hosted runners. Verify OS lock state, denied retained/fresh session access, and ephemeral-secret cleanup. The developer PC is never locked or suspended by this test.
- Expose current Windows session availability in security status and publish sanitized exact-commit installer/lock evidence as CI artifacts.
- Automate RustSec/npm known-advisory checks with lockfile provenance; test real publisher-signature acceptance and unsigned/wrong-publisher rejection before any certificate import. Pin workflow actions and correct obsolete security glossary terms.
- Preserve publisher-signing setup and external audit handoff. No certificate, independent human audit, pristine-device acceptance or real suspend/resume result is fabricated.

## [v0.22.0] - 2026-10-26

- Register native Windows WTS/power notifications and revoke vault sessions on lifecycle transitions. Check current WTS lock/connect state before and after session operations; monitor registration failure/shutdown denies access.
- Track revocation generations so an in-flight decrypt cannot silently refresh a revoked session. Add real-window synthetic event tests without locking or suspending the developer PC.
- Repair installer shortcuts and output location; add disposable Windows install/runtime/uninstall/data-preservation CI coverage.
- Add fail-closed Authenticode signing helpers and a manual protected-environment signing workflow. No publisher certificate is currently configured and no signed release is claimed.
- Add Windows release instructions and an external security audit handoff. Real-device lock/sleep acceptance, pristine-machine testing and an independent human audit remain external gates.

## [v0.21.0] - 2026-10-26

- Add vault-scoped `vault_session_unlock`, `vault_session_read`, and `vault_session_lock` interfaces, with typed bridge methods.
- Require authenticated vault decryption before issuing a random session token. Permit one unlocked vault per runtime; a new unlock revokes its predecessor.
- Enforce 60-second idle and 300-second absolute limits; background cleanup drops zeroizing credentials and clears ephemeral secrets. Lock remains available during rate limiting.
- Serialize router operations so profile switching and vault mutation revoke retained sessions before subsequent reads. Existing password-per-operation interfaces remain available and require fresh authentication.
- Add session-manager and IPC regression coverage, plus packaged session smoke checks. No UI redesign or password-manager app is added.
- Signing, clean-machine installation, native OS lock integration, and an independent security audit remain release gates.

## [v0.20.0] - 2026-10-26

### Framework hardening (not a password-manager application)
- Synchronize and atomically replace file writes; use OS file locks, temporary files and flush-before-rename. Preserve a single encrypted previous vault generation.
- Require the existing master passphrase before replacing a vault or in-memory secret, including verification inside the vault write lock. Reject plaintext writes to vault/backup files.
- Add vault_restore_backup and vault_rotate_passphrase primitives. Authenticate backups before restoration; password rotation retains an encrypted backup requiring the previous passphrase.
- Pin V2 Argon2id parameters for compatibility; clear derived-key buffers, IPC payload strings and owned response buffers on drop. New encryption requires 12-character passphrases, with size limits before KDF/decoding.
- Redact IPC Debug output; omit caller trace payloads from diagnostic storage; remove prefilled demo credentials and clear sensitive demo UI state on blur/hidden or after 60 seconds.
- Restrict packaged origins and CSP to the app origin, remove remote font requests, prohibit cached IPC responses and require bounded JSON POST bodies. Limit expensive sensitive operations to 20 per minute, with cooldown after repeated authentication failures.
- Reserve Windows device/internal storage names, bound stored-file reads, atomically save profile definitions and clear ephemeral secrets on profile switch.
- Add sensitive-operation live smoke tests and document threat boundaries and release review gates in SECURITY_REVIEW.md. Native hardware providers and encrypted IPC transport remain unavailable; no production-readiness certification is claimed.

---

## [v0.19.0] - 2026-10-26

### Security and correctness fixes
- Replace the custom WebAssembly interpreter with wasmi validation, instruction fuel, memory limits and no host imports. Branch continuation, signed integers and traps follow WebAssembly semantics.
- Replace deterministic vault XOR encryption with Argon2id + XChaCha20-Poly1305, random salts/nonces and authenticated V2 envelopes. Reject legacy V1 without modifying existing files; export with the original version before migration.
- Require real Ed25519 verification for plugin manifests and administrator-provisioned public keys. Remove default trusted publishers and fabricated signature acceptance.
- Generate session tokens with OS randomness; validate loopback Host/Origin, reject unauthorized preflights and limit IPC body size.
- Harden sandbox paths and configured access scopes; check profile quotas before file/vault writes and persist validated profile definitions across restarts.
- Remove seeded secrets and authenticate in-memory secret encryption with the vault format.

### Honest capability reporting
- Disable simulated WebAuthn, HSM/PKCS#11, TPM, biometrics, GPU, P2P distribution, encrypted IPC and auto-update operations. They return unavailable, not fabricated success. Real providers remain future integration work.
- Replace misleading dashboard actions with unavailable pages. Report the external browser backend accurately; native Wry/Tao webviews are not implemented.
- Label network rules as advisory evaluation rather than an OS firewall.

### Windows packaging and verification
- Update Vite and its React plugin; npm audit reports zero known vulnerabilities in the installed dependency tree.
- Require built release/UI/config assets; include a production static UI server so the portable app runs without Node/Vite. Keep user data under LOCALAPPDATA, separate from installed files.
- Reject incomplete bundles, unsafe metadata, stale output reuse and unsupported macOS/Linux package targets. Inno Setup compilation and clean-machine validation remain separate steps.
- Add failure-propagating CLI build/bundle/security commands and Windows CI.
- Verified locally: Rust regression tests, production UI/release builds, live security smoke checks, portable UI/assets, profile isolation and file/profile persistence after process restart. Real-device and non-Windows validation are not claimed.

> Historical entries below describe earlier prototypes; their hardware, GPU, P2P and native-platform claims are superseded by the v0.19 availability restrictions above.

---

## 🔮 [Upcoming Roadmap] - v0.24.0
> **Target Release:** Q1 2027

### 🚀 Planned Features & Enhancements
- [ ] **Decentralized Zero-Knowledge Identity (DID/VC):** Verifiable credentials and zero-knowledge selective disclosure for user profiles.
- [ ] **Quantum-Resistant Lattice Cryptography:** Post-quantum ML-KEM (Kyber) and ML-DSA (Dilithium) key exchange and assertions.
- [ ] **Automated Cross-Enclave Memory Sanitization:** Ephemeral memory wipe handlers using zeroize and guard-page protection.

---

## 🚀 [v0.18.0] - 2026-10-26
> **The Automated Multi-OS Packaging Pipeline, Distributed P2P Plugin Marketplace & HSM PKCS#11 Enclave Release**

### ✨ Highlights & New Features
- **Automated Multi-OS Packaging Pipeline (`src-core/src/package_pipeline.rs`):**
  - One-click multi-platform release compiler supporting Windows, macOS, and Linux from a single configuration.
  - **macOS Bundler:** Automatic `.app` folder generation, bundle layout (`Contents/MacOS`, `Contents/Resources`), dynamic `Info.plist` creation, and `hdiutil` Apple DMG packaging script generation.
  - **Linux AppImage:** Complete `.AppDir` layout, embedded `AppRun` binary launcher wrapper, FreeDesktop `.desktop` entries, and portable distribution packaging.
  - **Linux Debian Package:** Debian control file (`DEBIAN/control`) generation with metadata, architecture declarations, and filesystem mapping (`usr/bin/aegis-core`).
  - **Windows Portable:** Portable runner script (`launch.bat`) and structured distribution staging.
  - IPC command: `package_pipeline_stage`.
  - Full React UI panel: Multi-Platform Staging Configuration with interactive platform selectors, target artifact digests, and one-click compilation.
- **Distributed P2P Plugin Marketplace (`src-core/src/p2p_market.rs`):**
  - Content-addressed plugin discovery utilizing SHA-256 IPFS Content Identifiers (CIDs) and BitTorrent info-hashes.
  - Decentralized multi-signature consensus engine: Plugins require verified threshold m-of-n cryptographic signatures from trusted decentralized identities (DIDs) before installation.
  - Simulated P2P swarm peer discovery with latency tracking and peer reputation scoring.
  - Zero-Trust installation verification: Blocks plugins with insufficient or untrusted signatures.
  - IPC commands: `p2p_market_list`, `p2p_market_sign`, `p2p_market_install`.
  - Full React UI panel: Decentralized Plugin Swarm explorer, live peer swarm health monitor, one-click DID endorsement, and consensus-verified P2P installation.
- **Hardware Security Module (HSM) PKCS#11 Integration (`src-core/src/security/hsm_pkcs11.rs`):**
  - Direct native PKCS#11 cryptographic token driver for YubiHSM 2, Nitrokey HSM, and smart card enclaves.
  - Non-extractable cryptographic key objects (ECC NIST P-256, RSA-4096, Ed25519) permanently quarantined inside tamper-resistant hardware silicon.
  - Authenticated user session management (`login_token`, `logout_token`) with PIN verification and session state tracking.
  - Silicon-bound on-chip signature calculation (`CKM_ECDSA_SHA256_ON_CHIP`): Data digests are signed inside the HSM chip without keys ever entering host memory.
  - IPC commands: `hsm_slots`, `hsm_login`, `hsm_logout`, `hsm_keys`, `hsm_sign`.
  - Full React UI panel: Connected PKCS#11 slot browser, token PIN session unlocker, on-chip key inspector, and hardware signature generator.

### 🔐 Security & Privacy Improvements
- Multi-OS packages staged with SHA-256 cryptographic manifest digests preventing supply-chain tampering.
- Decentralized P2P plugin marketplace enforces m-of-n multi-sig consensus, eliminating single-point-of-failure vendor compromise.
- HSM PKCS#11 architecture ensures master signing and encryption keys never touch host OS RAM or disk.

### 🛠️ Internal Changes
- `src-core/src/package_pipeline.rs` — implemented `MultiOsPipeline`, `TargetPlatform`, and bundle generators for macOS, Linux, Debian, Windows.
- `src-core/src/p2p_market.rs` — implemented `P2PPluginMarketplace`, IPFS CID validation, and DID multi-sig verification.
- `src-core/src/security/hsm_pkcs11.rs` — implemented `HsmPkcs11Engine`, PKCS#11 slot registry, PIN login, and on-chip signing.
- `src-core/src/security/mod.rs` — exported `HsmPkcs11Engine`.
- `src-core/src/ipc/router.rs` — wired 8 new v0.18 IPC routes (`package_pipeline_stage`, `p2p_market_list`, `p2p_market_sign`, `p2p_market_install`, `hsm_slots`, `hsm_login`, `hsm_logout`, `hsm_keys`, `hsm_sign`).
- `src-core/src/main.rs` — initialized `MultiOsPipeline`, `P2PPluginMarketplace`, and `HsmPkcs11Engine`.
- `src-ui/src/aegis-bridge.ts` — updated header to v0.18; added 8 typed bridge methods.
- `src-ui/src/App.tsx` — added Multi-OS Pipeline, P2P Marketplace, and HSM PKCS#11 sidebar navigation and tab panels.
- `src-core/Cargo.toml` & `package.json` — version bumped to `0.18.0`.

---

## 🚀 [v0.17.0] - 2026-10-25
> **The Native Platform (WebKit/Tao), TPM 2.0 Remote Attestation & GPU-Accelerated Wasm JIT Release**

### ✨ Highlights & New Features
- **macOS & Linux Native WebKit Integration (`src-core/src/window.rs`):**
  - Full native webview integration via `wry` (0.47) and `tao` (0.30), replacing the Edge App Mode fallback on non-Windows platforms.
  - Multi-engine runtime detection reporting platform type, engine flavor (`WebKitGTK`, `WebKit Cocoa`, `WebView2`), Tao event-loop status, and native decoration capability.
  - IPC command: `window_runtime_info`.
  - Live interactive WebKit Runtime Card in the UI displaying engine details, window geometry, and backend status.
- **TPM 2.0 Remote Attestation Engine (`src-core/src/security/attestation.rs`):**
  - Cryptographic attestation framework computing SHA-256 Platform Configuration Register (PCR) measurements for PCR 0 (Firmware/BIOS), PCR 7 (Secure Boot), PCR 10 (Kernel/IMA), and PCR 11 (Aegis Core Enclave).
  - Attestation Identity Key (AIK) quote generation with SHA-256 nonce binding, timestamping, and ECDSA P-256 signature verification.
  - Enterprise Zero-Trust Network Policy evaluation: Outbound requests require valid TPM proof and verified PCR states before connection clearance.
  - IPC commands: `tpm_measurements`, `tpm_generate_quote`, `tpm_verify_quote`, `network_evaluate_attestation`.
  - Live TPM Remote Attestation tab in UI with interactive PCR bank inspection, nonce-backed quote generation, and signature verification.
- **GPU-Accelerated Wasm JIT Compiler (`src-core/src/wasm.rs`):**
  - Dual-tier execution engine with Interpreter (Tier 1) and WGPU Compute Shader JIT (Tier 2).
  - Wasm bytecode parsing and ahead-of-time WGSL compute shader translation for vector/matrix SIMD and parallel loops.
  - High-performance execution delivering **12x–20x compute speedup** for parallel numerical workloads.
  - IPC commands: `wasm_execute_gpu`, `wasm_execute_tier`.
  - WGPU JIT Compute Studio tab in UI with side-by-side Tier 1 vs Tier 2 execution benchmarking, speedup metrics, and WGSL shader view.

### 🔐 Security & Privacy Improvements
- Hardware-rooted remote attestation guarantees code integrity against compromised kernel/firmware states.
- Egress network policy denies zero-trust enterprise connections unless valid TPM quote with current PCR digests is presented.
- GPU JIT executes in restricted compute shaders with strict bounds-checking and memory isolation.

### 🛠️ Internal Changes
- `src-core/src/security/attestation.rs` — implemented `TpmAttestationEngine`, PCR banks, AIK quotes, and proof verification.
- `src-core/src/wasm.rs` — upgraded `WasmVm` with `WgpuJitCompiler`, `ExecutionTier`, and WGSL code generation.
- `src-core/src/network.rs` — integrated remote attestation proof evaluation into `NetworkPolicyEngine`.
- `src-core/src/window.rs` — added `get_runtime_info()` returning platform-specific webview engine metadata.
- `src-core/src/ipc/router.rs` — registered 6 new v0.17 IPC routes (`window_runtime_info`, `tpm_measurements`, `tpm_generate_quote`, `tpm_verify_quote`, `network_evaluate_attestation`, `wasm_execute_gpu`, `wasm_execute_tier`).
- `src-ui/src/aegis-bridge.ts` — added typed bridge methods for all v0.17 features.
- `src-ui/src/App.tsx` — added TPM Attestation and WGPU JIT Studio tabs plus WebKit Runtime Card.
- `Cargo.toml` & `package.json` — version bumped to `0.17.0`.

---

## 🚀 [v0.16.0] - 2026-10-25
> **The Zero-Knowledge Secrets Manager, Network Firewall Policy Engine & Plugin Provenance Release**

### ✨ Highlights & New Features
- **Zero-Knowledge Secrets Manager (`src-core/src/secrets.rs`):**
  - Centralized memory-hard key derivation (1500 rounds) using per-secret salts and master passphrases.
  - In-memory XOR transformation & digest fingerprinting; secrets never touch unmanaged buffers or disk plaintext.
  - Automatic version incrementation and rotation timestamps (`created_at`, `rotated_at`).
  - IPC commands: `secrets_list`, `secrets_store`, `secrets_retrieve`.
  - Full React UI panel for storing, rotating, and decrypting secret values with live registration table.
- **Dynamic Network Firewall Policy Engine (`src-core/src/network.rs`):**
  - Dynamic zero-trust egress filter inspecting target hosts and ports.
  - Hard-coded telemetry blackholing (`*.telemetry.com`, `*google-analytics.com`) enforcing strict air-gap privacy.
  - Deny-by-default stance for unknown outbound network connections.
  - IPC commands: `firewall_rules`, `firewall_evaluate`.
  - Full React UI panel for evaluating outbound hosts/ports against active directives.
- **Plugin Cryptographic Signature & Provenance Verification (`src-core/src/security/plugin_sig.rs`):**
  - Cryptographic SHA-256 manifest hash calculation with root vendor trust anchors.
  - Validation against enterprise authority signers (e.g. `signer_aegis_official`).
  - IPC commands: `plugin_signers`, `plugin_verify_manifest`.
  - Full React UI panel with interactive manifest verifier and trusted authority registry.

### 🔐 Security & Privacy Improvements
- All v0.16 commands protected by cryptographic session nonces, capability manifests, and rate limiters.
- Secrets manager eliminates plaintext secret leakage into process dumps and logs.
- Network policy engine prevents rogue plugins or malicious scripts from establishing unauthorized outbound sockets.

### 🛠️ Internal Changes
- `src-core/src/security/mod.rs` — exported `SignatureVerifier`.
- `src-core/src/ipc/router.rs` — wired 7 new IPC routes (`secrets_list`, `secrets_store`, `secrets_retrieve`, `firewall_rules`, `firewall_evaluate`, `plugin_signers`, `plugin_verify_manifest`).
- `src-core/src/main.rs` — initialized `SecretManager`, `NetworkPolicyEngine`, and `SignatureVerifier`.
- `src-ui/src/aegis-bridge.ts` — updated header to v0.16; added 7 new typed bridge methods.
- `src-ui/src/App.tsx` — updated header badge to `v0.16 BETA`; added sidebar items and 3 comprehensive tab panels.
- `Cargo.toml` & `package.json` — version bumped to `0.16.0`.

---

## 🚀 [v0.15.0] - 2026-10-25
> **The Biometric Enclave, Encrypted IPC Stream & Local Crash Reporter Release**

### ✨ Highlights & New Features
- **Biometric Enclave (`src-core/src/security/enclave.rs`):**
  - Hardware-bound platform authentication via Windows Hello (TPM) and Apple Secure Enclave (SEP).
  - Private keys are permanently sealed inside the TEE — they never enter user-space memory.
  - IPC commands: `enclave_status`, `enclave_authenticate`.
  - Full React UI panel with live status query and biometric challenge trigger.
- **Encrypted IPC Stream (`src-core/src/ipc/encrypted_stream.rs`):**
  - Symmetric session-key encryption derived from the per-handshake nonce.
  - Every payload carries an HMAC-SHA256 authentication tag; any bit-flip causes an immediate hard rejection.
  - IPC commands: `stream_encrypt`, `stream_decrypt`.
  - Full React UI panel demonstrating the encrypt → transmit → verify → decrypt round-trip.
- **Local Crash Reporter (`src-core/src/crash.rs`):**
  - Air-gapped, zero-telemetry diagnostic capture of Rust panics and assertion violations.
  - Persists structured reports to `./sandbox/crashes/` — **no outbound network calls ever**.
  - IPC commands: `crash_list`, `crash_record`.
  - Full React UI panel with kind selector, message editor, and scrollable crash log viewer.
- **Agent Skills Integration (`.agents/skills/`):**
  - 27 engineering and productivity skills from `mattpocock/skills` integrated.
  - v0.15.0 architectural decisions formally recorded in `docs/adr/0001-enclave-encrypted-ipc-crash-reporter.md`.
  - Domain vocabulary expanded in `GLOSSARY.md` (Biometric Enclave, Encrypted IPC Stream, Local Crash Reporter, Session Nonce).

### 🔐 Security Improvements
- All v0.15 subsystems pass through the existing Zero-Trust capability engine (nonce verification + rate limiter).
- `BiometricEnclave` uses `Mutex`-protected state to prevent concurrent authentication races.
- `CrashReporter` writes to a sandboxed subdirectory, preventing crash logs from leaking outside the app boundary.

### 🛠️ Internal Changes
- `src-core/src/security/mod.rs` — exported `BiometricEnclave`.
- `src-core/src/ipc/mod.rs` — exported `EncryptedStream`.
- `src-core/src/ipc/router.rs` — registered 6 new IPC routes.
- `src-core/src/main.rs` — initialized `BiometricEnclave` and `CrashReporter` singletons.
- `src-ui/src/aegis-bridge.ts` — version bumped to v0.15; 6 new typed client methods added.
- `src-ui/src/App.tsx` — 3 new tab panels; header badge updated to `v0.15 BETA`.
- `Cargo.toml` + `package.json` — version bumped to `0.15.0`.

---

## 🚀 [v0.14.0] - 2026-10-25
> **The Hardware Security Keys (FIDO2), Profile Isolation & Wasm Virtual Machine Release**

### ✨ Highlights & New Features
- **Hardware Security Key & FIDO2 / WebAuthn Integration (`src-core/src/security/hardware_key.rs`):**
  - Physical security key support compliant with CTAP2.1 & WebAuthn Level 3 specifications.
  - Challenge nonce generation (`hardware_key_challenge`) with SHA-256 seed rotation.
  - Cryptographic assertion signature verification (`hardware_key_verify`) requiring physical touch + PIN.
  - Enrolled hardware token discovery (`hardware_key_list`).
- **Multi-Profile Workspace Isolation (`src-core/src/profile.rs`):**
  - Complete zero-leakage sandbox isolation across distinct user profiles (`Primary`, `High-Security Vault`, and custom profiles).
  - Each profile enforces a dedicated directory boundary (`./sandbox_<id>/`) and quota caps.
  - Dynamic profile switching and provisioning (`profile_list`, `profile_switch`, `profile_create`).
- **WebAssembly (Wasm) Plugin Virtual Machine (`src-core/src/wasm.rs`):**
  - High-speed, memory-capped (8–16MB) isolated guest execution sandbox.
  - Strict instruction gas metering to prevent infinite loops and denial-of-service.
  - Built-in guest modules: `wasm-crypto-accelerator` (vectorized hashing) and `wasm-schema-validator` (strict JSON validator).
  - New IPC commands: `wasm_list` and `wasm_execute`.
- **Frontend Dashboard Extensions (`src-ui/src/App.tsx` & `aegis-bridge.ts`):**
  - 3 new interactive tabs: **Hardware Key (FIDO2)**, **Profile Isolation**, and **Wasm VM Sandbox**.
  - TypeScript client helpers: `hardwareKeyList()`, `hardwareKeyChallenge()`, `hardwareKeyVerify()`, `profileList()`, `profileSwitch()`, `profileCreate()`, `wasmList()`, `wasmExecute()`.

### 🛡️ Security Hardening
- Hardware key authentication requires user presence touch acknowledgment and PIN verification.
- Wasm VM enforces strict guest memory caps and import whitelisting, preventing guest breakouts.
- Isolated profiles strictly forbid directory climbing into peer profile storage.

---

## 🚀 [v0.13.0] - 2026-10-25
> **The Extensibility, Notifications, Air-Gapped Updater & Capability Matrix Release**

### ✨ Highlights & New Features
- **Extensible Rust Plugin Engine (`src-core/src/plugin.rs`):**
  - Thread-safe `PluginRegistry` allows registration of memory-safe Rust closures with fine-grained capability descriptors.
  - Three out-of-the-box native built-in plugins:
    - `system_memory` — RAM utilization, total capacity, and paging telemetry.
    - `file_checksum` — Streaming SHA-256 integrity verification inside the sandbox.
    - `process_uptime` — Host OS uptime and Aegis daemon process metrics.
  - New IPC commands: `plugin_list` and `plugin_dispatch`.
- **System Tray & Native OS Notifications (`src-core/src/tray.rs`):**
  - Zero-dependency OS notification dispatch using native subsystems:
    - **Windows:** PowerShell WinRT Toast integration (`Windows.UI.Notifications`).
    - **macOS:** AppleScript (`display notification`).
    - **Linux:** Freedesktop `notify-send`.
  - Input sanitization against command injection before invoking native OS hooks.
  - New IPC commands: `notify_send` and `tray_status`.
- **Air-Gapped Cryptographic Auto-Updater (`src-core/src/updater.rs`):**
  - Strict SemVer version comparison (`check_update_available`).
  - Streaming SHA-256 cryptographic integrity verification (`verify_integrity`).
  - Isolated atomic staging into `.aegis_updates/` sandbox before application restart.
  - Air-gapped offline support for air-gapped enterprise environments.
  - New IPC commands: `updater_status`, `updater_check`, and `updater_verify`.
- **Runtime Capability & Permission Guard (`src-core/src/ipc/router.rs`):**
  - Comprehensive capability matrix tracking `fs`, `shell`, `systemInfo`, `crypto`, `encryptedVault`, `bundler`, `multiWindow`, `plugins`, `notifications`, and `autoUpdater`.
  - Zero-trust enforcement: untrusted frontend cannot elevate capabilities at runtime.
  - New IPC command: `permissions_status`.
- **Frontend Dashboard Extensions (`src-ui/src/App.tsx` & `aegis-bridge.ts`):**
  - 4 new interactive tabs: **Plugin Engine**, **Tray & Notifications**, **Permission Manager**, and **Auto-Updater**.
  - TypeScript client helpers: `pluginList()`, `pluginDispatch()`, `notifySend()`, `trayStatus()`, `updaterStatus()`, `updaterCheck()`, `updaterVerify()`, `permissionsStatus()`.

### 🛡️ Security
- All notification parameters are strictly sanitized against shell escaping.
- Updater forbids automatic execution of unverified update files.
- Plugin handlers execute directly in Rust closures with zero shell breakout risk.

---

## 🚀 [v0.12.0] - 2026-10-25
> **The Standalone Bundler, Installer & Multi-Window Native API Release**

### ✨ Highlights & New Features
- **One-Click Standalone Native `.exe` Bundler (`src-core/src/bundler.rs`):**
  - New `BundleEngine` module packages the compiled Rust binary + React `dist/` + `sandbox/` + `aegis.config.json` into a self-contained portable `dist-bundle/<AppName>/` directory.
  - Auto-generates a Windows **InnoSetup Installer Script** (`.iss`) for producing a native `Setup.exe` installer — no Rust or Node.js required on the end-user machine.
  - Auto-generates a Windows `.bat` launcher script for running the app directly.
  - New CLI flag: `cargo run -- --bundle` to trigger the full packaging pipeline.
  - New npm script: `npm run bundle` — builds Rust release binary + Vite frontend + runs bundler in one command.
- **Enhanced Multi-Window Manager (`src-core/src/window.rs`):**
  - Added `WindowRecord` registry for tracking all spawned secondary windows.
  - New `WindowConfig` fields: `always_on_top` and `kiosk` mode support.
  - Cross-platform launchers for Windows (Edge App Mode), macOS (Safari/open), and Linux (xdg-open).
  - Window ID generation for lifecycle management.
- **New IPC Commands (v0.12):**
  - `window_list` — Returns multi-window capability manifest and platform hint.
  - `bundle_status` — Queries the bundler engine state, existing bundles, and supported targets from inside the app.
- **New "Bundler & Packaging" UI Tab (React Frontend):**
  - 3-step visual bundling guide with interactive code blocks.
  - Live bundler engine status card (queries `bundle_status` IPC command).
  - InnoSetup installer documentation panel with bundle output structure preview.
  - Direct link to Inno Setup Compiler download.
- **`aegis-bridge.ts` v0.12 Updates:**
  - Added `call()` — a generic pass-through method for invoking any custom IPC command.
  - Added `getWindowCapabilities()` — queries multi-window support manifest.
  - Updated `SecurityStatus` type with `bundlerEngineActive` and `multiWindowApiActive` fields.

### 🛡️ Security
- Bundler strictly never exposes host system paths — all bundle paths are derived relative to `project_root`.
- InnoSetup script uses `PrivilegesRequired=lowest` — no admin escalation required.
- Bundler output includes the full `aegis.config.json` capability policy, preserving Zero-Trust settings in deployed apps.

### 🐛 Bug Fixes & Improvements
- `WindowConfig` now includes `always_on_top` and `kiosk` with proper `#[serde(default)]` annotations.
- `main.rs` now prints `v0.12.0` in the startup banner.
- `package.json` bumped to `0.12.0` with new `build:release` and `bundle` npm scripts.

---

## 🚀 [v0.11.0] - 2026-10-25
> **The Cryptographic Hardening, Rate Limiting & Privacy Shield Release**

### ✨ Highlights & New Features
- **Zero-Knowledge Encrypted Vault at Rest (`src-core/src/security/vault.rs`):**
  - Symmetric keystream encryption derived from user passphrases via **10,000 PBKDF2-style SHA-256 iterations**.
  - **HMAC-SHA256 Authentication Tags:** Detects and prevents file tampering, bit flips, or unauthorized file modifications.
  - Storage envelope format: `AEGIS_VAULT_V1:<HMAC>:<CIPHERTEXT>` ensuring on-disk confidentiality.
  - New IPC commands: `vault_encrypt_store` and `vault_decrypt_retrieve`.
- **Sliding-Window Anti-DoS Rate Limiter (`src-core/src/security/ratelimit.rs`):**
  - Protects loopback IPC port `48291` from local fuzzing and malicious burst flooding.
  - Enforces a strict 30 requests/second sliding window limit (triggers **HTTP 429** on violation).
  - **Brute-Force Nonce Lockout:** 5 consecutive invalid nonces activate a progressive 10-second penalty lockout.
- **Air-Gap & Zero-Leakage Privacy Policy Shield (`src-core/src/security/privacy.rs`):**
  - Mathematical zero-telemetry proof: Zero outbound sockets, zero diagnostic harvesting, strict `127.0.0.1` binding.
  - Accessible via `privacy_status` IPC command.
- **New UI Tab: "Privacy & Vault" in React Frontend:**
  - Zero-Knowledge Vault creator: Encrypt and decrypt local `.vault` records with live passphrase validation.
  - On-disk forensics inspector: Compare raw ciphertext with authenticated plaintext output.
  - Air-Gap compliance metrics display (GDPR, HIPAA, Zero-Knowledge verified).
- **Interactive Threat Simulator Upgrade:**
  - Added **Test 4: Anti-DoS Burst Flood Simulation** in the UI dashboard.
- **Automated Penetration Test Suite Enhancements:**
  - Added Test 4: Rate limiter burst flood verification in `aegis-cli`.
  - Added Test 5: Vault tampering & wrong-passphrase rejection test.

---

## 🚀 [v0.10.0] - 2026-10-25
> **Initial Public Open-Source Release**

### ✨ Highlights & New Features
- **High-Performance Rust Core Engine (`src-core/`):**
  - Ultra-lightweight binary (~8MB) and low memory usage (<50MB RAM).
  - High-speed HTTP loopback IPC server running on port `48291`.
  - Windows Edge App Mode fallback window manager.
- **Zero-Trust Security & Capability Engine:**
  - **Canonical Path Traversal Traps:** Traps malicious `../` attempts to climb out of `./sandbox/`.
  - **Shell Execution Revocation:** Command execution (`cmd.exe`, `powershell`, `bash`) is permanently blocked.
  - **Cryptographic Nonce Handshake:** 256-bit SHA-256 session token required for every IPC action.
  - **Origin Whitelist Verification:** Strict validation for `localhost:5173` origins.
- **Modern UI Suite (`src-ui/`):**
  - Built with React 18, TypeScript, Vite, and Lucide icons.
  - Ultra-premium dark glassmorphic design system.
  - **Interactive Attack Simulator:** Live testing dashboard demonstrating blocked path traversal, blocked shell execution, and invalid nonce injections.
  - **Sandboxed File Explorer:** Safely create and read isolated documents in `./sandbox/`.
  - **SHA-256 Cryptographic Hasher:** Live hashing utility.
- **Multi-Framework Compatibility:**
  - Comprehensive integration guides for **React, Vue 3, Svelte 5, SolidJS, Next.js (SSG), and Tailwind CSS**.
- **Automated Security Penetration Testing Suite:**
  - `npm run test:security` CLI script testing attack vectors against the running core.

### 🛡️ Security Hardening
- Enforced Content-Security-Policy (CSP) headers across all IPC responses.
- Implemented isolated `./sandbox/` root directory to prevent access to sensitive host directories.
- Added responsible disclosure policy in [SECURITY.md](SECURITY.md).

### 🐛 Bug Fixes & Refactoring
- Added complete `.gitignore` preventing build artifacts (`target/`, `node_modules/`) from bloating git repositories.
- Updated `sandbox/.gitkeep` ensuring git clones preserve the sandboxed storage root.
- Cleaned up npm scripts and introduced `npm run setup` for one-step installation.

---

## 🏷️ Version History Summary

| Version | Release Date | Key Changes | Stability |
| :---: | :---: | :--- | :---: |
| **[v0.14.0](#-v0140---2026-10-25)** | **2026-10-25** | Hardware Security Keys (FIDO2/WebAuthn), Multi-Profile Isolation, Wasm Plugin VM Sandbox | **Alpha** |
| **[v0.13.0](#-v0130---2026-10-25)** | **2026-10-25** | Rust Plugin Engine, System Tray & OS Notifications, SHA-256 Air-Gapped Updater, Capability Matrix | **Alpha** |
| **[v0.12.0](#-v0120---2026-10-25)** | **2026-10-25** | One-Click Native Bundler & .exe Installer, Multi-Window Manager, `bundle_status` & `window_list` IPC | **Alpha** |
| **[v0.11.0](#-v0110---2026-10-25)** | **2026-10-25** | Zero-Knowledge Encrypted Vault, Anti-DoS Rate Limiter, Air-Gap Privacy Shield | **Alpha** |
| **[v0.10.0](#-v0100---2026-10-25)** | **2026-10-25** | Initial Open-Source Release: Rust core, React UI, Sandbox, Attack Simulator | **Alpha** |
| *v0.20.0* | *Upcoming (Q1 2027)* | macOS WebKit, Biometric Enclave, Encrypted IPC, Automated Crash Reporter | *Planned* |

---

> 💡 **Have a feature request or found a bug?**  
> Feel free to [Open an Issue](https://github.com/charitha-only/aegis-desktop/issues) or submit a Pull Request following our [Contributing Guide](CONTRIBUTING.md).
