# Local terminal diagnostics

Diagnostics are disabled by default. To export aggregate measurements from terminal snapshots displayed by the GUI:

```sh
nanoclick --diagnostics /path/to/new-measurements.jsonl
```

The destination must be new and its parent directory must exist. Creation is exclusive: existing files, including existing symlinks, are not overwritten. Unix creation requests mode 0600 subject to the process umask; Windows uses the destination's normal inherited permissions. A creation error stops startup before opening the GUI. The path is never saved in preferences.

Each JSONL record has schema_version 1, app_version, renderer, backend (`windows`, `x11`, `wayland`), run_id, reason (`stopped`, `completed`, `failed`), attempts, sent, failures, skipped, lateness_p50_ns, lateness_p95_ns, lateness_p99_ns and max_lateness_ns. Nanosecond fields are unsigned integer durations. Percentiles are the core histogram's conservative nearest-rank upper bounds with 100 µs buckets through 100 ms; overflow uses the observed maximum. They measure pre-injection deadline lateness, not input-call completion or target receipt.

The exporter writes a terminal run once when the GUI observes it. It does not export initial idle state or active progress and is not a durable audit of every run: abrupt exit, closing while active or restarting before a terminal state is displayed can leave no record for that run. It omits raw errors, settings, permission data, input events and target information. Normal use performs no diagnostic file I/O or serialization.

Writes occur only at terminal GUI observation, outside the worker/click loop. A write/flush error disables further export and displays a message; the engine remains available. This optional terminal file write is synchronous and can delay that GUI frame on a slow destination. It is not part of engine Stop acknowledgement.

## Benchmark integration

Set `NANOCLICK_DIAGNOSTICS_PATH` to a new path when invoking `scripts/measure-ui-gpu.sh` or `scripts/measure-ui-x11.sh`. The performance fixture passes it only to the measured application, leaving warm-start samples and GUI preflight unchanged. After Stop/Ready it requires one terminal record, checks its count against captured input, requires zero failures and ordered percentiles, and prints the engine lateness values separately from server interarrival error.

The environment variable belongs to the test harness; the application itself only enables export through its CLI flag. Use full 60-second runs for timing evidence. A --quick-profile run verifies wiring but cannot accept the timing budget. Loaded/unloaded, backend, renderer and capture scope must remain explicit.

For a loaded GUI run, set `NANOCLICK_BENCH_LOAD_THREADS=2`. The isolated performance observer starts two bounded-lifetime CPU threads immediately before F6 Start and joins them on every exit path; startup and idle phases run without this added load. The workers use the same black-box wrapping integer recurrence as the native X11 timing fixture. Only 0..2 threads are accepted. CPU/RSS measurements still refer to the application process, not the observer/load process. This deliberate test load is not part of the production engine.
