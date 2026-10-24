# Initial isolated X11 GUI acceptance

Date: 2026-10-24. Command: `scripts/verify-ui-x11.sh`. Actual application binary, debug build, dark theme, 520×600 window and temporary configuration. Private 1024×768×24 Xvfb, without interaction with the user's desktop. Later release/stress runs are tracked separately in ui-lifecycle-stress.md and ci.md.

Observed results:

- PASS: F6 starts/stops while an external receiver window has focus.
- PASS: the target receives complete down/up pairs; after the UI returns to Ready, no additional events arrive during 250 ms.
- PASS: actual Start click, move the pointer to the target and wait; the three-second preparation is retained, followed by received pairs.
- PASS: WM_DELETE_WINDOW closes the process within the timeout.

The harness waits for the window and primary action's visual state before acting. Coordinates/colors are fixtures for the default window in x86_64 Xvfb, not general evidence for every DPI/layout. Finite counts and three buttons have separate core/platform capture coverage.

This exceeds controller unit-test scope but does not establish behavior on an actual X11 compositor, Wayland, Windows, screen readers, DPI or every failure state. At this initial increment there was no hosted GitHub run; subsequent hosted logs and accepted CI scope are recorded in ci.md.
