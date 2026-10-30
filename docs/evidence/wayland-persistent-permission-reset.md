# Native persistent permission reset — 2026-10-30

## Artifact and boundary

The tested product remains the unchanged `cc7bea099a679dcdef540585a4282dd805d52a67` Linux WGPU executable, SHA-256 `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d`. The receiver contains the `d0a2874` correction, SHA-256 `763eb0584c9b41958f033935e8061ff574aae5dd677dcd1ab34922dc68a737ca`. Every accepted case ran in the same product process, PID 3483209, with portal frontend owner `:1.37` unchanged.

Live packages: Fedora 44, GNOME Shell/Mutter 50.5, xdg-desktop-portal 1.22.1, GNOME portal backend 50.0 and Control Center 50.4. The native Wayland compositor uses a 1280×800 virtual monitor and private D-Bus/runtime/config/data/cache, PipeWire and policy-only WirePlumber. The independent private Clutter controller from [the earlier native trials](wayland-native-revocation.md) handles test input. No host desktop input or physical device was used.

Artifacts are local and ignored under `artifacts/wayland-persist-20261030/`. No product/receiver source changed, no build or broad test/CI run was needed, and no Windows VM, push, merge or release action was performed. Coverage is the private Fedora native service/API and its real consent dialog. This does not claim a reset through GNOME Settings or reference Ubuntu/KDE acceptance.

## What is reset

Closing an active sharing session and deleting its remembered authorization are separate cases. The prior sharing-stop trials left the remembered authorization usable. This continuation exercises the actual private `org.freedesktop.impl.portal.PermissionStore` service: table `remote-desktop`, exact app ID `io.github.bensalcos.NanoClick`, method `DeletePermission`.

The [installed-version frontend implementation](https://github.com/flatpak/xdg-desktop-portal/blob/1.22.1/src/xdp-session-persistence.c) looks up restoration data only when the requested app has a permission entry for that token. [RemoteDesktop](https://github.com/flatpak/xdg-desktop-portal/blob/1.22.1/src/remote-desktop.c) delegates restoration to that helper. Removing the app's permission therefore leaves the old opaque record without authorization for this app; the test confirms behavior through the live portal rather than accepting source inspection alone.

Before deletion, the fixture resolves the private bus explicitly, verifies its isolated runtime, and requires exactly one authorized NanoClick entry. That entry must contain only NanoClick=`yes`. It deletes only that app permission and reads back the service: no entry authorizes NanoClick afterward. Existing empty records are left alone during the test. `persistent-reset.json` records before/after metadata using SHA-256 hashes of token keys; raw restoration tokens and opaque data are not printed in receipts. The product's preference file remains unchanged and contains no saved restore token.

## Accepted native sequence

An initial warm-up run established permission through the real dialog. The finite test plan is Left, Count=2, requested period 10 ms, manual Start. Receiver Recording state, raw Wayland edges and terminal run identity are checked separately.

| Case | Product run / receiver session | Product terminal / sent | Raw and receiver pairs |
|---|---|---|---:|
| Restored permission before reset | 5 / 3 | completed / 2 | 2 |
| Cancel renewed consent after permission deletion | 6 / 4 | failed / 0 | 0 |
| Explicitly grant the next consent request | 7 / 5 | completed / 2 | 2 |
| Subsequent Start reuses the new authorization | 8 / 6 | completed / 2 | 2 |

Run 5 completed without another consent dialog. After deleting the remembered permission, the same product process displayed the real RemoteDesktop dialog again (`reset-deny-dialog.png`). Before Cancel, `pending-reset-deny.json` confirms zero product button requests, zero raw receiver edges and no new terminal receipt. Native Cancel then produced run 6 with attempts/sent/failures/skips all zero and a matching empty, balanced receiver session.

The next explicit Start again displayed consent (`reset-grant-dialog.png`). `pending-reset-grant.json` confirms no product button request or raw edge before consent. Enabling remote interaction and choosing Share allowed run 7 to complete with two exact pairs. Run 8 subsequently restored the new permission without another dialog and delivered two exact pairs. The actual store then contained one authorized NanoClick entry (`permission-restored.json`).

All accepted receiver reports are balanced, with no other buttons, held state, anomalies, overlap or overflow. All completed producer runs have attempts/sent=2, failures/skips=0. Portal owner identity is unchanged before and after every accepted run. These facts distinguish remembered-permission deletion from frontend loss, app restart and a mock denial. No automatic restart followed deletion, and no product button request occurred while renewed consent awaited a decision.

Authoritative receipts: `result-restored-baseline.json`, `result-reset-deny.json`, `result-reset-grant.json`, `result-after-restored-grant.json`, the two pending-consent records, `persistent-reset.json`, `permission-restored.json`, `runs.jsonl`, `capture.jsonl`, `trace.log` and `target.log`. The trace filters only button requests and session closure, avoiding complete portal options/responses.

## Rejected setup attempt and fixture correction

The first deletion succeeded, but its automation selected the wrong window while arming and switched away before verifying consent. `armed-reconsent.png` shows the app rather than an armed receiver; `reconsent-dialog.png` does not show a dialog. The request eventually timed out as run 3 with zero attempts/sent. That attempt is not accepted as a renewed-consent dialog or matching receiver trial. Its early zero-input assertion was renamed `awaiting-consent.rejected-unseen-dialog.json`, and the first deletion receipt is retained as `persistent-reset.exploratory.json`.

The click used to raise the product was too close to the receiver's frame border. The corrected fixture uses an unambiguous product point, waits for focus, verifies Recording, and keeps the product in front while checking consent. A separate explicit regrant restored a working state; run 5 then established the successful restoration baseline before a fresh deletion and the complete accepted sequence above. Token-key and app-permission state were re-read before that deletion. Public-source requests that returned 404 were rejected and replaced with the valid versioned sources linked above.

## Cleanup and remaining matrix

After the final successful restoration, cleanup removed only the known private QA token records, returning the private `remote-desktop` table to its initially empty state (`permission-cleanup.json`). `cleanup.json` confirms no process with the private runtime remains, the user's GNOME compositor PID 4780 remains running, and original private preferences remain byte-exact at SHA-256 `cf5558b9836877b51bd1aff515c70a7c912e09d73d8cd157a165684708a92c5a`.

Persistent authorization removal through the actual service API, renewed native consent, zero-input cancellation and same-process recovery are accepted for this private Fedora setup. Reset through a desktop Settings UI was not exercised. Reference Ubuntu GNOME/KDE, other native lifecycle/path/layout variants, readers, physical monitor/DPI transitions and full timing/resource budgets remain separate. Keep the C++ reference and preview label; general software-RSS optimization remains paused.
