# Actual GUI performance: Linux X11, glow

Measured 2026-10-24 on the machine in [environment.md](environment.md), using the release executable, a private Xvfb server and Mesa llvmpipe software rendering. This is isolated X11 evidence, not physical GPU, Windows or Wayland acceptance.

Reproduce with `scripts/measure-ui-x11.sh target/release/nanoclick`. The helper discards one warmup, measures 30 starts to a visible usable Start button, stabilizes for 60 seconds, then samples idle and active operation for 60 seconds each. It reads aggregate process CPU ticks and VmRSS from `/proc`; CPU percentages use one core as 100%. Active input is 100 complete left-button pairs per second into an isolated capture window. Startup observation polls every 5 ms.

| Measurement | 100 ms progress publication | 500 ms publication | Budget |
|---|---:|---:|---:|
| Warm startup p95 | 84.466 ms | 88.575 ms | ≤500 ms |
| Idle CPU, one core | 0.0167% | 0.0167% | ≤1% |
| Idle RSS | 103.457 MiB | 106.965 MiB | ≤100 MiB: **unmet** |
| Active CPU, one core | 19.6663% | 5.2330% | ≤5%: **unmet** |
| Active RSS | 103.559 MiB | 107.000 MiB | Recorded separately |
| Captured complete pairs | 6000 | 6001 | 60 s observation |
| Interarrival absolute error p99 | 1 ms | 1 ms | Not deadline lateness |
| Interarrival absolute error maximum | 3 ms | 1 ms | Observed sample only |

Raw numeric records: [baseline](ui-glow-baseline.csv), [500 ms candidate](ui-glow-2hz.csv). Both used the same release profile and default glow features. The 500 ms candidate changes `core/src/engine.rs` progress publication only; the first pair and all lifecycle transitions still publish immediately. A complete application test passed after the change: shortcut outside focus, input pairing, Stop acknowledgement, no input after Stop, manual countdown and close.

Thread profiling identified llvmpipe rasterization as the largest aggregate contributor in the baseline, rather than the click worker. Publishing continuous progress at 2 Hz substantially reduces that cost without changing the scheduler interval. Counter refresh remains bounded to 500 ms; start, stop and terminal outcomes are immediate. The original architecture allowed up to 10 Hz, so this is a lower chosen frequency within that limit.

The full 60-second result supersedes a short 10-second exploratory result of 4.40% active CPU. Do not claim the active CPU gate passed. RSS also remains above budget; no threshold was changed. There are no repeated-trial confidence intervals yet, and the shared host has other applications running.

X11 event timestamps have millisecond resolution. Interarrival error measures spacing between received events; it does **not** measure scheduler deadline lateness, Stop latency or input delivery to every application. Those gates require their own evidence. Wgpu runtime comparison, loaded operation and native platform measurements remain pending.
