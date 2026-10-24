# Implementation history

Historical increments below retain their original test counts and pending checks. See STATUS.md for current results; early limitations are not a current completion report.

## Initial increment

- Pinned toolchain: Rust 1.97.1; edition 2024, declared MSRV 1.95 (not yet executed with 1.95 at this stage).
- Three-crate workspace. Core has no unsafe or UI/OS types; validated configuration, monotonic scheduler and persistent worker.
- First verification: 10 core tests passed; warning-free Clippy before adapters. An additional revocation test was added first and failed on the missing API before the signal was implemented.
- Initial waiter: std Condvar, interrupted under the same mutex as its predicate. This is a functional baseline, not evidence of 1 ms Windows precision. Compare waitable timers in P0/T16 before accepting budgets.
- eframe 0.37.0 requires Rust 1.95. Glow was the initial candidate, pending wgpu comparison.
- ashpd 0.13.13: remote_desktop imports screencast types without enabling that feature automatically; screencast was enabled for compilation without requesting screen capture. Bounded async-io client in the backend; no Tokio in the engine.
- Adapters written: paired SendInput, XTEST and RemoteDesktop portal with cleanup. Windows compilation and native testing were still pending.
- Initial English UI with separated logic/control and notification-driven repainting. Its provisional screen identified Wayland shortcuts as pending; T12 was not complete.

API sources: versioned Cargo registry source/documentation for eframe/egui 0.37.0, ashpd 0.13.13, global-hotkey 0.9.0 and x11rb 0.13.2; general primary links in docs/architecture/RUST.md. Test/probe results are added after executing each check.

## Subsequent evidence

- `cargo check --workspace --target x86_64-pc-windows-msvc --locked -j 2`: succeeded; a warning for the Linux-only ctx field was fixed afterwards. This was neither a linked build nor native Windows testing.
- `scripts/verify-x11.sh`: PASS for 10 ordered complete Left/Middle/Right pairs in Xvfb; PASS for outside-focus shortcut, repeat suppression while holding F6 for 900 ms, and preserving the original registration after a conflict.
- Replaced Linux's global-hotkey listener (50 ms polling found in its source) with an X11 listener waiting on socket/pipe and shutdown signal, without periodic polling. At this stage global-hotkey remained the parser and Windows implementation (MOD_NOREPEAT); the later Windows replacement is recorded in windows-shortcut.md.
- First reviewed Xvfb screenshot: shortcuts required scrolling at 520×600. Spacing was adjusted; final capture was pending. Xvfb captures do not replace native/DPI/reader QA.
- Core after panic recovery/revocation signal: 13 passing tests, including 200 Start/Stop cycles and recovery from a panicking backend. A regression first reproduced the UI remaining Running after worker death; the engine now publishes Faulted and permits retry.

## First integrated slice verification (2026-10-24)

- `cargo test --workspace --locked -j 2`: 16 tests passed (14 core, 2 controller), including visible first-pair progress during a long interval.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS Linux.
- `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked -- -D warnings`: PASS. The X11 capture example requires x11-tests in the Linux matrix; it is not an omitted Windows test.
- Final 520×600 Xvfb captures at `screenshots/linux-x11-{dark,light}.png` were visually reviewed: three complete sections, improved text contrast and visible primary action. This did not establish native DPI, keyboard or reader behavior.
- Remaining at this stage: preference persistence, artifacts/hosted CI, benchmarks, real Wayland consent/denial/revocation and Windows/KDE validation. No accepted parity or release readiness was claimed.

## Manual preparation after permission

The initial countdown lived in controller and elapsed before the worker requested Wayland permission. It moved into the engine as an optional wait after sink preparation. Snapshots publish ready_at during that wait; shortcut Start uses zero delay. Stop/shutdown/revocation interrupt it and sink cleanup finishes before acknowledgement.

Three additional tests cover a deadline after preparation, cleanup on cancellation and rejection of excessive delay. At this increment, 27 total tests and Linux Clippy passed; the actual X11 executable's GUI flow passed with the new sequence. This protocol test does not establish real Wayland portal consent.
