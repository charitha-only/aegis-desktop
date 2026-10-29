# Performance measurements

Run against an optimized core binary:

```text
cargo build --release --locked --manifest-path src-core/Cargo.toml
node --test aegis-cli/test/metrics.test.js
node aegis-cli/bin/benchmark.js
```

The harness starts only its own core processes in temporary projects with dynamically allocated ports. It records five process-start trials, twenty real `sys_info` IPC round trips, binary size, idle resident memory and a coarse CPU-time sample. The environment is included in JSON. Percentiles use the nearest-rank definition; with five startup samples, p95 is the maximum, not a statistically stable tail estimate.

Windows samples the owned process with Get-Process. Linux/macOS use ps. CPU percentages refer to one logical core and use OS accounting resolution; a reported zero does not prove zero CPU consumption. Linux/macOS peak resident memory is unavailable in this harness. The sampling helper's overhead is not included in core CPU accounting.

This measures the core process without a UI/webview and without controlled filesystem caches. It does not establish cold-boot startup, full desktop application memory, rendering latency, battery impact or performance on constrained devices. Hosted CI measurements are evidence to inspect, not hard performance budgets.

Before final v0.27: choose physical baseline machines for all three OSes, define reproducible foreground/background/UI workloads, collect enough repetitions, set numeric regression budgets and measure complete process trees. Preserve KDF security parameters unless a separately justified versioned security change is approved.


## Physical Windows exploratory run

The [v0.29.8 local device record](evidence/windows-physical-v0.29.8/README.md) includes raw samples, exact binary/source provenance and isolated SDK/IPC acceptance from a Windows 10 Ryzen 5/16 GiB desktop. It is core-only exploratory evidence, not completion of whole-app or constrained-device performance acceptance.
