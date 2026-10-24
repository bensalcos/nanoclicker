# Wayland session preparation ordering

The RemoteDesktop adapter now establishes its `Session.Closed` and portal service-owner subscriptions before calling SelectDevices or Start. Previously subscription happened after Start, leaving a window in which the desktop could revoke the newly active session before the monitor was ready. Portal owner disappearance/replacement also invalidates the session and now wakes cancellation without waiting for the next scheduled click.

The worker waits for a bounded subscription acknowledgement. Only then can it request pointer permission. If monitor creation/subscription fails, it attempts to close the allocated session and returns an error. If consent/preparation fails after monitoring begins, ordinary session cleanup stops and joins the monitor and closes the session; cleanup failures remain visible alongside the preparation error.

This ordering change does not request permission during startup or read-only capability probing. It does not enable unattended control of the user's desktop.

Validation here is source review and Linux compilation/Clippy. No consent, denial or revocation sequence has been exercised against GNOME/KDE native clients. Service-owner loss is observed through zbus `Proxy::receive_owner_changed`, re-exported by ashpd, without a new dependency or polling. Both owner loss and replacement terminate the run. This still needs an isolated D-Bus lifecycle test and native desktop validation; compilation alone is insufficient. T04/T12/T15 remain open until these cases have native evidence.

Primary API contract: published local source for zbus 5.19.0, `Proxy::receive_owner_changed`, describes notifications on owner changes/disconnection. The listener is installed before permission preparation, and the monitor races it against Session.Closed and explicit shutdown.
