# Wayland session preparation ordering

The RemoteDesktop adapter now establishes its `Session.Closed` and portal service-owner subscriptions before calling SelectDevices or Start. Previously subscription happened after Start, leaving a window in which the desktop could revoke the newly active session before the monitor was ready. Portal owner disappearance/replacement also invalidates the session and now wakes cancellation without waiting for the next scheduled click.

The worker waits for a bounded subscription acknowledgement. Only then can it request pointer permission. If monitor creation/subscription fails, it attempts to close the allocated session and returns an error. If consent/preparation fails after monitoring begins, ordinary session cleanup stops and joins the monitor and closes the session; cleanup failures remain visible alongside the preparation error.

This ordering change does not request permission during startup or read-only capability probing. It does not enable unattended control of the user's desktop.

Initial validation was source review and Linux compilation/Clippy; [eight isolated wire-level contract scenarios](portal-contract-tests.md) now also pass locally. No consent, denial or revocation sequence has been exercised against GNOME/KDE native clients. Service-owner loss is observed through zbus `Proxy::receive_owner_changed`, re-exported by ashpd, without a new dependency or polling. Both owner loss and replacement terminate the run. The isolated test uses a mock on an activation-disabled private bus; native desktop validation remains necessary. T04/T12/T15 remain open until these cases have native evidence.

Primary API contract: published local source for zbus 5.19.0, `Proxy::receive_owner_changed`, describes notifications on owner changes/disconnection. The listener is installed before permission preparation, and the monitor races it against Session.Closed and explicit shutdown.


## Global shortcut service restarts

The shortcut listener now subscribes to portal owner changes before BindShortcuts. It races both Session.Closed and owner changes against the binding request and normal activation/deactivation listening. Losing or replacing the service reports Unavailable promptly instead of leaving the UI with a stale registered shortcut. Cleanup remains bounded to one second; the listener remains event driven.

A new private-bus regression registered F6, released the service name and waited for Unavailable. Before this fix it timed out after four seconds. The repaired production listener passes five additional wire scenarios: owner loss, owner replacement, Closed after registration, Closed while BindShortcuts has no response, and owner loss while binding has no response. The two early failures must report Unavailable without first reporting Registered. The Closed cases also verify exactly one session-close attempt. Together with the eight RemoteDesktop scenarios, `scripts/verify-portal.sh` now passes thirteen scenarios locally. Workspace tests (42) and portal-feature Clippy pass. Hosted execution of the five new cases remains pending.

The mock advertises GlobalShortcuts v1 and returns the portal's `a(sa{sv})` shortcut dictionary wire type. It shares the activation-disabled private bus guard with RemoteDesktop and never connects to a compositor. These regressions do not establish real GNOME/KDE shortcut assignment or activation acceptance.
