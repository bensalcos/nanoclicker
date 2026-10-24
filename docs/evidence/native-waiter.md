# Cancelable worker wait

The execution engine now accepts a `Waiter` independently of its input transport. `Engine::new` retains the portable default; the application selects `nanoclick_platform::waiter()` through `Engine::with_waiter`.

Linux uses a condition variable with a dedicated predicate mutex. Cancellation sets the atomic flag before acquiring the signal mutex and notifying; the waiter checks the flag under that same mutex before parking. The wait releases its signal mutex before the engine touches its state mutex. This avoids a lost notification and lock inversion with Stop. No periodic polling or busy waiting is added.

Windows uses an unnamed high-resolution one-shot waitable timer and an unnamed manual-reset cancellation event. The blocking wait lists cancellation first so it wins when both objects signal. Relative due times round up to 100 ns units; the engine always rechecks `Instant` and cancellation after waking. No process-wide `timeBeginPeriod` change is made, and the timer is disarmed before publishing a terminal state.

A fresh cancellation event is created for every run. An already cloned callback from an older token may execute after terminal cleanup; its `Arc<OwnedHandle>` keeps the old event valid without signalling a later run's event. Handles remain owned through synchronous FFI calls. Unsafe code is confined to the platform crate.

Timer binding, waiting and cleanup failures become observable terminal errors. Cleanup runs even when binding fails or a backend panics. Input-session cleanup remains independent and its error is preserved alongside timer failures. Cancellation callback installation immediately signals a token already cancelled before binding; concurrent cancellation may signal twice, so callbacks are idempotent.

## Verification

- Linux: 33 Rust tests pass, including timer failure, combined bind/cleanup failure, cancellation before callback installation, cancellation during a native wait, and 50 cancellation/parking races. Existing lifecycle, native-revocation and manual-countdown tests also pass.
- Windows MSVC target: full workspace and tests pass compilation/Clippy. Two additional Windows-only tests cover cancellation-event isolation and interruption of a long timer. These have **not** executed natively in this environment.
- Actual application flow passes in private Xvfb. Release cancellation measurements use a no-input sink: [4,000 samples summarized](cancellation-wait-port.csv), p99 38–50 µs and maximum 273 µs on this host; they are not native SendInput, portal or GUI Stop measurements.

Windows timer accuracy, scheduler deadline lateness and native Stop performance remain acceptance gates. A successful cross-target check does not close T03/T07.

## Primary API references

Checked 2026-10-24:

- [CreateWaitableTimerExW](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createwaitabletimerexw): high-resolution flag supported since Windows 10 version 1803; Windows 11 is the project baseline.
- [SetWaitableTimerEx](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimerex): negative relative due times in 100 ns units; actual accuracy depends on hardware.
- [WaitForMultipleObjects](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitformultipleobjects): lowest index wins when multiple handles signal; owners must keep handles valid throughout the wait.

## Hosted execution and stronger timer coverage

Run 40089494911 at 2ecaf23 executed the original two Windows-only event/cancellation tests successfully and linked the release executable. Review found that the original cancellation test could signal before `wait_until` armed a timer, so that green result alone did not prove timer arming/disarming.

The native wait is now factored into private arming and signal-wait helpers, with the public Waiter contract unchanged. The strengthened cancellation test arms a 60-second timer before cancellation can race, then always enters the native wait. A new test fires and rearms a timer twice. Both report worker errors through the result channel and use a bounded observer timeout. Their new hosted execution is pending; timing-accuracy and input Stop budgets remain separate.
