# Native Wayland identity, restoration and delivery — 2026-10-29

## Scope and source

This follow-up fixes host portal registration over product base `b21fb749822a238e25b1f9e230e798dced67e52d`. The validated Linux WGPU executable SHA-256 is `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d`; the passive target is `09c855f576ebf492eba78faac956bcb0e008af5bb6b07556600d58760e67d487`. Local `artifacts/wayland-qa-20261029b/build-provenance.json` records the base commit and individual source hashes. The uncommitted performance-example change and earlier documentation edits are separate.

The real compositor is GNOME Shell/Mutter 50.5 on Fedora 44, headless Wayland with a 1280×800 virtual monitor, RemoteDesktop portal v2 and GlobalShortcuts v1. It uses its own D-Bus, runtime/config/data/cache directories, PipeWire and WirePlumber policy profile. No input was injected into the user's compositor, and no physical input device was assigned. A test-local copy of the supplied desktop entry matches `io.github.bensalcos.NanoClick`. This is native private Fedora evidence, not acceptance of the Ubuntu reference desktop, KDE, physical hardware timing or accessibility.

## Test-controller correction

The earlier control fixture created a Mutter ScreenCast stream associated with its RemoteDesktop session, but did not consume that stream. Its independent screenshot stream did not establish that the associated control stream was configured. [Mutter's implementation](https://github.com/GNOME/mutter/blob/50.5/src/backends/meta-remote-desktop-session.c) may acknowledge an absolute-motion request while dropping the motion if stream coordinate transformation is not ready.

The corrected controller consumes its own associated stream with a negotiated video format before accepting commands. Four clicks at distinct positions then generated exactly four GTK button activations in a separate native test window. This localizes the earlier unreliable positioning to the fixture; it does not establish that every historical key/button reached its intended target. The prior denial probe remains historical; the corrected fixture repeats the permission-cancel case below with an authoritative terminal receipt.

## Product defect and fix

Native shortcut configuration on the old source failed with `org.freedesktop.portal.Error.NotAllowed: An app id is required`. The Wayland window's app ID did not identify the process's D-Bus peer to the portal.

The [Registry contract](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html) associates an unsandboxed D-Bus peer with the application's desktop ID before portal calls and allows registration only once per peer. The new private connection helper creates a fresh session-bus connection and uses ashpd's host registration helper before constructing RemoteDesktop or GlobalShortcuts proxies on that same connection. Sandboxed identity detection remains ashpd's responsibility. Each new preparation/configuration uses a new registered peer, including after portal owner replacement; existing sessions retain their owner-loss monitoring, cancellation and bounded operations. Registration performs no input and bypasses no consent.

The mock now rejects CreateSession without prior peer registration, reproducing the native error on the old implementation. It also rejects duplicate registration and can deny registration entirely. The restored two-run case verifies two registrations on fresh peers; shortcut cases require one registration per peer; denied registration must fault before creating a session or sending input.

## Native receipts

All logs and screenshots below are local ignored artifacts under `artifacts/wayland-qa-20261029b/`.

- Before the identity fix, granting the real RemoteDesktop dialog produced ten sent pairs. The second manual Start reused permission without another dialog: `runs.jsonl` run 2 and `capture.jsonl` session 1 each report ten complete Left pairs, no failures, no skips, no other buttons, no held state, no anomalies or overflow. This validates the prior process-local restoration implementation in this environment.
- With the fix, GNOME displayed Add Keyboard Shortcuts and offered F6; Add resulted in `Registered · Press F6`. `fixed-shortcut-dialog.png` and `f6-registered.png` show the native flow.
- F6 with the passive target focused opened the real permission dialog. Cancel returned a terminal failed producer run with attempts/sent/failures/skips all zero (`runs-fixed.jsonl` run 1; `verified-cancel.png`). A new F6 request could then be granted. This corrected-fixture receipt supersedes uncertainty about the earlier cancel-versus-timeout probe.
- After granting, another outside-focus F6 run reused permission without another dialog. `runs-fixed.jsonl` run 3 and receiver session 2 match ten complete Left pairs, balanced, no extra-button events, overlap, held state, anomalies or overflow (`f6-delivery.png`).
- Closing and relaunching the app retained `portal_shortcut_configured=true` and automatically initiated binding. GNOME again offered F6 and required Add confirmation (`restarted.png`). After confirmation, F6 outside focus opened the input permission request (`restarted-f6-request.png`). This accepts automatic rebind intent and the observed assigned key; it does not promise dialog-free startup. RemoteDesktop restore tokens remain process-local, so a fresh app process requests input consent again.

## Exploratory requested-rate checks

The restarted fixed app used Count=1000 with the passive Wayland target focused/recording and an already granted process-local restore token. Every sample received exactly 1000 Left presses, releases and pairs; other buttons, held state, overlap, anomalies and overflow were zero. See `rate-results.json`, `rate.log`, `runs-restart.jsonl` and receiver sessions 3–5.

| Requested period | Shortcut to terminal | End-to-end pairs/s | Skipped periods | Producer failures |
|---|---:|---:|---:|---:|
| 10 ms | 10013.154 ms | 99.87 | 0 | 0 |
| 5 ms | 5021.204 ms | 199.16 | 0 | 0 |
| 1 ms | 1046.965 ms | 955.14 | 8 | 0 |

The duration includes preparation and diagnostic observation; it is not steady per-event receiver timing. These short single samples establish functional delivery at the requested settings in this private compositor, not a before/after speedup or the 60-second/reference-hardware budgets. The minimum interval does not guarantee sustained 1000 clicks/s.

## Historical owner-loss finding — superseded receiver counts

While a 10 ms Until stopped run was active, the test sent SIGTERM only to the portal frontend whose process environment matched the private runtime. The producer reached terminal failure in an observed 43.292 ms, reporting nine attempted/sent pairs and no send failures. This is actual private service-owner loss, not user permission revocation or p99 Stop acceptance.

Receiver session 6 records nine presses, eleven releases and nine complete pairs: no button remains held, but two surplus releases make the report unbalanced with two anomalies. **This historical count is superseded by the [capture replay correction and two raw-traced native receipts](wayland-capture-replay.md).** Attribution to the product, portal/compositor teardown or event translation is unresolved; the registration fix does not claim to fix this finding. Raw evidence is `ownerloss-terminal.png`, session 6 of `capture.jsonl`, producer run 5 of `runs-restart.jsonl` and corrected `owner-loss-result.json`.

The exploratory script's first immediate read incorrectly selected the previous saved receiver session 5 before session 6 had flushed. That stale result is explicitly rejected and retained only as `owner-loss-result.rejected-stale-receiver.json`. The script now waits for the expected next session ID; it must not substitute an older balanced receipt for the actual unbalanced failure session.

## Verification and remaining work

- `cargo fmt --all --check`, Linux workspace/all-target Clippy, Windows MSVC cross-target workspace/all-target Clippy and gated portal-example Clippy pass.
- `cargo test --workspace --locked` passes 69 tests; the private portal fixture passes 17 lifecycle/registration cases. The initial registration-required regression failed before the fix and passes after it.
- The full-feature release build passes, with the existing deprecated-linker-optimization warning. The private X11 fixture also passes all three buttons, held/conflicting shortcuts and mapping-change checks.
- `graphify update .` completed with 1570 nodes/2914 edges; the explicit `.aws` corpus exclusion remains. No hosted CI run, Windows GUI rerun, merge or release is inferred from these local checks.

Active-session revocation and same-process recovery now pass the [independent-controller native follow-up](wayland-native-revocation.md). Remaining: reference Ubuntu GNOME and KDE flows, persistent-permission reset, broader buttons/layouts, readers, physical multi-monitor/DPI and complete timing/resource acceptance. Keep the C++ reference. The general software-RSS optimization remains paused.
