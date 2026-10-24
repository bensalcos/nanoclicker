# Initial performance measurements

## Core cancellation (2026-10-24)

Command: `cargo run --release --locked -p nanoclick-core --example cancellation_latency -j 2`.

Environment: Fedora 44, Ryzen 5 5625U, Rust 1.97.1; see environment.md. Standard release build without custom LTO or target-cpu=native. Other applications were active; this first exploratory measurement did not control load/power. The environment's linker warned about deprecated optimization setting '1'; build and execution succeeded without suppressing it.

4000 Stops total: 1000 per period (1, 10, 100 and 999999 ms), with deterministic 0–1999 µs variation between the first pair and Stop. The observer waits on notifications without busy waiting. Measurement starts before requesting Stop and ends on observing the terminal snapshot, including observer overhead. Backend **without desktop input**.

[CSV data](core-stop-latency.csv): p99 255–296 µs; observed maximum 1281 µs. This demonstrates that core cancellation does not wait for a long interval to expire. It does not establish keyboard, portal, SendInput, XTEST or actual platform button-release latency.

The product budget of p99 ≤20 ms and maximum ≤100 ms requires acceptance **per native backend**. At this initial measurement, startup, CPU, memory, injection jitter, compressed size and C++ comparison were pending; later evidence is indexed in STATUS.md. Do not present a no-input sink's results as complete NanoClick performance.
