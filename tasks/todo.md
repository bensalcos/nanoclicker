# Rust backlog

Status: implementation in progress; see docs/STATUS.md and docs/evidence/implementation.md. Size S: 1–2 files; M: 3–5 files. Split tasks that exceed that scope before implementing them, preserving dependencies and acceptance criteria. Paths below are planned locations.

## P0 — Feasibility

### T01 — Establish the comparison protocol [S]
- [x] Record machines, available systems, version matrix and the test capture fixture in `docs/evidence/environment.md`.
- [x] Define parity fixtures and measure C++ on Windows if available; otherwise leave that baseline explicitly pending.

**Dependencies:** none. **Verification:** reproducible protocol and evidence distinguished from pending results. **Files:** `docs/evidence/environment.md`, `docs/evidence/baseline.md`.

### T02 — Prototype the renderer and accessible UI [M]

Partial evidence: matched-source release glow/wgpu runs on software X11 recorded in `docs/evidence/renderer-comparison.md`. Both exceed memory and active CPU budgets. Linux default remains provisional. A subsequent full glow run on a private AMD GPU-backed display meets startup/CPU/RSS targets in that case; software rendering remains over budget. Matched-source GPU glow/wgpu now both meet the measured startup/CPU/idle-RSS targets in the private display case; see `docs/evidence/gpu-renderers.md`. Windows and native accessibility acceptance remain pending. See `docs/evidence/gpu-glow.md`.
- [ ] Compare glow/wgpu with the minimum window, keyboard, DPI and accessibility tree; record size, startup and idle use.
- [ ] Select a renderer and define visual tokens/components with captures of both themes; if neither meets the criteria, evaluate an architectural alternative.

**Dependencies:** T01. **Verification:** Windows/Linux execution and review against UI.md/VALIDATION.md. **Files:** `experiments/ui/`, `docs/evidence/ui-spike.md`.

### T03 — Test Windows/X11 input and waiting [M]

Implementation: Windows high-resolution timer and per-run cancellation event compile; Linux condition-variable and X11 tests pass. Native Windows timing remains pending. The passive native capture target and manual protocol are prepared; nine unit tests and an actual private-X11 target check pass, while real Windows/Wayland capture runs and a downloadable Windows target bundle remain pending. See `docs/NATIVE_QA.md`, `docs/evidence/input-capture.md` and `docs/evidence/native-waiter.md`.
- [ ] Inject pairs into an isolated capture fixture, detect errors and release the button after partial failure; test shortcut registration/conflicts.
- [ ] Select cancelable native waiting and check Stop during a long interval without busy waiting.

**Dependencies:** T01. **Verification:** captured events, error tests and timings recorded per OS. **Files:** `experiments/input/`, `docs/evidence/input-spike.md`.

### T04 — Test Wayland portals [M]
- [ ] On GNOME/KDE, test a POINTER session and an outside-focus shortcut against a native Wayland client; document the assigned trigger.
- [ ] Test denial/revocation and measure NotifyPointerButton cost; select the D-Bus client and execution model.

**Dependencies:** T01. **Verification:** capability/version table, events and measurements; unavailable environments remain pending. **Files:** `experiments/wayland/`, `docs/evidence/wayland-spike.md`.

**Milestone P0:** [ ] Evidence supports UI/platform decisions; no critical limitation is hidden.

## P1 — First usable flow

### T05 — Create the workspace and validated domain [M]
- [x] Add three-crate manifests, toolchain/MSRV, lockfile and compatible per-target features.
- [x] Model interval, button, count and states with typed errors; reject invalid bounds.

**Execution note:** domain/workspace implementation proceeds while T02/T03/T04 collect desktop evidence; this does not close P0 or freeze the renderer.

**Dependencies:** T02 and T01. **Verification:** fmt, Clippy and boundary tests; per-target builds. **Files:** manifests/toolchain, `crates/core/src/config.rs`, `state.rs` and minimal entry points.

### T06 — Implement sessions with a fake backend [M]
- [x] Monotonic scheduler, immediate first click, exact N, skipped slots and bounded metrics.
- [x] Worker with priority Stop/Shutdown, terminal snapshot and run_id; Stop and shutdown interrupt waiting.

**Dependencies:** T05. **Verification:** fake clock, race, saturation and overflow tests; no sleeps for arithmetic tests. **Files:** `core/src/{engine,scheduler,ports,metrics}.rs`, tests.

### T07 — Integrate the Windows backend [M]
- [ ] SendInput wrapper with checked return values, button recovery and the selected cancelable waiter.
- [ ] Actual left-button Start/Stop from the harness; shutdown leaves no active worker or leaked handle.

**Dependencies:** T03, T06. **Verification:** Windows capture fixture and error tests; do not use the user desktop for automated testing. **Files:** `platform/src/windows/`, Windows integration.

### T08 — Deliver UI Start/Stop on Linux X11 [M]
- [ ] XTEST backend and UI/controller composition using existing contracts; connect Windows as well.
- [ ] UI supports manual Start with a cancelable countdown and acknowledged Stop without blocking the graphics thread.

