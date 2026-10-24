# Full glow measurement on a GPU-backed display

Date: 2026-10-24. Source, executable hashes and environment: [record](gpu-glow-environment.json). Aggregate measurements: [CSV](gpu-glow.csv). [UI preflight](gpu-display-validation.txt) and [GLX attestation](gpu-display-glxinfo.txt).

The default release application completed the full protocol through `scripts/measure-ui-gpu.sh`: thirty warm starts, sixty-second stabilization, sixty-second idle and sixty-second active input. The private headless Weston/rootful Xwayland display reports accelerated AMD Radeon rendering. The exact application's GL renderer string was not queried, so the attestation is explicitly scoped to the display. No application feature, font, accessibility code or budget was removed.

| Measurement | Observed | Initial target | Scope |
|---|---|---|---|
| Warm startup p95 | 124.316 ms | ≤500 ms | Thirty usable-window samples |
| Idle RSS | 85.707 MiB | ≤100 MiB | After sixty-second stabilization and idle phase |
| Idle CPU | 0 observed ticks / 60 s | ≤1% of one core | Sampling resolution; not a zero-cost guarantee |
| Active CPU | 2.0833% of one core | ≤5% of one core | Sixty seconds, requested 10 ms period |
| Active RSS | 87.094 MiB | No separate active-memory target | Informational |
| Captured input | 6001 matching down/up counts | Report actual throughput | About 100 pairs/s; first pair is immediate |
| Interarrival absolute error | p99 0 ms, maximum 2 ms | Not the deadline budget | Server timestamps have millisecond resolution |

The corresponding startup, CPU and idle-memory values are below the unchanged targets in this case. **Deadline lateness is not inferred from interarrival error**, and the GUI's internal deadline histogram was not exported by this harness. This result does not close the full performance or platform matrix.

A previous prototype of the same GPU display approach measured startup p95 327.084 ms, idle RSS 86.348 MiB, active CPU 2.5498%, active RSS 87.648 MiB and 5996 matching pairs. It lacked the final reusable launcher's geometry/validation protocol and remains exploratory. No causal improvement is inferred between the runs.

The renderer process is measured directly rather than a shell/tracer parent. Display-server/compositor CPU and memory are outside the application-process budget and are not included. The host is shared, uses its existing powersave governor and was not placed in a controlled benchmark power/load state. No active screen reader or activatable service is present on the private bus. Software-rendered Xvfb data remains separate and exceeds the budgets; hardware results do not grant it an exemption. Wgpu on the same GPU, Windows, native Xorg/GNOME/KDE and reader/monitor acceptance remain open.

The full UI preflight passes manual Start, minimum size, outside-focus F6, complete pairs, no post-Stop input and active-close cases. The GPU-only countdown fixture reasserted the emulated cursor once after its observed reset; it is not physical-pointer QA. The measured active phase has no cursor guard. See [setup and limitations](gpu-display-setup.md) for reproduction and the private-display isolation boundary.
