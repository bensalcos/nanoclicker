# Review improvements validation

## Wayland cancellation (item 6) — 2026-10-26

`scripts/verify-portal.sh` runs against its own non-activating D-Bus instance without DISPLAY/WAYLAND_DISPLAY. New async mock cases hold a press or release method for five seconds. Stop reached terminal Faulted in 8.172 ms for stalled press and 50.074 ms for stalled release; no incomplete pair was counted as sent and session Close was attempted exactly once. Release is attempted independently during cleanup even when the original operation is cancelled; failed confirmation remains an error. All thirteen existing permission/session/shortcut scenarios pass. These are mock cancellation results, not compositor timing or native input acceptance.

`futures_lite::block_on` drives the existing zbus future; it does not create an executor thread per click. The previous 10 ms timer raced the operation and did not impose a fixed 10 ms latency on successful calls. The revised operation has a separate deadline timer and checks cancellation at most every 5 ms while pending. Checked press/release remain sequential; speculative pipelining would weaken ordering/error attribution. Native achieved-rate and consent/revocation acceptance remain open.
