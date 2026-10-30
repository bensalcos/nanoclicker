# NanoClick review: fixes and improvements backlog

Native reader follow-up (2026-10-30): two reproduced accessibility defects now have focused regressions and [private Fedora GNOME/Orca receipts](../docs/evidence/native-orca.md). Default selector speech, forward traversal and preparing/cancelled status announcements pass. The unchanged artifact additionally passes Light/Dark/Right/Middle value speech, popup option traversal and nine reverse Tab steps. The Escape focus-loss case is now [fixed and natively verified](../docs/evidence/selector-focus.md), including outside-click behavior. Other lifecycle/active-counter behavior, human listening, Ubuntu/KDE and NVDA remain pending; migration acceptance is unchanged.

Source: code review of `feature/rust-migration-preview` against `master` (2026-10-26).
Audience: implementation agent (Codex).

## Ground rules for the implementer

- Read `AGENTS.md`, `docs/RESUME.md`, `docs/STATUS.md` and `docs/ACCEPTANCE.md` first.
- Every item below is a **candidate** finding. Confirm it in code (and with a failing test where practical)
  before changing behavior. If an item turns out to be wrong, record why in this file and skip it.
- One item per commit, with a unit test or a documented manual check. Keep text and docs in English.
- Windows behavior must be validated in the NanoClick Windows 11 VM lab as described in `AGENTS.md`;
  record the evidence under `docs/evidence/`.
- Do not touch the untracked `.aws` entry. Stage explicit paths only.
- After code changes run `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test --workspace`
  and `graphify update .`.

Status legend: **Spot-checked** = the cited code was looked at and matches the description;
**Unverified** = reported by the review, not yet confirmed.

---

## P1 — Bugs to fix

### 1. Window cannot stay on a secondary monitor (Windows)
- **Where:** `crates/app/src/ui.rs` ~L340–365 (`constrained_window_geometry` / `fit_window_to_monitor`)
- **Status:** Spot-checked. The position is clamped to `[0, monitor_size - window_size]`, assuming the
  monitor starts at the virtual-desktop origin, with a fixed `TASKBAR_RESERVE_PX` at the bottom.
- **Failure:** Drag NanoClick to a monitor at x ≥ 1920 or negative x → next frame snaps it back to the
  primary monitor. Taskbars on top/left/right are mishandled.
- **Fix:** Use the current monitor's *work area* origin and size (e.g. `MonitorFromWindow` +
  `GetMonitorInfoW.rcWork` on Windows; winit monitor position elsewhere). Only re-fit on first show,
  monitor change or scale-factor change, not every frame.
- **Acceptance:** Unit tests for a monitor with non-zero/negative origin and for a top taskbar; VM check
  with a second virtual display if feasible.

### 2. Shortcut presses during the startup probe are replayed as Start
- **Where:** `crates/app/src/ui.rs` ~L193 (`process_shortcuts`)
- **Status:** Spot-checked. The function returns early while `self.available` is not `Some(Ok(()))`,
  leaving queued activations in place.
- **Failure:** User presses F6 during the ≤3 s probe (or while the probe has failed). When the probe
  succeeds the queued activation toggles the engine and clicking starts without a fresh user action.
- **Fix:** While unavailable, drain/clear pending shortcut activations instead of returning early.
- **Acceptance:** Unit test: activation queued while unavailable → no toggle after availability turns Ok.

### 3. Preferences from a newer version are treated as "damaged"
- **Where:** `crates/app/src/settings.rs` ~L66 (`#[serde(deny_unknown_fields)]` on `Preferences`)
- **Status:** Spot-checked (attribute present). Ordering of parse vs. version check is unverified.
- **Failure:** A file written by a future version (extra fields, `version: 2`) fails serde parsing before
  the version check, so it is reported as invalid and the recovery button offers to replace it.
- **Fix:** Parse a minimal `{ version: u32 }` probe first. Unsupported versions → protected error,
  no recovery offered. Then parse the v1 schema with `deny_unknown_fields`.
- **Acceptance:** Tests: future-version file → "unsupported version" error without recovery; corrupt
  v1 file → still offers recovery (keep the behavior from commit `fe80737`).

### 4. Typed count is lost when "Until stopped" is saved
- **Where:** `crates/app/src/settings.rs` ~L115–130 (`Preferences::from_draft`) and `draft()`
- **Status:** Spot-checked. `count` becomes `None` for `UntilStopped`; `draft()` restores a hard-coded 10.
- **Failure:** Count=500 → switch to "Until stopped" → Save → relaunch → choose Count → shows 10.
- **Fix:** Store `count: u64` and the repeat mode separately. This needs a v2 preferences schema with a
  v1 → v2 migration (coordinate with item 3).
