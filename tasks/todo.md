# Rust backlog

Latest reader evidence (2026-10-30): [native Orca follow-up](../docs/evidence/native-orca.md) accepts scoped private Fedora default/changed-selector speech, forward/reverse navigation, popup option traversal and status cases after two fixes; the subsequently reproduced Escape focus loss is [fixed with native Fedora receipts](../docs/evidence/selector-focus.md). Full task checkboxes remain open for the remaining cross-platform matrix.

Status: implementation in progress; see docs/STATUS.md and docs/evidence/implementation.md. Size S: 1–2 files; M: 3–5 files. Split tasks that exceed that scope before implementing them, preserving dependencies and acceptance criteria. Paths below are planned locations.

## P0 — Feasibility

### T01 — Establish the comparison protocol [S]
- [x] Record machines, available systems, version matrix and the test capture fixture in `docs/evidence/environment.md`.
- [x] Define parity fixtures and measure C++ on Windows if available; otherwise leave that baseline explicitly pending.

**Dependencies:** none. **Verification:** reproducible protocol and evidence distinguished from pending results. **Files:** `docs/evidence/environment.md`, `docs/evidence/baseline.md`.

### T02 — Prototype the renderer and accessible UI [M]

Partial evidence: matched-source glow/wgpu runs on software X11 and a private GPU display are recorded in `docs/evidence/renderer-comparison.md`, `gpu-glow.md` and `gpu-renderers.md`. The private GPU cases meet measured startup/CPU/idle-RSS budgets; software-rendering RSS remains over budget. WGPU is the provisional default on Windows and Linux; Linux has an explicit Glow build alternative, and Windows always selects WGPU. The UI tokens, keyboard behavior and theme captures are implemented; native DPI, reader and full graphics-compatibility acceptance remain open.
- [x] Compare glow/wgpu measurements, define shared visual tokens/components and capture both themes; retain per-target defaults and renderer overrides.
- [ ] Complete the supported Windows/Linux DPI, minimum-window, screen-reader, compatibility and resource matrix before final renderer acceptance.

**Dependencies:** T01. **Verification:** Windows/Linux execution and review against UI.md/VALIDATION.md. **Files:** `experiments/ui/`, `docs/evidence/ui-spike.md`.

### T03 — Test Windows/X11 input and waiting [M]

Implementation and evidence: Linux condition-variable waiting and Windows high-resolution timer/cancellation event are implemented. The Windows 11 VM has received exact Count=1/10/1000 sessions for all three buttons, long-interval Stop/restart/close flows, and a current-source manual Start Count=10 receipt; large-count skipped slots remain functional evidence only. Native Windows timing, partial-insertion error injection and the shortcut conflict/layout matrix remain open. The passive capture target is included in the Windows preview. Manual countdown cancellation passed. See `docs/NATIVE_QA.md`, `docs/evidence/input-capture.md`, `docs/evidence/windows-vm-functional.md` and `docs/evidence/native-waiter.md`.
- [x] Select cancelable native waiting and verify cancellation/Stop paths without busy waiting in the implemented backends and available harnesses.
- [ ] Complete native partial-failure/release and timing acceptance on reference Windows/Linux systems.

**Dependencies:** T01. **Verification:** captured events, error tests and timings recorded per OS. **Files:** `experiments/input/`, `docs/evidence/input-spike.md`.

### T04 — Test Wayland portals [M]
- Partial evidence: production adapters are implemented and thirteen private-D-Bus lifecycle scenarios pass. Native compositor consent, target input, revocation and cost measurements remain open; mocks do not close this task. See `docs/evidence/portal-contract-tests.md` and `docs/evidence/wayland-lifecycle.md`.
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
- Source-level recovery refinement: `SendInput` now marks a button pressed only when exactly one event from its down/up pair was inserted; a zero-event result no longer triggers an unmatched cleanup release ([API return semantics](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)). Three fake-result regression tests cover zero insertion, partial-down cleanup and retry after a failed release. All three passed on the hosted Windows Server 2025 runner in workflow run `40281183415` at `68cd895`; Windows Clippy and the full workspace tests passed as well. This does not replace Windows 11 VM failure injection or reference-machine timing evidence.
- [x] Implement `SendInput` with checked record counts, partial-pair recovery, and the per-run cancelable Windows waiter.
- [x] Verify native left/right/middle pairs and finite counts through the Windows 11 VM capture target; exercise long-interval Stop/restart and close while active.
- [x] Exercise scripted zero/partial insertion and failed-release recovery through the test-only SendInput seam; all three regressions pass on hosted Windows Server 2025 in run `40281183415`.
- [ ] Collect reference-machine Stop/resource/timing evidence; native Windows 11 OS-level failure behavior remains unverified.

