# Shortcut activation delivery and Stop priority

A focused check of T11 reproduced a frontend bug: the previous `take_toggle` swapped the entire activation count to zero and returned its parity. Two callbacks before a UI update therefore produced no transition. Three callbacks produced only one. This could discard a legitimate Stop/Start sequence.

The fixed-size atomic counter now consumes one activation per handling pass. Remaining activations request another UI update; a queued restart stays pending while the engine is Stopping and resumes after its terminal wake. The UI does not poll that queue while waiting for Stop acknowledgement. Escape/close is handled before shortcut starts, and explicit Stop/manual actions clear older pending activations. A later physical activation remains a new request.

Three regressions failed before the fix and now pass: two queued callbacks are consumed individually without replay; a restart remains queued while a fake backend blocks cleanup and starts once after acknowledgement; explicit Stop discards pending starts. These headless tests send no desktop input. [Test output](shortcut-queue-tests.txt).

All 63 local all-targets Rust tests pass (25 core, 22 app, 7 platform, 9 capture-target). Linux feature/wgpu Clippy, Windows MSVC cross-target Clippy and Rust 1.95 checks pass. The rebuilt release product also passes the existing private-X11 GUI suite, including outside-focus shortcut, countdown, three finite buttons, active close and preferences. [GUI output](shortcut-queue-gui.txt). Release executable SHA-256: `ce31f3f2ebd14ed3c97534d99aca3aec341ba84c8189870d3b592f6705e64ced`.

This confirms callback delivery and acknowledgement ordering under deterministic gates plus normal real-X11 GUI flows. It does not claim a native Windows/Wayland burst capture or performance acceptance of the new revision. T10/T11 remain open for their full platform matrix. Hosted run [40177331385](https://github.com/bensalcos/nanoclicker/actions/runs/40177331385), source 42abf7d, passes all four jobs, confirming the product fix's build/unit/integration checks. Native Windows/Wayland burst capture and broader acceptance remain open.
