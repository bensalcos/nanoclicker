# X11 protocol stalls and cancellation

Date: 2026-10-24. Tests run on a private Fedora 44 Xvfb, without connecting to the user's display. The native fixture is a debug build; these single-case timings do not replace the release 1000-stop-per-interval acceptance protocol.

The original XTEST connection used x11rb 0.13.2's DefaultStream, whose readiness poll has no timeout. A real paused-server regression confirmed the worker in native polling, then requested Stop: no terminal state appeared within two seconds. The driver resumed its server so the old blocking destructor could exit. A second regression found the shortcut registration timeout followed by an unbounded worker join; its caller still had not returned after five seconds.

The input and shortcut connections now share a safe Rust deadline stream. Protocol setup/version queries have a three-second budget. Each pair and best-effort release exchange has a 250 ms budget. Blocked readiness polls request timeouts of at most ten milliseconds and check the existing cancellation token between polls; after cancellation each exchange gets 25 ms grace, including cleanup. The stream also checks its absolute deadline before nonblocking reads/writes, so irrelevant readable data cannot extend an operation indefinitely. It never replaces the engine waiter's notification hook.

The shortcut listener retains its event-driven idle wait and shutdown pipe. Fresh protocol deadlines apply when processing events; closing the listener also signals its private cancellation token. Keyboard-map queries and registration therefore use the same timed I/O. Authentication/address selection preserves x11rb's public connection semantics. Core/app still forbid unsafe code; the implementation adds no dependency or per-click allocation/logging.

## Regression results

`python3 scripts/verify-x11-stall.py` passes five cases:

| Case | Observed outcome |
|---|---|
| Stop during stalled input | Faulted/Failed, one attempted pair, zero acknowledged pairs, one failure; terminal observed in 85.617 ms |
| Stalled input without Stop | Faulted/Failed with the same counters; terminal observed about 750.222 ms after the driver observation command |
| Stop during protocol preparation | Idle/Stopped, zero attempts; terminal observed in 10.582 ms |
| Server disconnected during input | Faulted with retained reset/cleanup errors; state may already be terminal before the subsequent Stop request |
| Shortcut registration against a paused server | Error returned and worker joined in 3000.534 ms |

The first implementation bounded replies but waited about 750 ms after Stop. Adding a separate short cancellation grace reduced this observed emergency-failure case to about 86 ms while preserving release attempts. The uncancelled case exercises the regular 250 ms exchange budgets independently. [Recorded output](x11-stalled-server.txt).

Input timeout explicitly reports that delivery/release could not be verified. X11 requests already written cannot be recalled; queued input can be processed when the server resumes. Faulted does not certify successful delivery or release. The fixture verifies acknowledged counters and failure reporting, rather than claiming no target input after an unverified operation.

## Isolation and verification

The driver starts its own Xvfb with no TCP listener and an empty synthetic screen. It obtains the display through that process's descriptor, verifies its executable and ownership of both filesystem/abstract listening sockets, uses a private empty Xauthority/configuration, and waits for confirmed suspension before releasing the worker. It requires the named worker's actual /proc polling wait before issuing Stop/observation. Successful child shutdown must occur while the server remains paused. Finally it resumes/terminates only its own process and removes only unchanged owned filesystem resources. Unrelated or replaced paths are preserved. Unavailable polling observation fails the case rather than accepting it.

Two unit tests cover expired deadlines on an already-ready socket and cancellation-aware preparation with permitted recovery I/O. All 49 local Rust tests, normal three-button/X11 shortcut regressions, Linux/X11-feature/wgpu Clippy, Windows MSVC cross-target Clippy and Rust 1.95 checks pass. An independent read-only static review found no blocking/major issue; the uncancelled case was added from its coverage recommendation. Hosted run [40144962142](https://github.com/bensalcos/nanoclicker/actions/runs/40144962142), source `1142026d45dede0b589483b7cb856ffae3ca4f3e`, passes all four jobs: Linux, Windows Server 2025, alternate renderer and Rust 1.95. The downloaded Linux logs confirm all five stalled-server cases, thirteen private-bus lifecycle cases and the release GUI flows. [Hosted stall output](x11-stalled-server-hosted.txt).

## Remaining scope

These bounds cover X11 protocol exchanges. OS-managed DNS/socket establishment and Xauthority-file lookup remain outside the demonstrated bound and require further audit. The GUI toolkit's own display connection, native Windows/Wayland failures and other desktop/resource cases remain separate acceptance work. T15 remains open.

Primary API evidence: installed, pinned x11rb 0.13.2 Stream documentation requires nonblocking read/write and permits custom readiness polling; DefaultStream delegates its poll with no timeout. The wrapper uses rustix 1.1.5 timed poll, retries interrupted polls against the same deadline, and borrows descriptors without unsafe code.

## Healthy release regression after the fix

Source: `1142026d45dede0b589483b7cb856ffae3ca4f3e`. The same native release Stop fixture described in [the original protocol](x11-stop-latency.md) passes another 4000 cycles. The fixture SHA-256 is `13cab2513fca0bb289de923c327965fa724a1b462357432e0c3e51ec07e7cbcf`.

| Interval ms | Samples | p50 us | p95 us | p99 us | Maximum us | Captured pairs |
|---|---|---|---|---|---|---|
| 1 | 1000 | 181.501 | 683.353 | 1838.764 | 3729.665 | 1721 |
| 10 | 1000 | 176.401 | 888.298 | 1923.433 | 3145.168 | 1000 |
| 100 | 1000 | 173.506 | 862.359 | 2121.525 | 3019.071 | 1000 |
| 999999 | 1000 | 170.731 | 812.346 | 1869.601 | 3986.879 | 1000 |

All captured pairs match terminal counts and remain complete. Tail latency is higher than the earlier sample; both samples remain below the unchanged 20 ms p99 / 100 ms maximum budgets. Shared-host sequential measurements do not isolate the cause of that difference. [Complete output](x11-stop-after-deadlines.txt). This is healthy native X11 evidence, excluding the GUI and other backends.

The rebuilt release GUI also passes the private GPU display quick profile and full functional preflight: manual countdown, shortcut after idle, minimum window, active close and Save/relaunch. Executable SHA-256: `480a916edb2a034cd99d06fc113987444c512da0d097bd2473ef8aa98550e49b`. Ten seconds each of stabilization, idle and active sampling yielded 1001 matching pairs, zero failures/skips, engine p99 upper bound 400 us and maximum 564.853 us. Idle RSS was 86.277 MiB; active CPU 1.4993% of one core. Zero idle CPU ticks were observed. This short smoke does not replace the full timing protocol or native desktop acceptance. The manual preflight required one emulated-seat cursor correction; measured input required none. See [measurements](x11-stream-gui-smoke.csv), [functional output](x11-stream-gui-validation.txt) and [terminal record](x11-stream-gui-terminal.jsonl).
