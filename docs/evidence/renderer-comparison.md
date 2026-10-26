# Renderer comparison: isolated software rendering

Date: 2026-10-24. Both executables were built from application source `0435c9e`, Rust 1.97.1, standard release profile with line tables, no custom LTO or native CPU instructions. Same 520×600 logical window, dark theme and 2 Hz progress publication. Xvfb and host details are in [environment.md](environment.md).

Glow uses Mesa llvmpipe OpenGL (`LIBGL_ALWAYS_SOFTWARE=1`). Wgpu uses software Vulkan, explicitly selecting `WGPU_BACKEND=vulkan` and `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`. This prevents wgpu from silently using a physical GPU while glow is software-rendered. Both actual executables passed the isolated GUI flow, including manual countdown, global shortcut, complete pairs, no input after Stop and successful exit.

| Measurement | Glow | Wgpu | Initial budget |
|---|---:|---:|---:|
| Warm startup p95, 30 starts | 84.481 ms | 83.305 ms | ≤500 ms |
| Idle CPU, one core, 60 s | 0.0000% | 0.0167% | ≤1% |
| Idle RSS after stabilization | 107.230 MiB | 104.531 MiB | ≤100 MiB: **both unmet** |
| Active CPU at 100 pairs/s, 60 s | 5.2331% | 6.0329% | ≤5%: **both unmet** |
| Active RSS | 107.691 MiB | 104.883 MiB | Recorded separately |
| Complete received pairs | 6001 | 6001 | 60 s observation |
| Interarrival absolute error p99 | 1 ms | 1 ms | Not deadline lateness |
| Interarrival absolute error maximum | 2 ms | 1 ms | Observed only |

Data: [glow](ui-glow-wait-port.csv), [wgpu](ui-wgpu-software.csv). Reproduce with `scripts/measure-ui-x11.sh PATH_TO_RELEASE_BINARY` and the environment above. The helper records aggregate process CPU ticks, not host-wide utilization. Zero idle CPU means no additional tick was observed in that window, not literally zero execution cost. Each renderer has one complete run; do not infer statistical significance from a 1 ms startup difference. No compiler or other heavy verification process was run during these measurement windows.

Unstripped input executable SHA256:

- Glow: `23949b40fbbd21e6e8509b0b664fd488de86d064654b36924e2ee982d9592776`
- Wgpu: `1520b41c648c109e33847bc9d4ceb219f60d6ab3ed39a3ee29bb47d310ee80be`

Decision at the time: keep Glow as the provisional Linux default for this software-rendered case. The newer [current-source physical-GPU comparison](gpu-renderers.md#current-source-1-hz-gpu-follow-up--2026-10-26) changes the operational Linux default to WGPU provisionally; it does not close software-rendering budgets or native compatibility/accessibility acceptance.

A later [matched-source hardware-display comparison](gpu-renderers.md) measures both renderers with the expanded GUI fixture. Those results do not replace this software-rendering case or close its budgets.

## Current-source 1 Hz follow-up — 2026-10-25

To check whether the progress-refresh change alters the renderer trade-off, both renderer builds used the same current source with a 1,000 ms publication interval and the same private Xvfb/Mesa llvmpipe protocol. Each full run used 30 warm starts, 60 s stabilization, 60 s idle, 60 s active at 100 requested pairs/s and exact X11 event capture. WGPU explicitly selected lavapipe. Candidate hashes: Glow `39b8aab3673ea4deb6523d3025e686d824d5e9750e5ba890064720cb8f926d84`; WGPU `e7b5af41a028a1b36b74afcae5e0d390ed5aa58ed408b8c0fbfac8466deebff5`.

| Measurement | Glow run 1 | Glow run 2 | WGPU run 1 | WGPU run 2 |
|---|---:|---:|---:|---:|
| Startup p95, ms | 74.595 | 74.424 | 79.798 | 84.992 |
| Idle CPU, one core | 0.0000% | 0.0000% | 0.0167% | 0.0000% |
| Idle RSS, MiB | 107.309 | 107.059 | 106.141 | 105.945 |
| Active CPU, one core | 2.8165% | 2.9833% | 3.2998% | 3.4998% |
| Active RSS, MiB | 107.359 | 107.102 | 106.129 | 106.121 |
| Captured pairs | 6001 | 6001 | 6001 | 6001 |
| Interarrival absolute error p99/max, ms | 0/1 | 0/1 | 0/2 | 1/2 |

Across these two samples per renderer, WGPU used about 1.14 MiB less idle RSS but about 0.50 percentage points more active CPU and had slightly slower startup p95. Both software-rendered variants remain above the 100 MiB RSS target. This remains the software-rendering trade-off; repeated current-source hardware runs now favor WGPU for the Linux default based on startup and memory, with Glow retained as an explicit alternative. Native desktop acceptance remains open. Full progress-cadence data is in [UI polish performance](ui-polish-performance.md).
