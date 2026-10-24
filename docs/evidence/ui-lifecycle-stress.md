# Release UI lifecycle and resource stress

Date: 2026-10-24. Harness source and exact app SHA-256: [environment record](ui-lifecycle-environment.json). The release application's production sources match the harness checkout; the record distinguishes its build checkout from the later test-only commit. Aggregate checks: [log](ui-lifecycle-stress.txt). Resource samples: [CSV](ui-lifecycle-stress.csv).

```sh
cargo build --release --locked -p nanoclick
NANOCLICK_UI_STRESS_CYCLES=1000 scripts/verify-ui-x11.sh target/release/nanoclick
scripts/verify-ui-x11.sh target/release/nanoclick 2
```

The actual default-glow release executable runs in a private Xvfb, dark theme, 100% scale and the default **100 ms** interval. Each F6 Start originates outside the application's focus. The harness waits for captured input and the red active action, requests Stop, then requires the blue ready action with complete ordered pairs. Requiring the active color before Stop prevents an old blue frame from being mistaken for a new acknowledgement. A further 25 ms observation rejects post-Stop input. The initial standalone stop check retains its 250 ms observation.

After ten warm-up cycles, 1000 measured cycles completed. The target captured 1010 complete pairs in the stress phase, including warm-up. File descriptors, threads and RSS were sampled after warm-up and every 20 measured cycles. All 51 samples reported **109,928 KiB RSS (107.3515625 MiB), 21 descriptors and 31 threads**. The descriptor check rejects changes from the warm baseline; threads and RSS are recorded, not subjected to invented thresholds. No growth was observed at these sample points; this is finite evidence rather than proof that no leak can occur.

The observer reads only its child application's procfs resource counts. It verifies that `/proc/PID/exe` matches the requested direct executable, rejecting wrappers whose process metrics would describe a tracer or launcher. Directory-enumeration errors are propagated. Measurements do not include observer memory, and no per-click application logging is added.

## Active-close cases

Three additional application launches exercise graceful WM_DELETE_WINDOW handling:

- During the red manual countdown: clean exit with no target input.
- While running: clean exit with ordered complete pairs and no held pair.
- Immediately after a shortcut Stop request: clean exit and complete pairs. This is a Stop-request/close race, not proof that a rendered Stopping state was observed.

Each exit is followed by a 250 ms no-input check. These cases pass at 100% and 200% scale. The existing minimum-size Start and full three-second manual countdown also still pass. The earlier 100-cycle run likewise showed constant sampled RSS/descriptors/threads, and the two-cycle smoke run passed. Clippy passes for all X11-feature examples, including the performance harness that shares pixel-reading support.

## Scope and remaining gates

These are real release-UI, native-X11 lifecycle checks. They do not establish Windows/Wayland UI lifecycle behavior, backend-stall/disconnection recovery, uniform Stop latency, native monitor transitions or screen-reader acceptance. RSS is sampled between cycles rather than after the required 60-second idle stabilization. The formal memory/CPU budgets remain unaccepted; earlier full measurements exceeded them, and this test does not replace that protocol or lower its thresholds.

CI now requests twenty measured cycles against the already-built **release** application and runs the active-close cases. Previously that invocation built a debug application by default. The first hosted result for the new release/stress invocation is pending; the 1000-cycle evidence above is local.
