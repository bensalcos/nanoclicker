# Windows 11 VM functional preview

Date: 2026-10-25. Environment and isolation are recorded in [startup evidence](windows-vm-startup.md). This uses the user's Windows 11 lab through a separate NanoClick overlay, with no network, shared folders or physical USB. The VM is below the performance reference specification; these results do not accept hardware performance budgets.

## Artifact and startup

All four jobs pass in hosted run `40232606248`, packaging commit `8bde30f51400e6d4c6de00b894b74051f73d0b03`. Application renderer changes are source `23aba0d`; the subsequent change adds the exact upstream SPIR-V license. The downloaded ZIP, PE imports, notices and separate PDB pass verification on Linux. The ZIP is 6,253,702 bytes, below the compressed-size budget.

- Preview ZIP SHA-256: `05699ec25c60cd92ca6ab03aa265ebd2787545516a5970ffbd2b14379317fcc7`.
- Executable SHA-256: `6c36688780f892be6f64d88fb64b56b0f97996ab4001627220d7ec60900a4be0`.
- Passive target SHA-256: `ed2d17d5269d71c385e5e65451deef23d758dddf0705e680b5d4741d2a977fc0`.

The guest probe checks the copied app hash and packaging commit, and the target's matching commit/hash. The new executable passes `--help` without installing a VC++ redistributable. Both WGPU windows open on the existing Basic Display Adapter. The target reports native backend `windows`, scale 1.00. NanoClick displays English controls, registers F6 and remains idle on startup. No physical-GPU backend attestation is inferred from the WGPU build.

The initial QA probe accidentally selected the legacy executable from the comparison-media subdirectory and failed before launching Rust. The corrected probe uses an exact preview path and a fresh `C:\NanoClickLab\NativeGraphics2` directory. That probe error is not an application failure.

## Observed sessions

The app uses Count=10 at 100 ms for the finite cases. F6 is pressed with the capture window focused and the pointer in its client area. The JSONL summary confirms `balanced=true`, `unexpected_buttons=false` and `overflowed=false` for the sessions below; visible per-button held/anomaly counters are false/zero.

| Capture session | Case | Received complete pairs | Producer terminal evidence |
|---|---|---|---|
| 1 | Left, Count=10 | Left=10, all others=0 | Run 1 completed, sent=10, failures=0 |
| 5, 6 | Controlled Right repeats, Count=10 | Right=10 each, all others=0 | Runs 5/6 completed, sent=10 each, failures=0 |
| 7 | Middle, Count=10 | Middle=10, all others=0 | Run 7 completed, sent=10, failures=0 |
| 8 | Infinite Middle, interval 999999 ms, F6 Start then F6 Stop | Middle=1, all others=0 | Run 8 stopped, sent=1, failures=0; UI returns to Ready |
| 9 | Restart after Stop, same long interval, close while Running | Middle=1, all others=0 | Running/1 pair visible before Alt+F4; subsequent process query returns `NanoClickProcesses=0` |

The Stop target remained armed after the Stop request and retained one pair. The close target also retained one pair through activation changes and after process exit. These are bounded GUI observations, not global physical-button-state proof or native Stop-latency measurements. The closing run has no terminal diagnostics record: UI diagnostics only record snapshots actually observed before the UI exits, as documented in the existing diagnostics contract.

Sessions 2/3 recorded zero pairs despite producer runs reporting ten; they are retained as unresolved probe discrepancies and are **not accepted**. Session 4 is a deliberate control containing one virtual right-button pair plus ten producer pairs, so its total of eleven is not a Count=10 acceptance. Controlled repeats use explicit foreground activation, a fresh move within the capture client area, a visible Recording/zero-counter check, then F6. The valid repeats do not explain or erase the earlier mismatch. Reproduce and isolate it before claiming robust Windows delivery across focus transitions.

### Follow-up low-level investigation

On 2026-10-25, producer runs 3 and 4 again reported `completed`, `sent=10`, `failures=0`; the passive capture GUI remained at zero. A temporary, guest-only `WH_MOUSE_LL` observer recorded the injected left-button down/up events at client coordinate (650,350). For those rows, `WindowFromPoint` and `GetForegroundWindow` both resolved to capture PID 4136; the low-level hook marked the input as injected. This proves the events reached the Windows low-level mouse hook while the capture window was foreground and under the pointer. It does not prove that Winit delivered `WM_LBUTTONDOWN`/`WM_LBUTTONUP` or that egui accepted them. A separate guest-tablet click control, performed with the capture window visibly in front, also left the session counters at zero.

The first attempts to click the capture title bar were invalid: the administrative PowerShell window covered the target, so those coordinates hit PowerShell. Minimize that console before targeting the capture window. The later traced producer events do independently show the capture PID as both foreground and window-under-pointer, so the discrepancy remains unresolved after correcting the focus procedure. `egui-winit` only emits egui pointer-button events when it has a known in-window pointer position; confirm the post-activation `CursorMoved` path and the native mouse messages before drawing a product conclusion. The observer source and screenshots are local ignored VM artifacts; the temporary trace ISO was read-only in the guest.

## Evidence and remaining work

Original guest logs are `C:\NanoClickLab\NativeGraphics2\{help-result.json,startup-result.json,producer.jsonl,capture.jsonl}`. Follow-up run data is under `C:\NanoClickLab\FocusRepro1\{producer.jsonl,capture.jsonl,trace-focus-20261025.csv}`. Local ignored artifacts are under `artifacts/vm/windows11-lab/`: `windows-left-ten.png`, `windows-right-clean-final.png`, `windows-middle-clean-final.png`, `windows-capture-all-sessions.png`, `windows-producer-all-runs.png`, `windows-long-wait-config.png`, `windows-long-wait-stopped-ui.png`, `windows-active-close-before.png`, `windows-active-close-process.png`, `windows-close-target-final.png` and the `trace-*.png` captures. These were inspected directly; the session table transcribes its displayed records rather than claiming an exported raw log.

Still open: the zero-event discrepancies, N=1/large-count coverage, manual preparation/cancellation, shortcut conflicts/held keys/layouts, preferences and error flows, native DPI/monitor transitions, NVDA, lifecycle/resource stability and full timing/CPU/RSS protocols. No Windows milestone or final migration acceptance is marked complete from this subset.
