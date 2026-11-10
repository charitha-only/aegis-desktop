# Low-Resource Performance Baseline & Memory Profiling

This document establishes the verified resource consumption budgets and physical telemetry baseline for the Aegis Desktop Framework on constrained workstations, laptops, and low-resource devices (4 GB RAM or lower).

## 1. Measured Performance Telemetry (v0.30.7)

Physical measurements captured from the optimized release binary (`aegis-core.exe`, 8.45 MB) under real Windows host environments:

| Metric | Target Budget (4 GB Device) | Measured Result | Status |
| :--- | :--- | :--- | :---: |
| **Cold Startup Latency** | $< 100\text{ ms}$ | **26.31 ms** (Median), **33.25 ms** (P95) | ✅ **PASS** (3x faster than budget) |
| **IPC Roundtrip Latency** | $< 10\text{ ms}$ | **1.69 ms** (Median), **6.15 ms** (P95) | ✅ **PASS** |
| **Idle Resident Memory (RSS)** | $< 50\text{ MiB}$ | **8.80 MiB** (9,232,384 bytes) | ✅ **PASS** (5x lower than budget) |
| **Peak Resident Set (WorkingSet64)**| $< 64\text{ MiB}$ | **8.81 MiB** (9,236,480 bytes) | ✅ **PASS** |
| **Background Idle CPU Usage** | $< 1.0\%$ of 1 core | **0.0%** (Quiescent) | ✅ **PASS** |

> [!NOTE]
> Detailed JSON measurement evidence is recorded at `docs/evidence/performance-benchmark-v0.30.7.json`.

---

## 2. Resource Footprint vs Electron Comparison

Traditional Electron applications allocate a separate Chromium helper process per renderer, leading to substantial baseline memory consumption:

| Architecture Metric | Standard Electron 30 | Aegis Desktop Runtime | Efficiency Gain |
| :--- | :--- | :--- | :---: |
| **Binary Package Size** | ~85 MB – 140 MB | **8.45 MB** | **~10x–15x Smaller** |
| **Idle Resident Memory** | ~120 MB – 180 MB | **8.80 MB** | **~15x–20x Less RAM** |
| **Process Count** | 4 – 7 processes | **1 main process** (+ optional webview) | Single Process Model |
| **Cold Startup** | 400 ms – 1,200 ms | **26.3 ms** | **~15x Faster** |

---

## 3. Automated Performance Budget Verification

Developers and CI pipelines can continuously enforce memory and latency budgets using:

```bash
# Run physical performance benchmark
node aegis-cli/bin/benchmark.js

# Or enforce explicit thresholds via npm
npm run test:benchmark
```

Aegis fails closed with an error code if resident memory exceeds the allocated threshold ($50\text{ MiB}$) or IPC $p_{95}$ exceeds $10\text{ ms}$.

