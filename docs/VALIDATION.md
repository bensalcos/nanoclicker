# Quality, performance and acceptance

All numbers below are **initial budgets requiring measurement**, not results or real-time guarantees. Never lower them silently: record the data, cause and decision if they need revision.

## Budgets

| Measurement | Initial target | Conditions |
|---|---|---|
| Warm startup to usable window | p95 ≤500 ms | 30 release launches, without permission dialogs |
| Idle memory | RSS/private working set ≤100 MiB | Stabilized for 60 s; record OS and renderer |
| Idle CPU | ≤1% of one core | 60 s average, without animations or interaction |
| Active CPU at 100 clicks/s | ≤5% of one core | 60 s average, entire process; record backend |
| Stop to engine acknowledgement | p99 ≤20 ms, observed maximum ≤100 ms | 1000 randomized stops at 1/10/100/999999 ms intervals |
| Deadline lateness at a 10 ms period | p99 ≤2 ms | 60 s unloaded and 60 s under documented load |
| Compressed portable binary | ≤25 MiB | Per platform; list required libraries separately |

Apply the same table to each backend. Wayland does not receive an exemption for using D-Bus: a missed budget requires optimization or an explicit capability/supported-range decision. The minimum configurable interval of 1 ms does not promise sustained 1000 clicks/s; show the requested period and measured result separately.

Reference machines: x86_64, at least 4 cores and 8 GiB; record exact CPU, RAM, GPU/driver, OS/kernel, desktop/compositor/portal, power settings, load and commit. For each measurement distinguish cold/warm, debug/release and engine/UI/target scope. Measure the entire process for CPU/RAM, rather than only a microbenchmark.

## Test strategy

1. **Deterministic core:** fake clock/waiter/input; validation, exact N, first click, skipped slots, overflow, repeated Start, Stop during preparation and execution, Shutdown and partial failures. Test count arithmetic without waiting for real time.
2. **Concurrency:** Start/Stop/Start and close races, lost wakeups, full channel, terminal snapshot and events from a previous run_id; bounded stress tests. Use Loom only when it adds coverage of the application's synchronization protocol.
3. **Adapters:** isolated test targets capture button, down/up order and count. Never inject into shared CI desktops. Xvfb provides isolated X11 evidence, not Wayland support or native graphical QA.
4. **Failures:** occupied shortcut, lost X server/D-Bus connection, missing portal, denied/revoked permission, failed button release and corrupt configuration. Running must not survive an invalid backend.
5. **UI:** English text and messages, count completion unlocks controls, cancelable countdown, DPI, keyboard and screen readers according to UI.md.
6. **Performance:** release scheduler benchmarks and end-to-end measurements; bounded histograms, p50/p95/p99/maximum and skipped slots. Distinguish injection-call timestamps from capture-reception timestamps; no test establishes behavior in every application.

Expected checks for each supported target:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
cargo bench --locked
```

Include a valid feature matrix (selected renderer and backend per target); use `--all-features` only when it does not combine mutually exclusive options. Maintain MSRV checks, dependency/license audits and diagnostic symbols. Run Miri for the core if dependencies permit; do not treat it as proof of OS FFI behavior.

## Delivery matrix

| Platform | CI build/tests | Required native tests |
|---|---|---|
| Windows 11 x64 MSVC | fmt, clippy, tests, release | SendInput, shortcut outside focus, DPI, NVDA, close |
| Ubuntu 24.04 x64 X11 | tests, release, isolated integration | XTEST, conflicts, layouts, Orca |
| Ubuntu GNOME Wayland | Linux build and contract tests | Portals, native Wayland client, permissions and revocation |
| KDE Plasma Wayland | Linux build and contract tests | Same flows; record distribution/versions during P0 |

Initial artifacts: Windows ZIP with executable and licenses; Linux tar.gz with binary, `.desktop`, icon and requirements. AppImage/MSI/Flatpak follow only after verifying need and compatibility; do not describe a Linux binary as dependency-free portable without checking linkage and its glibc baseline.

Release requires accepted parity, native matrix and budgets, a license inventory, checksums and reproducible build instructions. Review icon provenance and complete the MIT notice with confirmed details. Code signing and publication require explicit credentials/scope; prepare artifacts locally without making those credentials a prerequisite for implementation.