**Dependencies:** T03, T06. **Verification:** Windows capture fixture and error tests; do not use the user desktop for automated testing. **Files:** `platform/src/windows/`, Windows integration.

### T08 — Deliver UI Start/Stop on Linux X11 [M]
- [x] Implement the XTEST backend and UI/controller composition; connect the Windows backend through the shared contracts.
- [x] Verify Linux X11 manual countdown, acknowledged Stop and shutdown in the isolated release GUI harness; verify Windows F6 Start/Stop and long-wait cancellation in the user's VM.
- Current-source receipts: exact `c2f21a1` passed both Windows F6 and accessible manual Start Count=10 Left runs with matching receiver records (ten balanced pairs, no other-button events, anomalies or overflow). Manual Start invoked the three-second countdown path; its intermediate frame was not captured. See `docs/evidence/windows-vm-functional.md`.

**Dependencies:** T02, T03, T07. **Verification:** outside-UI clicks, Stop and shutdown on both systems. **Files:** `platform/src/x11/`, `app/src/{main,controller,ui}.rs`.

**Milestone P1:** [ ] Measured, usable Windows/X11 left-button flow; renderer-independent core.

## P2 — Parity

### T09 — Automate platform checks [S]

Accepted CI evidence: run 40092934864 at acc817c passed all four jobs; follow-up run 40121663794 at e244efd passes the expanded GUI/portal cases; latest run 40297722966 at c55c282 passes Windows, Ubuntu, alternate-renderer and Rust 1.95 jobs. Downloaded X11 logs and Windows preview archives were inspected. Native GUI/input/reader gates remain separate. See `docs/evidence/ci.md`.
Follow-up: owned Xvfb launcher/authentication checks and adoption by local wrappers/CI fix the reproduced concurrent display-number selection race. Two real fixture tests and concurrent full GUI checks pass locally; hosted run 40146929746 at 88bd9b4 also passes all four jobs, and its fixture/GUI logs are independently downloaded. See `docs/evidence/x11-owned-display.md`.
- [x] Windows/Linux CI with fmt, Clippy, tests, release build and valid features.
- [x] Failure artifacts and isolated X11 integration; distinguish these checks from native QA.

**Dependencies:** T08. **Verification:** successful hosted runs and logs; local YAML is not accepted CI evidence. **Files:** `.github/workflows/`, scripts if needed.

### T10 — Complete buttons and repetition [M]

Evidence: actual form editing verifies left/right/middle N=1 and N=7 at 100/150/200% private-X11 scaling, exact captured pairs, Completed, fresh run IDs/counters, unlocked editing and a quiet window. N=1000 for all three buttons also completes on X11 and Windows 11 VM; the VM's 10 ms runs missed periods and do not establish performance. Deterministic tests cover queued restart during Stopping. See `docs/evidence/ui-finite-counts.md` and `docs/evidence/windows-vm-functional.md`.

- [x] Validate all three buttons, finite counts and infinite-mode Stop through the UI; snapshots expose count and stop reason.
- [x] Verify automatic completion unlocks controls, active editing is disabled, and queued restart uses a fresh run identity in the X11 harness.
- [x] Verify Windows finite delivery for N=1/10/1000 on all three buttons; retain VM timing limitations separately.

**Dependencies:** T09. **Verification:** core and capture tests, N=1/large N, rapid sequences and down/up failure. **Files:** `app/src/ui.rs`, `controller.rs`, existing adapters/tests.

### T11 — Integrate Windows/X11 shortcuts [M]