- **Acceptance:** Round-trip test; migration test from a v1 file.

### 5. Wayland: consent dialog on every Start
- **Where:** `crates/platform/src/wayland.rs` ~L60–75 / ~L160 (`WaylandInput::open`)
- **Status:** Spot-checked (no `persist_mode` / `restore_token` in the file).
- **Failure:** Each Start creates a new RemoteDesktop session, so GNOME/KDE show the permission dialog
  every time (up to 120 s wait). The global shortcut is impractical.
- **Fix:** Request `persist_mode = ExplicitlyRevoked`, keep the `restore_token` (memory, or preferences
  if persisted), and/or reuse the session across runs. Pass a parent `WindowIdentifier`.
- **Acceptance:** Manual check on GNOME Wayland: second Start shows no dialog. Document in `docs/evidence/`.

### 6. Wayland: click path is slow and Stop cannot interrupt it
- **Where:** `crates/platform/src/wayland.rs` ~L19 (`bounded`), ~L181–210 (press/release calls)
- **Status:** Partially spot-checked (`bounded(..., None)` calls exist; performance claim unverified).
- **Failure:** Two sequential D-Bus round trips per click, each wrapped in a timer race polling every
  10 ms; cancellation is `None`, so a stalled portal delays Stop by up to ~2 s per click.
- **Fix:** One persistent executor per session; pass the run's `Cancellation` into these calls; use a
  deadline timer instead of a 10 ms poll loop; consider pipelining press/release.
- **Acceptance:** Measure achieved rate at 1–10 ms intervals before/after; test that Stop returns promptly
  with a stalled mock portal.

### 7. Wayland: global shortcut is unregistered after every launch
- **Where:** `crates/app/src/hotkey.rs` ~L29
- **Status:** Unverified.
- **Failure:** After configuring via "Configure in desktop…" and restarting, status shows "Not configured".
- **Fix:** On Wayland startup, call `BindShortcuts` with the same id (GNOME/KDE reuse the assignment without
  a dialog), or persist a "portal configured" flag and re-bind automatically.

### 8. X11: shortcut grabbed at the wrong shift level on non-US layouts
- **Where:** `crates/platform/src/x11_shortcut.rs` ~L123
- **Status:** Unverified.
- **Failure:** French AZERTY: `Ctrl+Digit1` grabs Ctrl+<`&` key> without Shift, so it never fires.
- **Fix:** Prefer level-0 keysym matches; if the only match is level 1, add `ShiftMask` to the grab; report
  ambiguous mappings to the UI.
- **Acceptance:** Unit test over a synthetic keysym table (AZERTY-like).

---

## P2 — UX improvements

### 9. Unsaved-changes tracking
- **Where:** `crates/app/src/ui.rs` ~L174 (close / Escape handling)
- **Problem:** Edits since the last Save are silently discarded on close.
- **Proposal:** Dirty flag compared against the last loaded/saved `Preferences`; show "Unsaved changes"
  next to Save; on close either prompt (Save / Discard / Cancel) or auto-save when validation passes.
  Pick one and document it in `docs/UI.md`.

---

## P3 — Cleanup

### 10. Legacy C++ sources at the repo root
- **Where:** `Autoclicker.cpp/.h/.rc/.vcxproj/.slnx`, `AutoclickerEngine.*`, `MainWindow.*`, `framework.h`,
  `targetver.h`, `Resource.h`
- **Problem:** Not built by CI, drifts silently, unclear which implementation is authoritative.
- **Proposal:** Only after the Rust migration is accepted (see `docs/ROLLBACK.md`): tag the last C++ commit,
  then move the files to `legacy/` or delete them; update README and CI. **Do not do this before acceptance
  without asking the maintainer.**

---

## Suggested order

1. Items 2, 3, 4 (small, testable, app-level).
2. Item 1 (needs VM validation).
3. Items 5, 6, 7 (Wayland portal work; can share one session-lifecycle refactor).
4. Item 8, then 9.
5. Item 10 last, gated on migration acceptance.

## Implementation record

