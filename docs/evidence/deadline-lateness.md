# Bounded deadline-lateness measurements

Date: 2026-10-24. Source: `715b562276a1d8c8c89e3c13b4b35f096275c0f3`. Release binary SHA-256: `b3ff4687624517a8ec0240872e7786a6c88c1ab36bf9c7e26f730240c67f36cc`.

The production engine records lateness immediately before the input call, relative to its monotonic scheduled deadline. Every attempt contributes, including a failed input attempt. Skipped slots remain a separate counter. This measures the injection-call boundary, not target-application delivery or interarrival error.

The per-run histogram has 1002 fixed counters: exact zero, 1000 upper-bound buckets at 100 us resolution through 100 ms, and an overflow bucket that reports the observed maximum. Nearest-rank p50/p95/p99 are conservative estimates, clamped to the observed maximum. Quantiles are calculated only at progress publication (at most twice per second) and terminal publication. The histogram stays on the worker; snapshots carry only three additional durations. No per-click allocation or log entry is added. Keeping the histogram outside the adapter panic boundary preserves terminal measurements after a backend panic.

## No-input baseline

```sh
cargo run --release --locked -p nanoclick-platform --example deadline_lateness -- 60
```

The example uses the platform-selected production waiter, a 10 ms plan and a no-op input sink. It opens no portal, sends no input and has no GUI. Its observer sleeps during the measurement and waits for terminal acknowledgement afterward. This host is the shared Fedora/Ryzen environment in `environment.md`, without controlled load isolation. This is an exploratory baseline, not a pass for the unloaded/loaded native-input acceptance gate.

The 60-second local run produced 6000 attempts/sent operations and zero skipped slots: p50 upper bound 100 us, p95 500 us, p99 600 us, maximum 868.669 us. Raw output: `deadline-no-input.csv`.

## Verification and remaining work

Three histogram unit tests first failed with the no-op implementation, then passed with the fixed buckets. They cover nearest-rank outlier exclusion, exact bucket boundaries, sub-microsecond rounding, empty input and extreme duration overflow. Engine tests also check ordered terminal quantiles and retention after a backend panic. All 42 local Rust tests and Linux/Windows-target Clippy checks pass. This does not establish native Windows timing.

The required 60-second unloaded and documented-load measurements with native input, actual UI and captured delivery remain open. CPU/RSS budgets are unchanged and remain unmet. Stop latency must continue to be measured separately.
