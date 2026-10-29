# Wayland capture replay correction — 2026-10-29

## Scope and defect

The product remains `cc7bea099a679dcdef540585a4282dd805d52a67`, SHA-256 `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d`. This change corrects the passive QA receiver only. The corrected Linux WGPU receiver SHA-256 is `763eb0584c9b41958f033935e8061ff574aae5dd677dcd1ab34922dc68a737ca`. Native tests use the same isolated Fedora 44 GNOME/Mutter 50.5 setup described in [the registration evidence](wayland-native-registration.md), with a private session bus, PipeWire, policy-only WirePlumber and 1280×800 virtual monitor. No host input or Windows VM state was changed.

The receiver previously counted `input.raw.events` in `eframe::App::logic`. In the installed eframe 0.37.0, logic can run for an occluded/minimized window without an egui pass. The [App contract](https://github.com/emilk/egui/blob/0.37.0/crates/eframe/src/epi.rs) documents that events then belong to the last shown frame; [Context::run_logic](https://github.com/emilk/egui/blob/0.37.0/crates/egui/src/context.rs) retains those events while refreshing viewport information. Reading them again can duplicate a release or toggle recording repeatedly. Both behaviors were observed during the investigation. A forced multi-pass UI test did not duplicate input; that alternative hypothesis was rejected.

A headless regression first failed with four counted edges instead of two after hidden logic reused the visible frame. The final pointer regression places Press and Release in separate frames, then runs hidden logic twice: it must retain exactly one balanced pair. A second regression starts recording with Space and confirms three hidden logic calls do not toggle it again. Input consumption now runs in `App::ui`, which receives fresh input; hidden logic handles fresh viewport close requests only. The session counter still records unmatched releases and held buttons as anomalies. No filtering or acceptance thresholds were weakened.

The historical nine-press/eleven-release result remains preserved. It has no raw Wayland trace and cannot be repaired or attributed to the product retroactively. The receiver defect invalidates that count as authoritative evidence of surplus native releases. New traced trials below supersede it for private owner-loss balance only. Earlier positive receiver counts remain historical and are not silently recertified.

## Traced native owner-loss trials

Ignored artifacts: `artifacts/wayland-release-20261029/`. `buttons.log` captures only D-Bus `NotifyPointerButton` calls; `target-fixed.log` captures the receiver's actual `wl_pointer.button` messages using `WAYLAND_DEBUG=client`. These do not depend on the receiver's egui counter. The controller's manual Start click is outside the receiver and appears separately on the Mutter interface.

After the already granted app starts a 10 ms Until stopped run, the fixture waits for at least eight delivered releases. It resolves the portal frontend PID on the private bus, verifies its private-runtime environment, and sends SIGTERM only to that process. It then waits for the new terminal run and expected receiver session ID. A private screenshot stream triggers a fresh painted frame before reading the recording; a stale or unflushed session is rejected.

| Receipt | Product run / receiver session | Sent pairs | Forwarded portal implementation / raw Wayland / receiver pairs | Receiver anomalies / held | Observed terminal after SIGTERM |
|---|---|---:|---:|---|---:|
| `probe-2.json` | 4 / 2 | 33 | 33 / 33 / 33 | 0 / false | 86.230 ms |
| `probe-3.json` | 5 / 3 | 29 | 29 / 29 / 29 | 0 / false | 42.881 ms |

Every raw press has a corresponding raw release; every receiver report is balanced, with no other button, overlap or overflow. Run 4 attempted a 34th pair while the frontend disappeared: the frontend trace contains 34 press/release requests, but only 33 reached its implementation, the raw receiver and the completed-pair counter. The product records one send failure. Run 5 has 29 attempts/sent pairs and zero send failures; its owner monitor still reports terminal failure. Teardown fault handling and receiver delivery agree in both cases. These observed durations are not p99 Stop latency or reference-hardware performance acceptance.

The first corrected-receiver trial also received 36 balanced pairs, but its script timed out before the receiver log flushed; it is exploratory rather than one of the two complete receipts above. The pre-fix traced trial received 1339 raw and counted pairs, illustrating that replay is conditional rather than inevitable.

## Native sharing-stop exploration remains unaccepted

At a requested 1000 ms period, moving toward GNOME's orange sharing-stop indicator allowed the next product click to activate it before the controller's intended click. GNOME emitted the real portal Session.Closed signal, removed both product and auxiliary control sessions, and the product ended failed with seven sent pairs. Six pairs reached the target before the pointer left it; receiver session 4 records six balanced pairs and no held state. The subsequent controller call failed because its Mutter session had been removed.

This proves a real native session closure, but not the intended independently injected user action. The first new virtual keyboard also entered with its current key already held, so its initial Space did not produce a fresh key-down event. A replacement controller and subsequent attempted manual restart were not sufficient to complete a second, independently attributed revocation receipt. The zero-event recording is not acceptance. **User revocation, reliable control recovery and reauthorization after that action remain pending.** `closed.log`, `revoke-menu.png`, `revoke-app.png`, run 6 and receiver session 4 preserve the exploratory result. No further native pass is claimed.

## Verification and boundary

- `cargo test --workspace --all-targets --locked`: 80 tests pass, including 11 receiver tests.
- Linux workspace/all-target Clippy with `-D warnings`, `cargo fmt --all --check`, and the release receiver build pass. The existing linker optimization deprecation warning remains.
- `scripts/verify-input-capture.sh target/release/examples/input_capture` passes real private-Xvfb Left/Right/Middle sessions of 3/5/7 pairs and active close exporting exactly once.
- The change adds no dependency, production input request or logging path. Review checks input ordering, hidden close, unchanged anomaly detection and private/no-clobber output behavior.
- `cleanup.json` confirms no private-runtime processes remain. The user's compositor PID 4780 remains running. Only the private test lab was controlled.

Private Fedora portal-owner-loss balance now has two matching traced receipts. Reference Ubuntu GNOME/KDE, deliberate native user revocation, readers, physical monitor transitions and full timing/resource budgets remain open. General software-RSS optimization remains paused; retain the C++ reference and preview label. No new hosted CI, Windows GUI validation, push, merge or release is implied.