- **Item 2 — implemented:** shortcut activations are cleared while the capability probe is pending or failed. The regression test confirms that becoming available does not replay Start. `cargo test --locked -p nanoclick shortcuts_received_while_unavailable_are_not_replayed` passes on Linux. Native Windows verification remains pending for the final reviewed artifact.
- **Item 3 — implemented:** a minimal version probe runs before strict schema decoding. Unsupported versions expose no recovery action and cannot be replaced through Save/recovery; malformed current-version files remain recoverable. Nine settings tests pass, including future fields with a changed field representation. This supersedes the earlier preview behavior that allowed replacing version-99 files.
- **Item 4 — implemented:** schema v2 stores `count` and `repeat_mode` independently. Strict v1 migration preserves finite counts and maps null to remembered Count=10/Until stopped, without rewriting until Save. Eleven settings tests pass, including Count=500 in Until stopped and both v1 forms. X11 fixtures now expect v2.
- **Item 1 — implemented; native QA pending:** Windows queries `MonitorFromWindow`/`GetMonitorInfoW.rcWork`, transforms physical desktop coordinates using the current scale, and refits only on first geometry/monitor/work-area/scale change. Three geometry tests pass (positive/negative monitor origin, top/left taskbar, exact chrome). Windows MSVC all-target compilation passes. The final artifact still requires VM display/scale validation; a second display must be checked if the virtual adapter exposes one.
- **Item 8 — implemented:** X11 resolves unshifted keys first, adds Shift for a level-1-only keysym, and rejects ambiguous base mappings or unsupported groups. Three synthetic-layout tests pass, including AZERTY Ctrl+Digit1 resolution. Native layout QA remains open.
- **Item 6 — implemented in part; native rate QA pending:** press/release now observe the run token, request deadlines use a dedicated timer, and cancellation cleanup separately attempts release/close with bounded waits. Private stalled-press/stalled-release regressions reach terminal state in 8.172/50.074 ms, with no complete pair falsely counted; all thirteen prior lifecycle cases pass. The existing `block_on` races do not create per-click threads or impose a 10 ms delay on successful calls; zbus retains its connection executor. Sequential checked press/release is retained to preserve ordering and error visibility. Native GNOME/KDE achieved-rate measurements at 1–10 ms remain unverified.
- **Item 5 — implemented; compositor QA pending:** a process-local PortalContext requests ExplicitlyRevoked persistence, consumes each single-use restore token and stores its successor only after successful Start. Tokens are not serialized and never bypass consent. The app exports an owned parent-window identifier for RemoteDesktop and shortcut dialogs (completed native export, graceful unsupported fallback). The private portal verifies the second run receives the first run's replacement token and both sessions close; all prior scenarios pass. GNOME/KDE second-Start dialog behavior remains a native acceptance gate.
- **Item 9 — implemented:** a semantic dirty comparison tracks valid or invalid edits, successful Save/recovery refreshes the baseline, and close presents Save/Discard/Cancel after stopping input. Failed Save cannot close the app; Cancel retains edits; shortcuts are drained while the decision is open. Eight UI tests pass, including dirty/save/error/close decisions and existing focus/layout regressions. The private-X11 close fixture explicitly selects Discard; native GUI checks remain pending for the final artifact.
- **Item 7 — implemented; native rebinding QA pending:** successful Wayland assignment automatically persists only a rebind-intent flag, without saving unrelated edits. Startup rebinds stable id `toggle` only after a prior explicit assignment. Actual registration/error status still comes from the portal. A regression confirms assignment persistence keeps unsaved form changes separate; all 33 app tests pass. The portal contract permits a configuration dialog, so unconditional dialog-free rebinding is not claimed; GNOME/KDE restart behavior remains native QA.
- **Item 2 follow-up — same-frame probe completion covered:** queued activations are cleared before applying a probe result, covering a coalesced shortcut/probe wake even when no unavailable frame was rendered. The extended regression passes.
- **Item 10 — correctly gated, unchanged:** migration acceptance is still open. C++ retirement is deferred by `docs/ROLLBACK.md`; removing/moving the reference now would violate the review's explicit prerequisite.
- **Item 9 follow-up — graphical regression fixed:** batched Alt-release events exposed a close-dialog keyboard bug in the real X11 run. Save/Discard now consume event-level shortcut modifiers. The regression passes; the complete release/Xvfb flow subsequently passes global/manual control, active-close cases, all three buttons at N=1/7/1000, and preferences v2 Save/relaunch/discard. The earlier timed-out run is rejected as acceptance evidence.
- **Item 5 lifetime follow-up:** the native parent export is completed during window creation rather than abandoned on a timer. ashpd's internal native worker borrows the display, so dropping its future at a timeout would be unsafe across shutdown. The owned completed export remains shared for session lifetime. This changes Linux-only code; Windows review artifact e39e759 has identical Windows behavior.