**Dependencies:** T02, T03, T07. **Verification:** outside-UI clicks, Stop and shutdown on both systems. **Files:** `platform/src/x11/`, `app/src/{main,controller,ui}.rs`.

**Milestone P1:** [ ] Measured, usable Windows/X11 left-button flow; renderer-independent core.

## P2 — Parity

### T09 — Automate platform checks [S]

Accepted CI evidence: run 40092934864 at acc817c passed all four jobs; follow-up run 40121663794 at e244efd also passes all jobs and the expanded GUI/portal cases. Downloaded X11 logs and Windows preview archives were inspected. Native GUI/input/reader gates remain separate. See `docs/evidence/ci.md`.
Follow-up: owned Xvfb launcher/authentication checks and adoption by local wrappers/CI fix the reproduced concurrent display-number selection race. Two real fixture tests and concurrent full GUI checks pass locally; hosted run 40146929746 at 88bd9b4 also passes all four jobs, and its fixture/GUI logs are independently downloaded. See `docs/evidence/x11-owned-display.md`.
- [x] Windows/Linux CI with fmt, Clippy, tests, release build and valid features.
- [x] Failure artifacts and isolated X11 integration; distinguish these checks from native QA.

**Dependencies:** T08. **Verification:** successful hosted runs and logs; local YAML is not accepted CI evidence. **Files:** `.github/workflows/`, scripts if needed.

### T10 — Complete buttons and repetition [M]

Partial GUI evidence: actual form editing now verifies left/right/middle N=1 and N=7 at 100/150/200% private-X11 scaling, exact captured pairs, Completed, fresh run IDs/counters, unlocked editing and a 250 ms quiet window. See `docs/evidence/ui-finite-counts.md`. Large counts, restart during Stopping and native Windows/Wayland remain open.

- [ ] Three buttons, infinite mode and N validated through the UI; snapshots show count and stop reason.
- [ ] Automatic completion unlocks controls; active editing is disabled and rapid restart does not reuse previous state.

**Dependencies:** T09. **Verification:** core and capture tests, N=1/large N, rapid sequences and down/up failure. **Files:** `app/src/ui.rs`, `controller.rs`, existing adapters/tests.

### T11 — Integrate Windows/X11 shortcuts [M]

Additional X11 evidence: pointer-only map changes preserve the binding, actual keyboard-map changes invalidate it, and unchanged XKB master-device transitions retain it. Native layout/group and Windows/Wayland acceptance remain open. See `docs/evidence/x11-shortcut-mapping.md`.
- [ ] Default F6 and rebinding without losing a valid binding on conflict; one press produces one transition.
- [ ] The English UI shows the effective shortcut and allows outside-focus Stop.

**Dependencies:** T10. **Verification:** conflicts, modifiers/layout, held key and messages in terminal states. **Files:** hotkey adapters, UI/controller.

### T12 — Integrate Wayland and capabilities [M]

Partial protocol evidence: thirteen real-D-Bus mock scenarios pass on an activation-disabled private bus, including five GlobalShortcuts service-loss/closure cases before and after binding. This exercises the production adapter and engine without a compositor; real GNOME/KDE portal and input acceptance remains open. See `docs/evidence/portal-contract-tests.md`.
- [ ] Portal adapter with cancelable preparation, sessions and revocation signals; UI distinguishes input injection from shortcuts.
- [ ] Desktop-assigned trigger and explicit manual mode when no shortcut exists; unavailable input injection prevents Start.

**Dependencies:** T04, T08, T10. **Verification:** native GNOME/KDE flows, denial and revocation; separate XWayland testing. **Files:** `platform/src/wayland/`, `capabilities.rs`, controller and capability view.

**Milestone P2:** [ ] Complete parity matrix, with evidence for each supported environment and visible limitations.

## P3 — Usability

### T13 — Persist recoverable preferences [M]

Implementation: versioned explicit Save and seven recovery tests pass on Linux and hosted Windows. Actual release X11 Save/relaunch now also checks the restored interval, unsaved-edit discard and no automatic Start at 100/200% scale. Native Windows/Wayland desktop QA and other preference fields remain pending. See `docs/evidence/ui-preferences.md`.
- [x] Theme, plan and requested shortcut in a versioned schema, without active state or automatic resumption.
- [ ] Safe writing/replacement on each OS, invalid-file recovery and standard directories.

**Dependencies:** T11, T12. **Verification:** roundtrip, corruption, future version and failed write preserving the last valid file. **Files:** `app/src/settings.rs`, integration/controller and tests.

### T14 — Accept UI, DPI and accessibility [M]

Partial evidence: names/values verified in the actual AccessKit tree; Tab/Shift+Tab order, Enter behavior, token contrast and primary-action bounds have tests. Xvfb full flows pass at 100/150/200%, including minimum-size scrolling; default help clipping is fixed. Native DPI and NVDA/Orca remain pending. See `docs/evidence/ui-scaling.md` and `ui-accessibility.md`.
- [ ] Polish tokens, focus, errors, scaling and themes against UI.md with actual captures.
- [ ] Complete keyboard and NVDA/Orca testing, without clipping at 420×520 and 200% DPI.

