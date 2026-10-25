# Finite counts through the release GUI

Date: 2026-10-24. Production executable source: `9051e65a23a8f5bf8d40025f87b5667c3bb9de58`, release default glow. SHA-256: `f5565bf1e5ef6f2cc9fc3a084bffacb64472693ff2f506d8ea620590de7ec413`. The harness additions are introduced with this evidence. Final 100% harness executable SHA-256: `7293de3e19fc76a2925ee70f3fd2c2d51e25f397b9315612048fd6494e4cde88`.

## Actual GUI protocol

The private X11 fixture launches a fresh actual application for each mouse button. It edits the interval to 25 ms, selects the button in the visible combo box, clicks Count, and edits N through the real text field. It moves focus/pointer to its empty capture target and presses the registered F6 once. It never constructs an engine plan or seeds finite preferences behind the UI.

For N=1 and then N=7 in the same process, it requires exactly N ordered press/release pairs for the selected native button (left=1, right=3, middle=2), a ready primary action and a terminal diagnostics record. The terminal run identity advances from 1 to 2; attempts/sent match N, failures are zero and the reason is Completed. The second edit and differing exact count verify controls are editable again and counters do not carry over. A further 250 ms observation must show no late input. This is an observed quiet window, not a universal temporal guarantee. No Stop shortcut is sent to end these runs.

All three buttons pass both counts at 100%, 150% and 200% synthetic X11 scaling: eighteen finite runs in total. [100% output](ui-finite-100.txt), [150% output](ui-finite-150.txt), [200% output](ui-finite-200.txt). The final 100% log also includes the six terminal JSON records. The 150%/200% harness checked the same fields internally before its pass markers; those runs preceded the stdout-only addition that prints the records.

The 150%/200% checks ran concurrently on distinct verified, authenticated private displays; see [owned display isolation](x11-owned-display.md). Their full existing prelude and Save/relaunch checks also pass. An earlier attempt with xvfb-run exposed a launcher race; that failure is documented rather than counted as product acceptance. The 100% run used the final owned launcher and terminal-output harness.

## Scope

These tests exercise real form editing, global Start, automatic completion, per-run identity/counters and subsequent editing on private Xvfb. The existing manual-start test separately covers the three-second countdown; finite runs use the immediate shortcut. The diagnostics exporter records the terminal snapshot observed by the UI, rather than an OCR assertion of footer wording. The fixture reads the actual primary-action ready color after completion.

The 25 ms / short-run samples do not establish timing budgets or no skipped periods under all loads. Very large N, restart during Stopping, other interval boundaries, native Windows/Wayland, real monitor transitions and screen readers remain separate acceptance work. T10 remains open for its full matrix. Hosted run [40146929746](https://github.com/bensalcos/nanoclicker/actions/runs/40146929746), source 88bd9b4, passes all four jobs. Its downloaded [GUI output](ui-finite-hosted.txt) confirms all six finite cases at 100% plus the existing prelude, twenty measured stress cycles, three close cases and Save/relaunch. Its stable sampled RSS remains over budget; this does not accept memory performance.

## Large-count follow-up

The existing GUI fixture now also edits N=1000 and a 1 ms requested interval, then verifies exactly 1000 acknowledged/captured complete pairs for each of left/right/middle. These three additional cases pass locally at 100% scale using the release producer from 42abf7d (SHA-256 `ce31f3f2ebd14ed3c97534d99aca3aec341ba84c8189870d3b592f6705e64ced`). Session IDs advance to 3 after the existing N=1/7 cases, controls become editable again and the 250 ms quiet window still passes. [Output including terminal records](ui-finite-large.txt).

The large case has a fifteen-second test safety timeout; N=1/7 retain their four-second timeout. This tests completion rather than a throughput deadline: requesting 1 ms does not promise sustained 1000 clicks/s. The middle-button sample records three skipped scheduling periods while still completing all 1000 pairs with zero failures. Native large-count and 150/200% large-count acceptance are not inferred from this 100% result.

All four hosted jobs pass for the producer's shortcut fix at 42abf7d, run 40177331385. The expanded large-count fixture is a subsequent test-only change; its hosted confirmation is pending.
