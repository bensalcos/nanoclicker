# Owned X11 test displays

Date: 2026-10-24. This change fixes test isolation; it does not change production input or renderer behavior.

## Reproduction

Two concurrent 150%/200% GUI checks failed before reaching finite-count coverage. Failure captures were byte-identical (SHA-256 `674f0d987f5b9d16e9c957c486be140c2f202cbd52b4f94ae6dd03808091f51b`) and showed the same 150% window with a GrabKey Access conflict. Sequential runs passed. Inspection of this host's `/usr/bin/xvfb-run` showed that `-a` selects a display number by scanning lock files before launching Xvfb, then continues after a startup delay. A launcher-only reproduction, issuing no input commands, confirmed that two simultaneous invocations both reported `:103`. [First output](x11-legacy-display-a.txt), [second output](x11-legacy-display-b.txt). The original failing GUI invocations did not record their display numbers; the separate reproduction establishes the launcher race directly.

## Fix and scope

`scripts/x11_fixture.py` creates one Xvfb with `-displayfd`, private `-auth` data, no TCP listener and the requested synthetic screen size. It verifies the owned process executable and both abstract/filesystem X11 socket names against that process's socket descriptors before returning an environment to a test. Display selection therefore belongs to Xvfb, rather than a free-number guess. Each private directory is temporary and its authority file is mode 0600; authentication uses a new random cookie. No host authentication file is read or changed.

The context terminates/resumes only its own server. It removes only unchanged resources whose identity and ownership were captured from that server, preserving unrelated replacements. The command wrapper propagates command failure and performs context cleanup on ordinary exit/interruption. SIGKILL and arbitrary uninterruptible kernel I/O cannot guarantee Python cleanup.

Local GUI, capture, software benchmark, native input/deadline/Stop scripts now use the shared wrapper. Existing requested screen sizes are preserved. The paused-server/FIFO regression driver shares the same ownership checks and uses a separate client authority file for FIFO cases. The Ubuntu container smoke still uses its single xvfb-run invocation inside an isolated container; no concurrent launcher is used there. Historical measurements retain their original methodology and source qualifications.

## Verification

Two real private-Xvfb unit tests pass: concurrent contexts have distinct verified displays and terminate their servers; the correct private cookie completes X11 setup while missing/incorrect cookies are rejected. Hosted Linux CI now runs these tests and preserves their log. Hosted run [40146929746](https://github.com/bensalcos/nanoclicker/actions/runs/40146929746), source 88bd9b4, passes all four jobs. The downloaded [fixture output](x11-owned-display-hosted.txt) confirms both real-Xvfb tests; its GUI, paused-resource and portal logs also pass.

The complete GUI checks also pass concurrently at 150% and 200% with the new launcher, recording different owned displays (`:2`/`:3`) and server PIDs (7338/7347). These runs used the working GUI harness with additional finite-count assertions; production executable source is 9051e65. See [finite GUI evidence](ui-finite-counts.md). Normal three-button/shortcut checks and all eight paused-server/FIFO cases pass with authenticated owned displays. This demonstrates the reproduced test race is removed for these paths; native desktop validation remains open.

## Startup observation follow-up

Run 40147506617 at 1256e01 passed Windows, MSRV and alternate-renderer jobs but failed the Ubuntu owned-server test before product GUI execution: one five-second readiness observation returned empty. Server stderr was discarded, so the original server-side cause cannot be determined from that run. This does not invalidate the earlier successful run at 88bd9b4 or establish a product startup regression.

The fixture now checks the same process after each short observation, continuing within a separate thirty-second absolute cold-server setup deadline. It does not restart a server on an empty observation. Startup errors retain process status and at most 8192 bytes of that owned server's stderr. The concurrent-test barrier follows the fixture deadline, aborts on a peer failure and retains both errors instead of masking the original one. Authentication and socket ownership checks are unchanged. The application's 500 ms warm-start target and all production I/O/performance budgets are unchanged.

Two regressions failed before this change: a controlled first empty readiness observation incorrectly aborted a real private Xvfb, and a deliberately invalid Xvfb option lost its diagnostic. All four real/private-resource tests now pass locally. [Recorded output](x11-startup-observer.txt). The observer fix handles those reproduced weaknesses; hosted confirmation and diagnosis of any recurrent server-side delay remain pending.
