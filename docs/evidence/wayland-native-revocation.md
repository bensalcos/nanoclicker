# Native GNOME revocation, buttons and preference errors — 2026-10-29

## Artifact and environment

No product or receiver source changed in this continuation. The product is the exact `cc7bea099a679dcdef540585a4282dd805d52a67` Linux WGPU artifact, SHA-256 `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d`. The receiver includes the `d0a2874` hidden-input correction, SHA-256 `763eb0584c9b41958f033935e8061ff574aae5dd677dcd1ab34922dc68a737ca`.

Live versions were rechecked: Fedora 44, GNOME Shell/Mutter 50.5, xdg-desktop-portal 1.22.1 and RemoteDesktop v2. The headless native Wayland compositor has a 1280×800 virtual monitor, private D-Bus/runtime/config/data/cache, PipeWire and policy-only WirePlumber. Ignored artifacts are under `artifacts/wayland-revoke-20261029/`; `provenance.json` identifies the binaries. No host desktop input, physical device, Windows VM action, network integration or hosted CI was used. This accepts the stated private Fedora cases, not the reference Ubuntu/KDE matrix or physical performance/accessibility.

## Independent controller and rejected setup attempts

The prior auxiliary RemoteDesktop controller was removed by GNOME's sharing-stop action along with the product session. This test instead loads a QA-only extension in the private compositor. It owns a service on the private bus and injects events through Clutter virtual pointer/keyboard devices. It is outside RemoteDesktop sessions, survives their revocation, and is neither a production dependency nor a modification to the user's shell configuration. Four distinct clicks generated exactly four GTK button activations (`gtk-events.jsonl`).

Clutter's virtual pointer accepts logical button 1 for Left; the portal and raw Wayland traces use Linux button 272. Initial fixture calls incorrectly passed 272 to Clutter and generated invalid-button warnings; those calls did not pass the baseline and are rejected. Keyboard input uses the tested evdev codes. A separate focus race sent Space before the receiver entered keyboard focus; the initial key-down then appeared only as an already-held key in `wl_keyboard.enter`. The fixture now waits after focus transitions and checks a fresh Recording frame before Start.

The first native sharing-stop attempt reached product terminal failure and two raw pairs, but the receiver was unarmed. Its zero-event session 1 is rejected as matching-count evidence. Another attempt has matching run 3/session 2 counts but its wrapper failed on an existing screenshot directory; retain it as supporting exploration, not one of the complete receipts below. Screenshot filenames are now unique, phase details are saved before capture, and the expected new receiver session must be present before its result is accepted.

## Deliberate revocation of the active native session

After granting the native RemoteDesktop dialog, the app is set to Until stopped at a 1000 ms period. A fresh receiver session is armed, manual Start runs its countdown, and two raw press/release pairs are received. Immediately after the second release, the independent controller moves to GNOME's orange sharing-stop control and presses it. The D-Bus trace distinguishes the QA controller call from all product button calls: the controller's Button press precedes the real portal Session.Closed signal by about 13–16 ms. The next product period is almost a second away, so this is not an automatic product click on the control.

| Receipt | Product run / receiver session | Product attempts / sent | Raw and receiver press / release pairs | Anomalies / held | Observed terminal after controller press |
|---|---|---|---|---|---:|
| `revocation-4.json` | 4 / 3 | 2 / 2 | 2 / 2 | 0 / false | 43.775 ms |
| `revocation-5.json` | 5 / 4 | 2 / 2 | 2 / 2 | 0 / false | 42.449 ms |

Both terminal reasons are `failed`, with zero send failures and skips. The receiver reports balanced input, no other buttons, overlap or overflow. Its complete post-closure recording contains no further product edges. The observed terminal measurement includes the controller's deliberate 40 ms button hold and polling; it is not a p99 Stop budget or a measurement of monitor latency alone.

