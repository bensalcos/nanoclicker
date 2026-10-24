# Deadline lateness from the actual GUI application

Date: 2026-10-24. Application source `2b4ea47ba47c381f3e0f670039f752b551a45ccc`, default glow release; SHA-256 `0f6a7049b40936492bd87dcacdd3c13a32f2c8fbe053cc431e4a9dec52e58e51`. The loaded observer is source `ef4bc51d48f464a2057a136afd7a84000e13c698`; that change leaves the application unchanged.

The optional diagnostics exporter now makes the engine's terminal histogram available from the actual GUI. Each run starts through its F6 shortcut and stops through the same UI/controller path. The observer waits for Ready, then requires exactly one terminal JSONL record whose submitted count matches captured down/up counts, with zero failures and ordered quantiles. Initial idle and active progress produce no records; the no-added-load file was observed empty during its active phase.

Environment: Fedora 44, Ryzen 5 5625U (6 cores/12 threads), private headless Weston 15 GL plus rootful Xwayland 24.1.13, AMD Renoir/Mesa 26.2.2 GPU-backed display, scale 1, dark theme, 10 ms requested period and private activation-disabled D-Bus. The GPU setup and isolation are described in [gpu-display-setup.md](gpu-display-setup.md). No physical desktop input or portal consent was used. No heavy local builds ran during the measured phases.

Each full run includes the expanded GUI preflight, thirty startup samples, 60 seconds stabilization, 60 seconds idle measurement and 60 seconds active input. The loaded case starts two observer-owned CPU worker threads immediately before active Start, using the same black-box wrapping-integer recurrence as the native X11 fixture. Startup and idle phases precede this deliberate load. RAII stops/joins the workers on normal completion, errors and unwinding. The configured load is logged; an independent per-thread utilization trace was not captured.

| Measurement | No added load | Two load threads |
|---|---:|---:|
| Captured down/up counts and engine sent | 6001 each | 6001 each |
| Failures / skipped periods | 0 / 0 | 0 / 0 |
| Engine p50 upper bound | 100 µs | 100 µs |
| Engine p95 upper bound | 200 µs | 100 µs |
| Engine p99 upper bound | 400 µs | 100 µs |
| Engine observed maximum | 749.458 µs | 1300.360 µs |
| Server interarrival error p99 / max | 1 / 1 ms | 1 / 2 ms |
| Startup p95 | 212.534 ms | 243.490 ms |
| Idle RSS | 86.984 MiB | 87.086 MiB |
| Active CPU, one application core | 1.3666% | 1.1666% |

Both engine p99 upper bounds are below the unchanged 2 ms target in these two GUI fixture cases. Percentiles are conservative nearest-rank histogram estimates with 100 µs buckets through 100 ms; overflow uses the observed maximum. They measure pre-injection deadline lateness. Server timestamps have millisecond resolution and measure a different point; target processing latency is not established.

These are single runs on a shared host with uncontrolled background applications/power. Lower loaded p99/CPU does not show that load improves performance. Differences from earlier renderer runs are not attributed to this exporter. Strict pair ordering is covered by the separate GUI/adapter preflight; the active performance observer checks matching down/up counts. Native Windows/Wayland timing, software-rendering budgets, actual monitor/reader behavior and broader T16 acceptance remain open.

Data: [no-added-load CSV](gui-deadlines-no-added-load.csv), [terminal JSONL](gui-deadlines-no-added-load.jsonl), [GUI preflight](gui-deadlines-no-added-load-validation.txt); [loaded CSV](gui-deadlines-two-load-threads.csv), [terminal JSONL](gui-deadlines-two-load-threads.jsonl), [GUI preflight](gui-deadlines-two-load-threads-validation.txt); [source, hashes and environment](gui-deadlines-environment.json). The unloaded observer's binary hash was not captured before rebuilding it for the loaded case; its source commit and complete output are retained.

Reproduce with a verified Weston prefix and new output paths:

```sh
cargo build --release --locked -p nanoclick
NANOCLICK_DIAGNOSTICS_PATH=/path/to/new-no-load.jsonl \
  scripts/measure-ui-gpu.sh WESTON_PREFIX target/release/nanoclick
NANOCLICK_BENCH_LOAD_THREADS=2 NANOCLICK_DIAGNOSTICS_PATH=/path/to/new-loaded.jsonl \
  scripts/measure-ui-gpu.sh WESTON_PREFIX target/release/nanoclick
```

The local release and observer also reject existing output paths and invalid load counts without connecting to a display. Local Clippy, Windows MSVC cross-target Clippy, wgpu Clippy and Rust 1.95 checks pass. Forty-seven Linux Rust tests pass; hosted run 40139252678 also passes all four jobs with 47 Linux and 50 native Windows tests. Downloaded Ubuntu GUI/portal logs confirm twenty measured stress cycles, Save/relaunch and all thirteen portal scenarios. See [CI evidence](ci.md).
