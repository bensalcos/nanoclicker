# Resume handoff

## Current state — 2026-10-25

The migration goal is active. Branch: `feature/rust-migration-preview`. Last pushed commit is `b8908e7b42b4b4a8f6fbd89c449dda3203ad883a`; latest product change is `23aba0d9d3956629ba08d4c8146a10a424c03160` (Windows WGPU default and shared renderer selection). Commit `b8908e7` instruments the passive QA target. All four jobs pass in run `40236185840`; its Windows preview is downloaded under `dist/preview-windows-b8908e7/` and its QA target was executed in the VM.

Read `AGENTS.md`, `STATUS.md`, `ACCEPTANCE.md` and `evidence/windows-vm-functional.md`. The last file records exact hashes, successful sessions, failed probes and remaining gates. Do not restart completed architecture/core/X11 work or treat the available Windows VM as unavailable.

## Windows lab

The source Model O lab was preserved. NanoClick has its own working overlay, UEFI/TPM state and read-only QA media at `artifacts/vm/windows11-lab/`. Source guidance/controller remain in `~/Git/ModelO_1-0-9/docs/windows-vm.md` and `scripts/vm/windows_lab.py`. The NanoClick VM was running at the last check; recheck QMP/process state rather than launching another instance. No network, shared folders, physical USB or Model O proxy is connected. The virtual USB tablet is permitted.

Windows 11 Pro build 26300.9457, two vCPU, 4 GiB RAM, Basic Display Adapter, 1280×800 at 100% were observed. Preview and capture target execute with WGPU; the VC++ redistributable remains absent. No reference-hardware performance acceptance is inferred.

At the latest recheck, Windows 11 and the NanoClick/`input_capture` processes are running. The app has been restored to saved defaults (System, Left, 100 ms, Count=10, F6) and reopened Ready with zero pairs sent. Native preferences testing also restored a saved Light/Right/37 ms/Count=7/F7 configuration, discarded a later unsaved 91 ms interval on relaunch, then saved the defaults back into the VM. Windows Count=1 and Count=1000 at 10 ms have been received for Left, Right and Middle with exact balanced pairs and no anomalies. The large-count runs reported 1185–1265 skipped periods in the VM, so they do not establish timing performance. The passive target remains available; recheck its cursor position before the next run. PowerShell remains in the background with old probe/inspection output; its red diagnostics are unrelated to NanoClick. The latest guest data is under `C:\NanoClickLab\PointerDiag1`; diagnostics and capture logs use new paths. A matching zero-event failure mode was reproduced: without a known in-window pointer position egui-winit discarded mouse-button events; after moving inside the focused target, a QEMU-tablet click registered and Count=10 produced ten complete left pairs with no anomalies. The exact cursor state of the two original sessions was not recorded. See [Windows functional evidence](evidence/windows-vm-functional.md).

Downloaded preview: `dist/preview-windows-b8908e7/`, including the instrumented `qa/input_capture.exe` and commit/hash metadata. Read-only media `tools-wgpu-diag-b8908e7.iso` is mounted in the second virtual optical drive, E: during this session. Probe scripts and helper commands are ignored artifacts, not production code. The corrected probe selects the exact preview path, not a recursive first executable (the media also contains a legacy executable).

Guest input uses QMP only. Absolute coordinates use 0..32767; a center move was verified through the guest Cursor.Position as (640,400). The ignored `guest_pointer.py` is fixed to the observed 1280×800 display: update its mapping if resolution changes. Prefer explicit foreground activation by PID/title, then a fresh in-client pointer move and a visible Recording/reset-counter check before F6. Send commands without a trailing newline and use one explicit Return; inspect guest readiness at focus transitions. Do not assume rapid batched input or a producer counter proves target receipt.

## Next work

1. Continue Windows QA using the instrumented target. Always focus the capture window, move the guest cursor into its client, and verify the visible position and event counters before the run. A matching zero-event failure was reproduced when egui had no pointer position; the exact state of the original sessions was not recorded. Keep their records and continue the all-button and lifecycle matrix.
2. Continue Windows QA: other finite-count/interval cases, manual preparation/cancellation, shortcuts/layout/conflicts, invalid-file preference recovery, DPI/NVDA, resource stability and native timing protocols. Count=1/10/1000 are evidenced for all three buttons and normal preferences Save/relaunch/discard passes. Other matrix entries and performance remain open. Use this VM for functional tests and retain physical performance limits separately.
3. Native GNOME/KDE permission/input/accessibility and the remaining per-backend performance gates remain open. Preserve budgets and the C++ reference until the full matrix is accepted.

## Git and publication

The user authorized this feature branch and a draft PR, but no merge/release. Draft creation remains blocked by the previously rejected GitHub CreatePullRequest permission; do not repeatedly retry it or request authorization already granted. The prepared body is `/tmp/nanoclick-rust-migration-pr.md` if still present. Avoid full hosted runs for documentation-only updates.

The unrelated untracked `.aws` entry remains outside scope: never read, modify, remove, stage or commit it. Stage explicit project paths only. Documentation following the VM session may be committed locally without a documentation-only CI push.
