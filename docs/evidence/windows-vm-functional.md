# Windows 11 VM functional preview

Date: 2026-10-26. Environment and isolation are recorded in [startup evidence](windows-vm-startup.md). This uses the user's Windows 11 lab through a separate NanoClick overlay, with no network, shared folders or physical USB. The VM is below the performance reference specification; these results do not accept hardware performance budgets.

## Artifact and startup

All four jobs pass in hosted run `40232606248`, packaging commit `8bde30f51400e6d4c6de00b894b74051f73d0b03`. Application renderer changes are source `23aba0d`; the subsequent change adds the exact upstream SPIR-V license. The downloaded ZIP, PE imports, notices and separate PDB pass verification on Linux. The ZIP is 6,253,702 bytes, below the compressed-size budget.

- Preview ZIP SHA-256: `05699ec25c60cd92ca6ab03aa265ebd2787545516a5970ffbd2b14379317fcc7`.
- Executable SHA-256: `6c36688780f892be6f64d88fb64b56b0f97996ab4001627220d7ec60900a4be0`.
- Passive target SHA-256: `ed2d17d5269d71c385e5e65451deef23d758dddf0705e680b5d4741d2a977fc0`.

The instrumented QA target used for the 2026-10-26 follow-up is from hosted run `40236185840`, commit `b8908e7b42b4b4a8f6fbd89c449dda3203ad883a`. Its Windows preview ZIP SHA-256 is `e774de3e1c670a88dd5f6971136a57b59a09d21771a97968df9c91dfbe5ea902`; QA executable SHA-256 is `95eef665b6b68a0acb386fa555ee8ee8f8727e24049eeb7b4c5348289bda6f99`.

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

Follow-up finite cases: Left, Right and Middle Count=1 at 100 ms each completed with exactly one matching pair. At Count=1000 and 10 ms, each button also completed with exactly 1000 captured down/up pairs and no held button or anomaly. The producer reported 1188 skipped periods for Left, 1185 for Right and 1265 for Middle in this VM; these runs are functional count/receipt evidence, not timing-budget evidence. The Left target showed 2024 cumulative button edges (24 preceding edges plus 2000 for that run). Count=10, interval=100 ms and Left were restored in the unsaved app UI afterward; the preference Save control was not used. Screenshots: `count1-final.png`, `count1-right-result.png`, `count1-middle-result.png`, `count1000-complete.png`, `count1000-right-result-confirmed.png`, `count1000-middle-result.png` and `final-default-config-allcounts.png` under `artifacts/vm/windows11-lab/`.

The Stop target remained armed after the Stop request and retained one pair. The close target also retained one pair through activation changes and after process exit. These are bounded GUI observations, not global physical-button-state proof or native Stop-latency measurements. The closing run has no terminal diagnostics record: UI diagnostics only record snapshots actually observed before the UI exits, as documented in the existing diagnostics contract.

Sessions 2/3 recorded zero pairs despite producer runs reporting ten. An instrumented reproduction identified a matching capture-target setup failure, not a NanoClick delivery failure: egui-winit drops window mouse-button events until it knows an in-window pointer position. The exact pointer state was not recorded for those original sessions, so they remain preserved and excluded. Session 4 remains a deliberate control containing one virtual right-button pair plus ten producer pairs, so its total of eleven is not a Count=10 acceptance. For each fresh capture target, bring it to the foreground, move the pointer inside its client, verify the target reports a known cursor position, then arm and start NanoClick with F6.

### Follow-up input-capture investigation

On 2026-10-25, producer runs 3 and 4 again reported `completed`, `sent=10`, `failures=0`; the passive capture GUI remained at zero. A temporary, guest-only `WH_MOUSE_LL` observer recorded the injected left-button down/up events at client coordinate (650,350). `WindowFromPoint` and `GetForegroundWindow` both resolved to capture PID 4136; the hook marked the input as injected. This showed that Windows saw the events while the capture target was foreground and under the pointer, but did not establish egui's pointer state.

The first attempts to click the capture title bar were invalid because the administrative PowerShell window covered the target. The source `egui-winit` 0.37.0 handler also confirmed that it converts `MouseInput` to egui `PointerButton` only when its `pointer_pos_in_points` is set. This condition was made visible in the passive QA target in commit `b8908e7`.

On 2026-10-26, the new target was launched with the pointer outside its client. Its UI showed “Cursor in client: no known position” and `0 moves · 0 button edges`. Moving the guest pointer into the focused client changed this to a known position `(366,243)` and recorded a cursor move. A direct QEMU-tablet click then appeared as one complete left-button pair. In a fresh capture session, NanoClick Count=10 produced exactly `10/10` left events and ten pairs, with no held button or anomalies. The target showed `1 moves · 22 button edges` since launch: two from the preceding control click and twenty from the ten producer pairs. Ending the session returned the target to Ready. Screenshots are `diag-new-target2.png`, `diag-pointer-moved.png`, `diag-positioned-qmp-click.png`, `diag-count10-position-known.png` and `diag-count10-final.png`. The downloaded QA executable hash is `95eef665b6b68a0acb386fa555ee8ee8f8727e24049eeb7b4c5348289bda6f99`, built from `b8908e7`.

The matching failure mode is attributed to the passive capture harness lacking a post-focus in-client pointer move; it is not evidence of failed `SendInput`. The exact cause of each original session cannot be retroactively confirmed because cursor-position telemetry was not recorded then. Continue native Windows acceptance for all buttons, lifecycle, preferences, scaling/accessibility and performance. The observer source and screenshots are local ignored VM artifacts; the diagnostic ISO was read-only in the guest.

## Evidence and remaining work

Original guest logs are `C:\NanoClickLab\NativeGraphics2\{help-result.json,startup-result.json,producer.jsonl,capture.jsonl}`. The low-level follow-up is under `C:\NanoClickLab\FocusRepro1\{producer.jsonl,capture.jsonl,trace-focus-20261025.csv}`; the instrumented capture run is under `C:\NanoClickLab\PointerDiag1\{capture.jsonl,mouse-trace.csv}`. Local ignored artifacts are under `artifacts/vm/windows11-lab/`: the original accepted-session screenshots, `trace-*.png`, and the five `diag-*.png` captures listed above. The original session table transcribes its displayed records; the follow-up capture was inspected from its live UI and guest JSONL output remains available for the next QA pass.

Still open: N=1/large-count coverage, manual preparation/cancellation, shortcut conflicts/held keys/layouts, preferences and error flows, native DPI/monitor transitions, NVDA, lifecycle/resource stability and full timing/CPU/RSS protocols. No Windows milestone or final migration acceptance is marked complete from this subset.
