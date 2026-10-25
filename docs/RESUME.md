# Resume handoff

## Current state — 2026-10-25

The migration goal is active. Branch: `feature/rust-migration-preview`. Last pushed commit is `8bde30f51400e6d4c6de00b894b74051f73d0b03`; latest application change is `23aba0d9d3956629ba08d4c8146a10a424c03160` (Windows WGPU default and shared renderer selection). Static CRT fix is `441e679`. All four jobs pass in run `40232606248`; reuse this result and its downloaded artifacts instead of rebuilding unchanged source.

Read `AGENTS.md`, `STATUS.md`, `ACCEPTANCE.md` and `evidence/windows-vm-functional.md`. The last file records exact hashes, successful sessions, failed probes and remaining gates. Do not restart completed architecture/core/X11 work or treat the available Windows VM as unavailable.

## Windows lab

The source Model O lab was preserved. NanoClick has its own working overlay, UEFI/TPM state and read-only QA media at `artifacts/vm/windows11-lab/`. Source guidance/controller remain in `~/Git/ModelO_1-0-9/docs/windows-vm.md` and `scripts/vm/windows_lab.py`. The NanoClick VM was running at the last check; recheck QMP/process state rather than launching another instance. No network, shared folders, physical USB or Model O proxy is connected. The virtual USB tablet is permitted.

Windows 11 Pro build 26300.9457, two vCPU, 4 GiB RAM, Basic Display Adapter, 1280×800 at 100% were observed. Preview and capture target execute with WGPU; the VC++ redistributable remains absent. No reference-hardware performance acceptance is inferred.

The current app and capture processes remain open in the guest; NanoClick PID 6992 and capture PID 4136 were last observed, with PowerShell minimized and capture Ready. Recheck a screenshot and process state before continuing. The guest data is under `C:\NanoClickLab\FocusRepro1`; diagnostics and capture logs are never overwritten. A low-level Windows trace confirmed injected pairs at (650,350) with both target-under-pointer and foreground PID 4136, while the capture session still recorded zero. See the follow-up section in [Windows functional evidence](evidence/windows-vm-functional.md). This narrows the discrepancy to delivery past the low-level hook or the Winit/egui capture path; it does not yet identify a product defect.

Downloaded preview: `dist/preview-windows-8bde30f/`, including `qa/input_capture.exe` and commit/hash metadata. Read-only media `tools-wgpu-trace.iso` is mounted in the second virtual optical drive, E: during this session. Probe scripts and helper commands are ignored artifacts, not production code. The corrected probe selects the exact preview path, not a recursive first executable (the media also contains a legacy executable).

Guest input uses QMP only. Absolute coordinates use 0..32767; a center move was verified through the guest Cursor.Position as (640,400). The ignored `guest_pointer.py` is fixed to the observed 1280×800 display: update its mapping if resolution changes. Prefer explicit foreground activation by PID/title, then a fresh in-client pointer move and a visible Recording/reset-counter check before F6. Send commands without a trailing newline and use one explicit Return; inspect guest readiness at focus transitions. Do not assume rapid batched input or a producer counter proves target receipt.

## Next work

1. Continue isolating zero-event capture sessions against producer runs reporting ten. The temporary hook confirms injected events reach Windows while capture PID 4136 is foreground and under the pointer, yet the GUI counter remains zero. Confirm `CursorMoved` and native `WM_MOUSE*` delivery after focus. Minimize PowerShell before focusing the capture window. Keep all original logs/sessions; do not mark robust native delivery accepted until understood.
2. Continue Windows QA: N=1/large counts, manual preparation/cancellation, shortcuts/layout/conflicts, preferences/error flows, DPI/NVDA, resource stability and native timing protocols. Use this VM for functional tests and retain physical performance limits separately.
3. Native GNOME/KDE permission/input/accessibility and the remaining per-backend performance gates remain open. Preserve budgets and the C++ reference until the full matrix is accepted.

## Git and publication

The user authorized this feature branch and a draft PR, but no merge/release. Draft creation remains blocked by the previously rejected GitHub CreatePullRequest permission; do not repeatedly retry it or request authorization already granted. The prepared body is `/tmp/nanoclick-rust-migration-pr.md` if still present. Avoid full hosted runs for documentation-only updates.

The unrelated untracked `.aws` entry remains outside scope: never read, modify, remove, stage or commit it. Stage explicit project paths only. Documentation following the VM session may be committed locally without a documentation-only CI push.