A reproduced frontend parity bug is fixed: queued activations are delivered individually, restart waits for Stop acknowledgement, and explicit Stop/Close supersedes older pending requests. Three deterministic regressions and the existing actual X11 GUI suite pass. Native burst/layout acceptance remains open; see `docs/evidence/shortcut-queue.md`.

Additional X11 evidence: pointer-only map changes preserve the binding, actual keyboard-map changes invalidate it, and unchanged XKB master-device transitions retain it. Native layout/group and Windows/Wayland acceptance remain open. See `docs/evidence/x11-shortcut-mapping.md`.
- [x] Implement default F6, rebinding and conflict-safe registration; shortcut actions produce prioritized state transitions.
- [x] Verify the English UI's effective shortcut and outside-focus Start/Stop in X11 and Windows VM flows.
- [ ] Complete Windows/X11 held-key, conflict, modifier and keyboard-layout acceptance; native Wayland trigger acceptance is tracked under T12.

**Dependencies:** T10. **Verification:** conflicts, modifiers/layout, held key and messages in terminal states. **Files:** hotkey adapters, UI/controller.

### T12 — Integrate Wayland and capabilities [M]

Partial protocol evidence: thirteen real-D-Bus mock scenarios pass on an activation-disabled private bus, including five GlobalShortcuts service-loss/closure cases before and after binding. This exercises the production adapter and engine without a compositor; real GNOME/KDE portal and input acceptance remains open. See `docs/evidence/portal-contract-tests.md`.
- [x] Implement portal adapter lifecycle, cancelable preparation and revocation handling; distinguish injection from shortcut capabilities in the app.
- [x] Disable Start when input injection is unavailable and support a desktop-assigned trigger with manual mode when no global shortcut is available.
- [ ] Verify actual desktop-assigned trigger, permission denial/revocation and target delivery on GNOME and KDE; protocol mocks are not native acceptance.

**Dependencies:** T04, T08, T10. **Verification:** native GNOME/KDE flows, denial and revocation; separate XWayland testing. **Files:** `platform/src/wayland/`, `capabilities.rs`, controller and capability view.

**Milestone P2:** [ ] Complete parity matrix, with evidence for each supported environment and visible limitations.

## P3 — Usability

### T13 — Persist recoverable preferences [M]

Implementation: versioned explicit Save and seven recovery tests pass on Linux and hosted Windows. Actual release X11 Save/relaunch checks restored interval, unsaved-edit discard and no automatic Start at 100/200% scale. Windows 11 VM verifies saved fields, unsaved-edit discard, idle startup, `fe80737` corrupt/future-version recovery with exact backups, blocked-parent protection and backup-creation failure with source preservation. A temporary Linux X11-only build verifies invalid-file recovery/relaunch and failed-backup error/source preservation in private Xvfb; it omits Wayland and AccessKit because host development metadata is missing. Full-feature Linux and native Wayland error/recovery acceptance remain pending. See `docs/evidence/ui-preferences.md`.
- [x] Theme, plan and requested shortcut in a versioned schema, without active state or automatic resumption.
- [x] Verify versioned preferences, safe replacement, protected-path behavior and standard config directories through Linux tests and Windows CI/VM Save/relaunch.
- [x] Verify invalid-file recovery and exact backup preservation in the Windows 11 GUI.
- [x] Verify unsupported-version recovery and backup-creation failure with source preservation in the native Windows GUI.
- [x] Verify backup-creation error and source preservation in isolated Linux X11 GUI.
- [ ] Verify write-error presentation and recovery in native Wayland GUI.
- [x] Verify invalid-file recovery and post-recovery relaunch in the Linux X11 GUI on isolated Xvfb.
- [ ] Verify recovery in native Wayland and full-feature Linux builds; keep portal consent and desktop acceptance separate.

**Dependencies:** T11, T12. **Verification:** roundtrip, corruption, future version and failed write preserving the last valid file. **Files:** `app/src/settings.rs`, integration/controller and tests.

### T14 — Accept UI, DPI and accessibility [M]