The portal owner remains `:1.35` before and after both actions: this is native Session.Closed rather than frontend process loss. The independent controller remains usable. Subsequent explicit manual Starts in the same product process obtain new sessions, deliver input and can be revoked again. No new consent dialog was displayed for these same-process retries. GNOME retained permission to restore; this test does not claim that persistent stored permission was erased. There was no automatic restart, and no app relaunch was needed for recovery.

Raw evidence: `trace.log`, `target.log`, `input-actions.jsonl`, `phase-{4,5}.json`, `runs.jsonl`, `capture.jsonl`, the two revocation receipts and `revoked*.png`. The fixture never issues a direct session Stop or kills the portal in these accepted trials.

## Three-button finite delivery after revocation

All three buttons were selected in the actual form, with Count=10 and a 10 ms period, using manual Start and the same product process after revocation. Every run completed with attempts/sent=10, failures/skips=0. Raw `wl_pointer.button` events agree with the receiver's per-button counts.

| Button | Product run / receiver session | Native button | Complete pairs |
|---|---|---:|---:|
| Left | 6 / 5 | 272 | 10 |
| Right | 7 / 6 | 273 | 10 |
| Middle | 8 / 7 | 274 | 10 |

Every session is balanced; all other button counts, anomalies, held flags, overlap and overflow are zero. See `finite-{left,right,middle}.json` and corresponding screenshots. These short functional trials do not accept sustained requested-rate budgets.

## Full-feature native preference I/O and recovery

The same full-feature application was tested through its Wayland GUI with an isolated preferences directory. Directory mode was restored after each fault injection.

- **Save denied:** mode 0555 prevented atomic temporary-file creation. The UI showed Permission denied and retained Unsaved changes. Original and failed-Save SHA-256 both equal `76ae96444e23ebeffe73981152c688144720b48db5b525537da52a5ab1bf1388`. Restoring mode 0755 and clicking Save persisted the chosen Middle/10 ms/Count=10 settings. The settled UI says Preferences saved and clears the dirty indicator. `save-error.json`, `save-denied.png` and `save-retry-settled.png` are authoritative; the first retry screenshot caught the preceding frame and is not final UI evidence.
- **Backup denied:** after a fresh launch with deliberately malformed private JSON, the app offered safe defaults and recovery. Denying directory writes made recovery show Permission denied while retaining the recovery button. The source stayed byte-exact at SHA-256 `d815c47b113a09a15fb982cd0b8cea19be2f6b6f22fd37171a86914acda5dfc9`; no backup was created. See `backup-denied.json`, `corrupt-initial.png` and `backup-denied.png`.
- **Recovery retry:** after restoring directory permissions, clicking recovery created exactly one unique backup with the original corrupt bytes and saved valid schema v2 defaults: Left, 100 ms, Count value 10, Until stopped, System and F6. `recovered.json` records the matching source/backup hashes. The success UI names the backup. A fresh process loaded those defaults, showed Ready with zero sent pairs and emitted no terminal run without Start (`recovered-relaunch.png`, empty `recovered-runs.jsonl`).

These cases cover this Fedora full-feature native GUI. Future-schema/parent-obstruction cases now have a separate [native Fedora follow-up](wayland-protected-preferences.md). Reference distributions, other path variants and the remaining reader/DPI matrix retain their separate scope. Earlier Windows evidence is not recertified by these Linux checks.

## Final state and remaining boundary

`cleanup.json` confirms no processes with the private runtime remain, the user's GNOME compositor PID 4780 remains running, and the private original preference bytes and directory permissions are restored. The QA extension exists only in the ignored private lab. Historical diagnostic files and the generated corrupt-file backup remain local evidence.

This continuation closes deliberate active-session revocation, same-process recovery, three-button Count=10 delivery and the stated preference errors/recovery on private Fedora GNOME. Reference Ubuntu GNOME and KDE, persistent-permission reset, broader native lifecycle/layouts, other native path/error variants, readers, physical monitor transitions and full resource/timing budgets remain open. Keep the C++ reference and preview label; general software-RSS optimization remains paused. No source build, broad test rerun, Windows validation, push, merge or release was performed for this evidence-only continuation.
