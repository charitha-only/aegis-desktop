# v0.27.0 performance tooling preview

This source version starts the measurement milestone. It does not establish
low-resource-device suitability or a whole-application performance budget.

## Reproducible core reports

After building an optimized core, run from the project root:

```text
cargo build --release --locked --manifest-path src-core/Cargo.toml
node aegis-cli/bin/benchmark.js > core-benchmark.json
node aegis-cli/bin/benchmark.js /absolute/path/to/aegis-core
```

Schema 2 retains five startup durations and twenty sequential IPC durations in
`rawSamples`, nearest-rank summaries, a two-second idle interval, core RSS/CPU
when available, binary size, host OS/architecture/CPU/RAM, timestamp, source Git
revision, tracked working-tree state and binary SHA-256. The binary hash identifies the
measured artifact; a Git revision alone does not prove that a binary was built
from that revision. A dirty tracked tree or externally supplied binary must be disclosed. Untracked
files are excluded from the tracked-dirty flag so output artifacts do not mark
the measured source as modified; that flag is not a complete source attestation.
Schema-1 consumers must migrate; the runtime IPC format is unchanged.

The fixture launches only an owned headless core and stores temporary data.
There is no browser, UI rendering or native window. Startup timing includes
process launch until the listener log; it is not rendered UI readiness. Host
filesystem caches and scheduling are uncontrolled. Five/twenty samples are an
exploratory sample, not a statistical confidence claim. CPU is coarse, expressed
as percent of one logical core, and excludes children; RSS accounting differs
across operating systems. OS failures produce missing metrics, never zero-cost
claims. Compiler/webview/installer costs are not included.

## Explicit budgets

```text
node aegis-cli/bin/benchmark.js --budget=your-core-budget.json > measured.json
```

A budget is a nonempty JSON object with one or more keys:
`startupP95Ms`, `ipcP95Ms`, `rssBytes`, `idleCpuPercent`. Values must be finite
nonnegative numbers; unknown keys/types are rejected before launching fixtures.
The budget file is limited to 64 KiB. Missing/nonfinite requested measurements
fail rather than passing. The report includes each observed value, limit and
verdict. An exceeded budget writes the report and exits 1; a measured pass exits
0. Without a budget, `budget` is null: measurement alone is not acceptance.
There are no invented universal thresholds. Select and document limits from
repeated target-device measurements and product requirements; do not weaken
vault cryptography to meet a speed target.

Tests exercise the public evaluator and actual CLI. The CLI fixture sets a
zero startup allowance deliberately to demonstrate a nonzero failing exit with
retained raw evidence. It is not a production threshold. Three-OS portable CI
runs the fixture and uploads unbudgeted schema-2 reports as benchmark artifacts.
Hosted results are separate from physical-device acceptance.

## Physical device procedure

1. Record device identity, CPU, RAM, storage, OS/version, architecture, browser/
   webview/runtime versions, power mode and package/source/binary identities.
   Use actual available Windows, Linux and macOS machines; a VM/hosted runner
   does not establish physical acceptance on those OSes.
2. Define the same representative app/workload and data size. Label first-launch,
   warm-launch and cache conditions honestly. Repeat measurements and retain raw
   results and failures, not only a favorable summary.
3. Install the actual package and measure rendered readiness, interactions,
   responsiveness and background activity. Record the entire application process
   tree including webview/browser helpers. Distinguish RSS, shared memory and
   private/PSS-style measurements; simple RSS sums can double-count shared pages.
   The current core tool does not collect these whole-app/UI measurements.
4. Exercise file/native services, keyboard/screen readers, scaling/displays,
   install/upgrade/uninstall/data retention and controlled restart/recovery.
   Perform lock/suspend/resume in a saved, separate test session and record actual
   observed revocation/re-authentication, not simulated-event results as real power
   acceptance. Signing/notarization claims require real publisher credentials.
5. Store one record per source/package/OS/device with environment, workload,
   commands, raw artifacts, acceptance limits, pass/fail/blocked and reviewer.
   Missing devices stay blocked/unperformed. Low-resource acceptance needs an
   actual representative low-resource device; ordinary developer PCs are not
   substitutes. Avoid recording real passwords or vault content in artifacts.

Full UI/process-tree automation, physical workloads, baseline-derived budgets
and release regression policy remain in [RELEASE_GATES.md](RELEASE_GATES.md).
The [release ledger](IMPLEMENTATION_STATUS.md) records completed exact-source CI.
