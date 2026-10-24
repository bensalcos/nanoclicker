# Native X11 Stop latency

Date: 2026-10-24. Source: `c18c59b0316a9cd4181c37dec97da06d0e2ed320`. Release binary SHA-256: `164046964658ed02bca17166dcdb4a4b818a8141cd934b918768c335c271c3fb`. Raw results: [CSV](x11-stop-latency.csv). Host: shared Fedora 44, Ryzen 5 5625U (12 logical CPUs), powersave governor; private Xvfb, no application GUI and no added CPU load.

```sh
scripts/measure-x11-stop.sh
```

The production engine and XTEST adapter run 1000 start/stop cycles for each requested interval: 1, 10, 100 and 999999 ms. Each run waits for an actual captured pair, then requests Stop after a reproducible xorshift delay of 0..2000 us (fixed seed `0x6e616e6f636c6963`). Delays are not uniform across the entire long interval. The observer polls at 100 us and records elapsed time from the Stop request until the retained terminal Idle snapshot, including native cleanup and observer delay. It separately drains and validates ordered left-button pairs against the terminal sent count before starting another run. Per-sample timeouts are 3 s and are test safety limits, not replacements for the unchanged 20 ms p99 / 100 ms maximum budgets.

| Interval | Samples | p50 | p95 | p99 | Maximum | Captured pairs |
|---|---|---|---|---|---|---|
| 1 ms | 1000 | 167.975 us | 175.559 us | 188.063 us | 386.736 us | 1641 |
| 10 ms | 1000 | 162.585 us | 165.190 us | 174.438 us | 206.337 us | 1000 |
| 100 ms | 1000 | 162.575 us | 166.923 us | 188.243 us | 198.042 us | 1000 |
| 999999 ms | 1000 | 162.705 us | 167.444 us | 187.903 us | 294.723 us | 1000 |

All 4000 runs retained the correct terminal run identity, Stop reason and captured/sent pair count; no capture ended with a held button. Nearest-rank quantiles are computed from 1000 bounded samples per interval. The original three-button and hotkey acceptance prelude also passed. Clippy passes for the extended example.

Observed local values are below the unchanged Stop budgets. This does not establish UI response time, Stop via a desktop shortcut, uniform coverage of all timer phases, memory stability, disconnection behavior, native Windows/Wayland timing or arbitrary target-application behavior. The actual UI and other platform gates remain open. No global desktop input is sent: the wrapper creates its own display and the example requires the isolation guard.
