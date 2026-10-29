# Physical Windows exploratory evidence — v0.29.8

Partial device evidence, not completion of physical desktop acceptance.

- Measured source checkout: `cdfa231b8677e0e9302b56db0f5b3a68c1b5c1ea` (documentation follow-up to source tag v0.29.8 / b3fb1ff).
- Core rebuilt with `cargo build --locked --release --manifest-path src-core/Cargo.toml` on the local Windows GNU toolchain. Build succeeded with existing compiler warnings; this was not a warning-free build.
- Windows 10 Pro x64, build 19045; AMD Ryzen 5 3500, six logical CPUs, approximately 16 GiB RAM. This desktop is not a constrained-device baseline. No hostname, serial number, account name or private key is included.
- See core-benchmark.json for exact binary SHA-256, timestamp, raw samples and measurement limitations. Five starts and twenty sys_info IPC round trips are exploratory, not statistical release budgets. The process is headless; no whole-application/UI/webview measurement was performed.
- Startup median: 28.01 ms; observed maximum/p95: 286.23 ms. The first measured start was slower; cache state was uncontrolled.
- IPC median: 2.19 ms; p95: 3.48 ms. Idle resident memory sample: 9,121,792 bytes (about 8.70 MiB). Coarse CPU accounting reported zero delta; this does not establish zero CPU consumption.
- SDK smoke exercises the real release core and TypeScript client under temporary application-data roots: Unicode file roundtrip, byte receipt, sorted listing, traversal denial, vault roundtrip and lock/lifecycle-dependent authorization. See sdk-smoke.txt.
- IPC deadline smoke uses owned temporary processes and loopback endpoints. See ipc-deadline-smoke.txt for incomplete/trickling requests, bounds, admission and recovery results.

## Reproduce safely

Run from the exact checkout, after reviewing the fixture scripts:

```text
cargo build --locked --release --manifest-path src-core/Cargo.toml
node aegis-cli/bin/benchmark.js
node aegis-cli/bin/sdk-smoke.js
node aegis-cli/bin/ipc-deadline-smoke.js
```

These fixtures stop their own child processes. No workstation lock/suspend, clipboard interaction, installer replacement, power interruption or real credential vault test was performed. Do not interpret a passing API lifecycle check as a real lock/sleep/resume event.

## Open acceptance

- Windows interactive embedded UI, navigation, keyboard/screen-reader access, scaling/multiple displays, native services, whole-process/UI performance and real lock/sleep/resume on a dedicated test device.
- Physical Linux/macOS acceptance and a selected low-resource device class.
- Crash/power-loss recovery on representative storage using disposable data.
- Owner-controlled publisher signing and macOS notarization. CurrentUser/My and LocalMachine/My code-signing certificate queries returned no matching certificates on this machine; no signing identity was created/imported and no publisher-signed artifact was produced. The existing manual signed-Windows workflow remains gated on securely provisioned publisher credentials.
- A qualified separate reviewer, report and retest. Project-authored tests and this evidence are not an independent audit.
