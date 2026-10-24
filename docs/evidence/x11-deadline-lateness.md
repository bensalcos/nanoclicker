# Native X11 deadline lateness

Date: 2026-10-24. Source: `65d509da93b8260b3545c648637a1388df61197c`. Release binary and host details: [environment record](x11-deadline-environment.json). Raw aggregate data: [CSV](x11-deadline-lateness.csv).

The existing isolated X11 acceptance harness now has an optional timing mode. It uses the production engine and checked XTEST adapter at a 10 ms period. A private Xvfb target captures all button events and checks left-button down/up alternation and the final pair count against the terminal engine snapshot. Capture uses constant-space counters. The engine's histogram supplies pre-injection deadline estimates; these are separate from target reception timestamps. No GUI is launched in this measurement.

```sh
scripts/measure-x11-deadlines.sh 60 0
scripts/measure-x11-deadlines.sh 60 2
```

The wrapper always starts a private Xvfb and removes inherited Wayland selection. The example refuses injection without its isolation guard. Arguments allow 1..3600 seconds and 0..2 load threads. Load threads continuously run an integer recurrence through `black_box`; an RAII guard signals and joins them on success and error. The capture loop polls at 1 ms. This observer/load activity is outside any measured application CPU budget.

| Added CPU load | Duration | Captured pairs / sent | Skipped | p50 upper | p95 upper | p99 upper | Maximum |
|---|---|---|---|---|---|---|---|
| None | 60.001931 s | 6001 / 6001 | 0 | 100 us | 200 us | 500 us | 2196.153 us |
| Two load threads | 60.001614 s | 6001 / 6001 | 0 | 100 us | 100 us | 100 us | 724.788 us |

Both observed p99 upper bounds are below the unchanged 2 ms target in this specific environment. The maximum in the first run exceeds 2 ms; the deadline budget is a p99 threshold, not a maximum guarantee. The two-thread result being lower does not establish a beneficial effect of load: this is a shared Fedora host with a powersave governor and no CPU affinity/isolation. No causal optimization claim is made. Background load was already present before the experiment.

One-second smoke runs with zero and two load threads also passed. The normal acceptance prelude verifies all three buttons, held-key suppression and shortcut conflicts before each timing phase. Histogram code and the native waiter have separate unit tests.

This closes a local native-X11 measurement gap, not the full timing gate. Reference Ubuntu/native GPU, the actual UI, Windows, Wayland, controlled load/power conditions and repeated acceptance measurements remain separate. CPU/RAM budgets remain unmet. This is also not the required 1000-stop native latency test or proof of delivery behavior in arbitrary applications.