Partial evidence: names/values verified in the actual AccessKit tree; Tab/Shift+Tab order, Enter behavior, token contrast and primary-action bounds have tests. Xvfb full flows pass at 100/150/200%, including minimum-size scrolling; default help clipping is fixed. The exact `c2f21a1` preview passes Windows 11 VM visual checks at 100% and 125% on 1280×800, including live scale changes in both directions with Start visible; the VM is restored to 100%. Multiple-monitor/other-display coverage, the wider minimum-size and 200% matrix, broader UI polish and NVDA/Orca remain pending. See `docs/evidence/ui-scaling.md` and `ui-accessibility.md`.
- [ ] Polish tokens, focus, errors, scaling and themes against UI.md with actual captures.
- [ ] Complete keyboard and NVDA/Orca testing, without clipping at 420×520 and 200% DPI.

**Dependencies:** T13. **Verification:** visual matrix and checklist with per-environment results. **Files:** `app/src/{ui,theme}.rs`, `docs/evidence/ui-qa.md`.

### T15 — Harden lifecycle and failure handling [M]

Windows VM evidence (2026-10-25): the capture-target zero-event condition was reproduced when no in-client pointer position was known, and a fresh in-client move restored event receipt. Exact Count=1/10/1000 was received for Left/Right/Middle; 1000 at 10 ms missed 1185–1265 periods and is not timing acceptance. Preference Save/relaunch/discard and long-wait Stop/restart/active close are documented. Manual cancellation passed. Manual countdown completion currently has no matching target receipt: a recent session reached Preparing, but the app returned to Ready while the capture remained at zero; focus transitions in that probe make it inconclusive, so do not classify it as a product failure or a pass. See `docs/evidence/windows-vm-functional.md` and `docs/RESUME.md`.

Current-source follow-up (2026-10-26): `68cd895` has two matching Windows 11 VM Count=10/F6 Left receipts, tied to that commit. The exact `0a3380c` artifact now has a clean F6 Count=10 Left receipt: one new receiver log reports 10 balanced pairs, no extra events/overflow; producer run 9 reports 10 sent, zero failures and one skipped period. VM timing is informational. Manual Start completion remains open; the latest UIA invocation completed producer run 11 but the receiver reported no in-client cursor position and recorded zero events. See `docs/evidence/windows-vm-functional.md`.

Partial evidence: five private-Xvfb failure cases now cover stalled input with/without Stop, cancellation during protocol preparation, disconnection and shortcut registration timeout; see `docs/evidence/x11-stalled-server.md`. Three additional private-authority-FIFO cases verify Stop, setup timeout and shortcut shutdown. A four-slot helper contains OS-managed setup without unbounded thread accumulation; underlying blocked-OS recovery remains open. See `docs/evidence/x11-setup-cancellation.md`. Private-bus tests cover cancellation during preparation, failed release recovery, early session closure, revocation and portal owner loss/replacement. Actual X11 release UI also passes 1000 measured Start/Stop cycles with stable sampled resources and graceful close during preparation/running/Stop-request races at 100/200% scale. Windows/Wayland desktop/resource stability and other failure variants remain open. See `docs/evidence/ui-lifecycle-stress.md`.
- [ ] Cover closing in Preparing/Running/Stopping, disconnected backend, revoked permissions and failed release.
- [ ] Ensure observable terminal state and closed resources; repeat cycles and verify memory stability.

**Dependencies:** T14. **Verification:** injected failures and bounded native stress; aggregate logs without per-click events. **Files:** affected engine/controller/adapters, tests and evidence.

**Milestone P3:** [ ] Normal and failure flows accepted with verified accessibility.

## P4 — Performance and delivery

### T16 — Measure and optimize bottlenecks [M]

- [x] Compare two full 2 Hz controls against two full 1 Hz candidate trials; all captured 6001 matching pairs, with active CPU at 4.6998%/4.9831% and 2.8165%/2.9833%, respectively.
- [x] Observe the 1 Hz progress footer during actual 100-click/s X11 operation; it shows the immediate first pair and legible updates about once per second.
- [x] Compare current-source 1 Hz Glow and WGPU software runs; Glow has lower CPU, WGPU saves about 1.14 MiB RSS, and both remain above the memory target.
- [x] Compare two current-source 1 Hz trials per renderer on the private AMD GPU display; WGPU averages 53.284 ms startup p95 and 58.827 MiB idle RSS versus Glow at 112.062 ms and 83.084 MiB, with near-equal CPU and exact 6001-pair capture.
- [ ] Reduce RSS below 100 MiB without regressing input/Stop or other renderers/backends; retain native reader/Wayland checks separately.

