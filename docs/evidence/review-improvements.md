# Review improvements validation

Native reader continuation (2026-10-30): [Orca evidence](native-orca.md) records two reproduced accessibility defects, their regression fixes, nine native forward-focus transitions, generated selector/status speech, full local checks and byte-exact cleanup. Full cross-platform reader acceptance remains open.

## Wayland cancellation (item 6) — 2026-10-26

`scripts/verify-portal.sh` runs against its own non-activating D-Bus instance without DISPLAY/WAYLAND_DISPLAY. New async mock cases hold a press or release method for five seconds. Stop reached terminal Faulted in 8.172 ms for stalled press and 50.074 ms for stalled release; no incomplete pair was counted as sent and session Close was attempted exactly once. Release is attempted independently during cleanup even when the original operation is cancelled; failed confirmation remains an error. All thirteen existing permission/session/shortcut scenarios pass. These are mock cancellation results, not compositor timing or native input acceptance.

`futures_lite::block_on` drives the existing zbus future; it does not create an executor thread per click. The previous 10 ms timer raced the operation and did not impose a fixed 10 ms latency on successful calls. The revised operation has a separate deadline timer and checks cancellation at most every 5 ms while pending. Checked press/release remain sequential; speculative pipelining would weaken ordering/error attribution. Native achieved-rate and consent/revocation acceptance remain open.

## Session restoration and parenting (item 5)

The private-bus fixture confirms `persist_mode=2` on both preparations and restore tokens `[None, mock-token-1]`. Each one-click run finishes and closes its session. The token returned by Start replaces the consumed token in process memory. A new process starts with no restore token; disk preferences contain none. Parent handles are exported once for the live window and shared by input/shortcut sessions. Export is bounded to one second, with unparented fallback where unsupported. No compositor consent or input was exercised by this fixture.

The [RemoteDesktop portal contract](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html) defines persistence as a restoration hint; the portal still controls permission and token renewal. Real GNOME/KDE dialog suppression/revocation remains open.
