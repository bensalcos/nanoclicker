# Isolated portal lifecycle contract tests

Date: 2026-10-24. Test source: `0a4837e0937b3805603f5191bc017bc48b579f24`. Reproduce with:

```sh
scripts/verify-portal.sh
```

The fixture implements RemoteDesktop, Request response signals and Session.Close over a real private D-Bus transport. It calls the production `WaylandInput::open` and engine; pointer notifications only append to bounded mock state. It has no compositor or input-device connection. This is wire-level contract evidence, not a GNOME/KDE portal implementation or native desktop input test.

## Isolation

The wrapper creates a private directory/socket and explicit session-bus configuration with no includes or service activation directories, permits only the current UID, and removes DISPLAY/WAYLAND_DISPLAY and supplies the private directory as XDG_RUNTIME_DIR, preventing default Wayland-socket fallback. The example additionally requires its isolation marker and rejects any bus advertising activatable services other than D-Bus itself before it claims a portal name. Initial mock ownership uses DoNotQueue without ReplaceExisting. An existing service therefore causes failure, rather than being replaced.

The first experiment used the default dbus-run-session configuration. After simulated owner loss, cleanup auto-activated installed portal/helper services and the next fixture could not acquire its name. That run is not accepted evidence. The explicit activation-disabled configuration corrects this; the final eight-scenario run contains no service-activation messages. A negative guard test confirms the default session-bus configuration is rejected before portal ownership. Known PIDs recorded in the first run were checked afterward and were no longer present.

## Verified scenarios

| Scenario | Required observation |
|---|---|
| Normal Stop | One ordered down/up notification pair; retained Stopped state; one Session.Close |
| Selection denied | Faulted/Failed retained; no pointer notifications or Start; one Session.Close |
| Stop during preparation | Stop while Start has no response; no pointer notifications; retained Stopped state; one Session.Close |
| Release failure | Failed first release is retried during cleanup; sent remains zero, failures is one; session closes |
| Closed during SelectDevices | Already-installed listener invalidates preparation; no pointer notifications; session closes |
| Revocation during long wait | Closed wakes the 999999 ms wait, retains failure, and closes the session |
| Service-owner loss | Name release wakes the long wait; no additional pair; cleanup failure remains visible |
| Service-owner replacement | Replacement without a gap wakes the long wait; no additional pair; cleanup failure remains visible |

All eight pass locally; [aggregate log](portal-lifecycle.txt). The observer timeout is four seconds, a test safety bound rather than a latency budget. No native timing acceptance is inferred.

A negative control temporarily disabled the production monitor's failure notification. The closed-during-selection case then timed out in Running after sending a pair, as expected. The original production source was restored byte-for-byte and the positive suite passed again. No production adapter changes were needed for these tests. The fixture explicitly uses the already-present zbus 5.19.0 as a Linux development dependency; no new package version or shipping dependency was introduced.

Linux CI now runs the feature-gated example and its Clippy check, retaining portal.log with the existing diagnostic artifact. Hosted run 40094952287 at 0a4837e passes all four jobs, including all eight portal scenarios on Ubuntu 24.04. The diagnostic artifact was downloaded and its portal.log inspected; it contains eight PASS lines and no service-activation messages. Real GNOME/KDE consent, denial/revocation, native Wayland delivery, shortcut behavior, D-Bus failure/timeout variants and performance remain open.