Current provisional probe (2026-10-26): Linux WGPU now explicitly compiles Vulkan/WGSL without GLES. Ubuntu 24.04 `cargo check` and release build pass; `cargo check --locked --offline --target x86_64-pc-windows-msvc -p nanoclick` also passes on the current source. The Linux GUI acceptance harness passes across manual Start, shortcuts, Stop/close, finite counts and preferences. The binary is 2.49% smaller, but one quick profile measured 109.980 MiB idle RSS and 3.5994% active CPU. The older binary could not complete a matched profile with the current harness, so there is no accepted RSS or CPU improvement. Keep the runtime-performance claim provisional and repeat matched profiles; the exact `0a3380c` Windows artifact now passes one F6 Count=10 Left receipt in the user's existing Windows 11 VM. Review Vulkan-only Linux runtime requirements and keep timing, RSS, accessibility and compatibility gates open. The VM procedure and isolation constraints are in `AGENTS.md`.

Feature-graph correction (2026-10-26): the Linux target-specific `eframe` feature had kept `wgpu` in the Glow alternative. It is now activated only by the app's `wgpu` feature; `cargo tree` shows no `wgpu` for the Glow configuration. The separated Glow release artifact passes full isolated X11 GUI acceptance and one exploratory quick profile records 107.305 MiB idle RSS/3.0999% CPU; memory remains over budget. The 63-test workspace/all-target suite and Clippy pass for both default WGPU and Glow feature sets. Repeat full profiles; the exact `0a3380c` Windows artifact has a matching F6 functional receipt, while manual Start and performance budgets remain open.

Partial evidence: actual release GUI measurements on Xvfb show the earlier 2 Hz progress change reduced active CPU from 19.67% to about 5.23%, while RSS remained above target. Two full 60 s runs at 2 Hz measured 4.6998%/4.9831% active CPU and 107.137/107.301 MiB idle RSS; two full 1 Hz runs measured 2.8165%/2.9833% active CPU and 107.309/107.059 MiB idle RSS, with 6001 captured pairs each. Both matched trials show about 40% lower CPU at 1 Hz. Two full current-source 1 Hz WGPU software runs measured 3.2998%/3.4998% active CPU and 106.141/105.945 MiB idle RSS, each with 6001 pairs. Glow is faster in software; WGPU saves about 1.14 MiB but neither closes the RSS budget. An actual-UI X11 observation at 100 clicks/s shows a legible count increment about once per second and an immediate first pair; screenshot load makes that observation unsuitable for performance claims. Two short 1 Hz profiles measured 2.60%/2.80% with 1001 pairs each. Two current-source physical-GPU trials per renderer now show WGPU startup p95 of 53.221/53.346 ms and idle RSS of 61.684/55.969 MiB, versus Glow at 110.392/113.732 ms and 83.602/82.566 MiB; active CPU was near equal and all runs captured 6001 pairs. WGPU is the provisional Linux default, with Glow retained as an alternative build. Software RSS, Vulkan compatibility, native backend and accessibility gates remain open. Native X11 input completed 60-second no-added-load and two-load-thread timing runs with matching captured pairs; this excludes the GUI and other backends. An additional 4,000 native X11 Stop cycles retain matching captured pairs. The actual release GUI now also exports engine timing: two 60-second private GPU-display runs show p99 upper bounds of 0.4/0.1 ms without added load/with two CPU load threads, respectively; see `docs/evidence/gui-deadline-lateness.md`. The matched-source Glow/WGPU GPU-display experiments meet startup/CPU/idle-RSS targets in that case (see `docs/evidence/gpu-renderers.md`); software and other native performance gates remain open. Latest current-binary 10-second quick profiles on executable SHA-256 `2cab232157fc957bbfaceca1909f9816ab0ed9c47d28ea110b6cded435828031` measured 105.258 MiB idle RSS on Vulkan/lavapipe, 104.832 MiB with `MALLOC_ARENA_MAX=1` (rejected; only 0.426 MiB lower), and 117.746 MiB on WGPU OpenGL software (rejected). All three received exactly 1001 pairs; these single exploratory samples do not accept the 100 MiB RSS gate. See `docs/evidence/ui-polish-performance.md` and `docs/evidence/optimization-ledger.md`. See `docs/evidence/gpu-glow.md`, `docs/evidence/ui-performance.md`, `docs/evidence/ui-polish-performance.md`, `docs/evidence/ui-progress-feedback.md`, `docs/evidence/renderer-comparison.md`, `docs/evidence/optimization-ledger.md`, `docs/evidence/x11-deadline-lateness.md` and `docs/evidence/x11-stop-latency.md`.

