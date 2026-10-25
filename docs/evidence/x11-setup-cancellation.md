# X11 connection setup cancellation

Date: 2026-10-24. Follow-up to [protocol deadlines](x11-stalled-server.md). This change affects cold connection setup, not per-click delivery.

## Reproduction and decision

A private Xvfb and a private `XAUTHORITY` FIFO with no writer reproduced another hang on the pre-fix source `2bf28b6`: `/proc` showed the input worker in `wait_for_partner` inside the authority-file open; requesting Stop produced no terminal state within two seconds. The driver opened only its own FIFO to let the old blocking destructor exit. No host authority file or desktop was used.

The upstream x11rb 0.13.2 implementation performs blocking DNS/socket establishment and `File::open`/buffered authority reads before protocol setup. Its public authentication helper does not take a cancellation token or deadline. Wrapping only protocol readiness cannot cover those operations. Reimplementing authentication or rejecting remote displays would change existing connection semantics.

The platform now runs that metadata preparation in a named, short-lived helper and waits with an absolute three-second deadline, checking cancellation at most every ten milliseconds while waiting. The helper does not perform the X11 handshake or input injection. Only a timely, uncancelled result transfers its owned socket/authentication data to the input or shortcut worker. An abandoned result is dropped, closing its socket. Checks between address attempts and before/after authentication prevent continuing preparation once its token/deadline is observed.

There are four process-wide setup slots, reserved atomically before spawning. A slot stays owned by its helper until that operation actually returns or unwinds; cancellation/timeout does not release it early. Saturation returns an explicit busy error without spawning or queuing more work. Normal helpers exit after one result; no helper polls while the application is idle. The existing event-driven shortcut idle wait and engine waiter's notification hook are unchanged.

This deliberately contains blocking work rather than claiming to forcibly interrupt arbitrary OS calls. An abandoned operation may retain a thread, socket and authentication buffers until the OS returns. All four slots can remain unavailable, requiring recovery of the OS resource or an application restart. Kernel tasks stuck in uninterruptible I/O can also delay process termination: this implementation does not prove a universal kernel/process-exit bound. A successful cold connection can spend up to three seconds in metadata setup and a separate three seconds in protocol setup. Per-pair protocol bounds remain as previously documented.

## Verification

[Recorded output](x11-setup-cancellation.txt) covers eight real private-resource cases:

- All five existing paused/disconnected-server protocol cases pass.
- Stop while the setup helper is observed in the FIFO open wait reaches Idle/Stopped with zero attempts in 10.584 ms.
- Without Stop, the same blocked FIFO produces Faulted/Failed with zero attempts and the setup-timeout error in about 3000.792 ms.
- Shortcut registration with the same FIFO returns an error and joins its listener in about 3000.434 ms.

The FIFO cases also require the fixture process to exit successfully while no FIFO writer exists. These are single debug observations, not release Stop quantiles or native desktop acceptance. The driver requires the actual named helper's FIFO wait; an unavailable observation fails the test.

Two deterministic unit tests separately verify cancellation before blocked work completes, destruction of its late result, retention of all four slots after timeout, rejection of a fifth operation and reuse after the original operations finish. All 51 workspace Rust tests (25 core, 19 application, 7 platform) pass, as do workspace feature Clippy, Windows MSVC cross-target Clippy, Rust 1.95 checks, alternate wgpu Clippy and the normal three-button/X11 shortcut regressions. The rebuilt release GUI also passes its isolated X11 functional flow, including outside-focus shortcut, minimum size, manual countdown, all three close cases and Save/relaunch. [GUI output](x11-setup-gui.txt). Executable SHA-256: `f5565bf1e5ef6f2cc9fc3a084bffacb64472693ff2f506d8ea620590de7ec413`. Hosted run [40146057324](https://github.com/bensalcos/nanoclicker/actions/runs/40146057324), source `9051e65a23a8f5bf8d40025f87b5667c3bb9de58`, passes all four jobs. Downloaded Linux logs confirm all eight failure cases, including the three actual FIFO waits, plus the release GUI and thirteen portal cases. [Hosted failure output](x11-setup-hosted.txt).

This closes the reproduced authority-open caller hang and bounds waiting for all OS-managed setup paths. Real stalled DNS/connect and remote-filesystem recovery, native desktop failures and broader T15 acceptance remain open. The bounded helper is a containment decision, not evidence that those OS operations themselves were cancelled.
