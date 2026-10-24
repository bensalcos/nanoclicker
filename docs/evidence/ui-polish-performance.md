# Release check after accessibility/layout changes

Date: 2026-10-24. Application source: `fb36ed5`, default glow renderer, standard release profile. Input executable SHA256: `39cb222a455908f2979c8084dc71cfcaf6142d5a9d16d57b93ea51cb70fcb965`.

Command: `LIBGL_ALWAYS_SOFTWARE=1 scripts/measure-ui-x11.sh target/release/nanoclick`. Same host, private Xvfb/Mesa llvmpipe setup and 2 Hz progress publication as the earlier glow runs. No compiler or heavy verification process ran during measurement. The helper now derives the primary-action position from window geometry and explicitly fixes X11 scaling to 100%; startup observation still polls every 5 ms.

| Measurement | Result | Initial target |
|---|---:|---:|
| Warm startup p95, 30 starts | 73.712 ms | ≤500 ms |
| Idle CPU, one core, 60 s | 0.0000% | ≤1% |
| Idle RSS | 106.789 MiB | ≤100 MiB: **unmet** |
| Active CPU at 100 pairs/s, 60 s | 5.3499% | ≤5%: **unmet** |
| Active RSS | 106.914 MiB | Recorded separately |
| Complete received pairs | 6001 | 60 s observation |
| Interarrival absolute error p99 | 1 ms | Not deadline lateness |
| Interarrival absolute error maximum | 1 ms | Observed only |

[Numeric record](ui-glow-accessibility.csv). Zero idle CPU means no additional process tick was observed in that window. CPU uses aggregate process ticks with one core as 100%; RSS includes software graphics libraries.

The pre-polish matched-source glow run observed 84.481 ms startup p95, 107.230 MiB idle RSS and 5.2331% active CPU. This new single run has lower startup/RSS and slightly higher active CPU. Neither is a repeated-trial statistical comparison; do not claim a proven CPU improvement or no regression. The active CPU and memory gates remain open, without changing thresholds or removing accessibility features to make the measurements pass.

Next performance work needs profiling and measured changes, followed by the same functional/keyboard checks. Native input Stop, scheduler deadline lateness, loaded operation, physical GPU behavior, Windows and Wayland remain separate acceptance requirements.