Follow-up triage on 2026-10-26: one valid quick profile with `GLIBC_TUNABLES=glibc.malloc.trim_threshold=0` measured 104.719 MiB idle RSS and 104.871 MiB active RSS with 1001 matching pairs; one first attempt exited before active measurement and was discarded. The single valid result is only about 0.54 MiB below the matching current-binary baseline and remains above budget, so the tunable is rejected for product use. This adds no full-profile acceptance; continue investigating the application/renderer footprint.

The documented LLVMpipe `LP_NUM_THREADS=1` setting was also sampled on the WGPU OpenGL software route: 117.105 MiB idle RSS, 2.8986% active CPU and 1000 pairs, versus the prior 117.746 MiB/4.2992%/1001-pair quick sample. This is an unmatched exploratory result, remains far above budget, does not affect Vulkan/lavapipe, and is rejected for product use. T16 remains open.
- [ ] Measure the VALIDATION.md table per renderer/backend, compare C++ where a baseline exists and profile deviations.
- [ ] Optimize only reproducible findings; verify that no improvement worsens Stop, CPU or precision without a recorded decision.

**Dependencies:** T15. **Verification:** release, capture fixture, 30 starts, load tests and 1000 Stops; data tied to commits. **Files:** `benches/`, harness, `docs/evidence/performance.md` and profiled module.

### T17 — Package and verify clean installation [M]

Partial evidence: Linux and Windows preview archives include notices/checksums and separate symbols; the current WGPU-default Linux bundle passes the restricted Ubuntu 24.04 GUI flow, and the Windows README is generated with Windows-specific WGPU guidance. Renderer-aware packaging now emits an isolated Glow dependency/license inventory and correct OpenGL/EGL runtime README; the local Glow package passes the full X11 GUI flow. The exact `c2f21a1` Windows preview executes in the user's Windows 11 VM with matching F6 and manual Start delivery receipts. Native clean-reference installation and final artifact acceptance remain pending. See `docs/evidence/packaging.md` and `docs/evidence/windows-vm-functional.md`.
- [x] Produce and verify Windows ZIP/PDB and Linux tar.gz/symbol previews with icons/notices/checksums and recorded runtime requirements.
- [ ] Run on clean reference installations; review features, dependencies/licenses and glibc baseline.

**Dependencies:** T16. **Verification:** execute the final artifact, not only `cargo run`; record the audit and resources used. **Files:** packaging scripts, artifact workflow, documentation.

### T18 — Close the migration and update documentation [M]

Prepared documentation: README, `CHANGELOG.md`, `docs/ACCEPTANCE.md` and `docs/ROLLBACK.md` now describe the implemented preview, evidence boundaries and preserved C++ reference. Final acceptance and legacy retirement remain gated by the native/performance matrix; no checkbox is narrowed to documentation alone.
- [x] Maintain the Rust README, build/usage/permission guidance, changelog, rollback guide and evidence/acceptance matrix.
- [ ] Remove C++/Visual Studio in a separate change after acceptance; preserve a historical reference and review MIT/icon ownership before distribution.

**Dependencies:** T17 and all milestones. **Verification:** reproduction from a clean checkout, links and rollback guide to the C++ commit; publication only with the required scope and credentials. **Files:** README/changelog/docs and subsequent legacy removal.

**Milestone P4:** [ ] Parity, budgets and native matrix accepted; artifacts ready for distribution.