**Dependencies:** T13. **Verification:** visual matrix and checklist with per-environment results. **Files:** `app/src/{ui,theme}.rs`, `docs/evidence/ui-qa.md`.

### T15 — Harden lifecycle and failure handling [M]

Partial evidence: five private-Xvfb failure cases now cover stalled input with/without Stop, cancellation during protocol preparation, disconnection and shortcut registration timeout; see `docs/evidence/x11-stalled-server.md`. Three additional private-authority-FIFO cases verify Stop, setup timeout and shortcut shutdown. A four-slot helper contains OS-managed setup without unbounded thread accumulation; underlying blocked-OS recovery remains open. See `docs/evidence/x11-setup-cancellation.md`. Private-bus tests cover cancellation during preparation, failed release recovery, early session closure, revocation and portal owner loss/replacement. Actual X11 release UI also passes 1000 measured Start/Stop cycles with stable sampled resources and graceful close during preparation/running/Stop-request races at 100/200% scale. Windows/Wayland desktop/resource stability and other failure variants remain open. See `docs/evidence/ui-lifecycle-stress.md`.
- [ ] Cover closing in Preparing/Running/Stopping, disconnected backend, revoked permissions and failed release.
- [ ] Ensure observable terminal state and closed resources; repeat cycles and verify memory stability.

**Dependencies:** T14. **Verification:** injected failures and bounded native stress; aggregate logs without per-click events. **Files:** affected engine/controller/adapters, tests and evidence.

**Milestone P3:** [ ] Normal and failure flows accepted with verified accessibility.

## P4 — Performance and delivery

### T16 — Measure and optimize bottlenecks [M]

Partial evidence: actual release GUI measured on Xvfb; 2 Hz progress lowers software-rendering CPU from 19.67% to 5.23%. Active CPU and RSS budgets remain unmet. Native X11 input completed 60-second no-added-load and two-load-thread timing runs with matching captured pairs; this excludes the GUI and other backends. An additional 4,000 native X11 Stop cycles retain matching captured pairs. The actual release GUI now also exports engine timing: two 60-second private GPU-display runs show p99 upper bounds of 0.4/0.1 ms without added load/with two CPU load threads, respectively; see `docs/evidence/gui-deadline-lateness.md`. The matched-source glow/wgpu GPU-display experiments both meet startup/CPU/idle-RSS targets in that case (see `docs/evidence/gpu-renderers.md`); software and other native performance gates remain open. See `docs/evidence/gpu-glow.md`. See `docs/evidence/ui-performance.md`, `docs/evidence/x11-deadline-lateness.md` and `docs/evidence/x11-stop-latency.md`.
- [ ] Measure the VALIDATION.md table per renderer/backend, compare C++ where a baseline exists and profile deviations.
- [ ] Optimize only reproducible findings; verify that no improvement worsens Stop, CPU or precision without a recorded decision.

**Dependencies:** T15. **Verification:** release, capture fixture, 30 starts, load tests and 1000 Stops; data tied to commits. **Files:** `benches/`, harness, `docs/evidence/performance.md` and profiled module.

### T17 — Package and verify clean installation [M]

Partial evidence: refreshed Linux preview at 88bd9b4 with notices, source archives, checksums and separate symbols passes expanded isolated Ubuntu 24.04 GUI execution, including all three finite button plans; a second packaging pass has identical archive hashes and GNU debuglink CRC matches the supplied symbols. Windows CI now produces a ZIP/PDB preview with zero unresolved notices; the downloaded 88bd9b4 archives pass independent structural/hash verification. Eight packaging tests pass; Windows runtime and native desktop acceptance remain pending. See `docs/evidence/packaging.md`.
- [ ] Windows ZIP and Linux tar.gz with icons, licenses, checksums, requirements and separate symbols.
- [ ] Run on clean reference installations; review features, dependencies/licenses and glibc baseline.

**Dependencies:** T16. **Verification:** execute the final artifact, not only `cargo run`; record the audit and resources used. **Files:** packaging scripts, artifact workflow, documentation.

### T18 — Close the migration and update documentation [M]

Prepared documentation: README, `CHANGELOG.md`, `docs/ACCEPTANCE.md` and `docs/ROLLBACK.md` now describe the implemented preview, evidence boundaries and preserved C++ reference. Final acceptance and legacy retirement remain gated by the native/performance matrix; no checkbox is narrowed to documentation alone.
- [ ] Rust README with build, usage, permissions and actual limitations; changelog and final evidence matrix.
- [ ] Remove C++/Visual Studio in a separate change after acceptance; preserve a historical reference and review MIT/icon ownership before distribution.

**Dependencies:** T17 and all milestones. **Verification:** reproduction from a clean checkout, links and rollback guide to the C++ commit; publication only with the required scope and credentials. **Files:** README/changelog/docs and subsequent legacy removal.

**Milestone P4:** [ ] Parity, budgets and native matrix accepted; artifacts ready for distribution.
