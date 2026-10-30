# UI accessibility implementation and evidence

Native continuation (2026-10-30): [private Fedora GNOME/Orca evidence](native-orca.md) confirms default and changed selector speech, forward/reverse Tab traversal, popup option traversal and two lifecycle announcements after fixing native name/description gaps. The historical implementation-only evidence below does not independently prove native reader behavior. Human listening and the wider reader matrix remain open.

The form now supplies semantic names for Interval, Count, Global shortcut, Mouse button and Theme. Visible labels are linked with `labelled_by`; the shortcut and theme controls use explicit accessible names where their role is otherwise unnamed. Current values remain exposed independently of those names.

The footer is measured before reserving the form's scroll area, but its interactive control is created after the form. This keeps Start/Stop fixed at the bottom while Tab follows the visible reading order. Theme is created before Save even though their group is right-aligned. No custom keyboard event loop replaces egui's focus management.

Session status uses an atomic polite live region. Counter updates remain ordinary text, avoiding continuous live announcements. Real reader behavior is still pending validation.

## Automated evidence

Five additional tests bring the workspace total to 38:

- The generated AccessKit tree exposes the five named fields/selectors and their current values.
- Actual window-view Tab order is Theme, Save, Interval, Mouse button, Until stopped, Count mode, Count field, Global shortcut, Apply, Start. Reverse traversal is checked too, allowing egui's documented next-frame focus update.
- Enter in Interval or Global shortcut does not start the engine.
- The primary action stays within 420×520, retains a 46-unit target and exists in Idle, Preparing, Running, Stopping and Faulted views with very long errors and maximum-length counters. The same tree exposes polite atomic status.
- Both themes check body/secondary text contrast ≥4.5:1, enabled control boundary/focus contrast ≥3:1 and selected text ≥4.5:1. Control outlines are opaque and at least one logical unit wide; focus uses two. These checks cover palette tokens and their specified fills, not every antialiased pixel or every operating-system visual state.

The view tests use the production rendering functions, an unregistered shortcut and an engine sink that cannot send desktop input. They neither read user preferences nor connect to desktop services. Texture deltas from the headless egui passes are explicitly cleared.

Checks passed: full workspace tests, Linux Clippy including X11 examples, Windows MSVC target Clippy/tests compilation, alternate wgpu feature Clippy and Rust 1.95 compatibility check. Windows tests were not executed natively.

## Runtime evidence and remaining acceptance

[Scaling review](ui-scaling.md) records actual Xvfb UI flows and captures at 100/150/200%, including minimum-size scrolling and active states. It demonstrates the isolated X11 path, separately from the headless tree tests.

Native NVDA/Orca operation, announcement cadence, real monitor transitions and keyboard use under actual window managers remain required. Enabling AccessKit, passing token checks or emitting a live region alone does not establish complete accessibility acceptance.
