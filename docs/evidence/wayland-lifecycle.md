# Wayland session preparation ordering

The RemoteDesktop adapter now establishes its `Session.Closed` subscription before calling SelectDevices or Start. Previously subscription happened after Start, leaving a window in which the desktop could revoke the newly active session before the monitor was ready.

The worker waits for a bounded subscription acknowledgement. Only then can it request pointer permission. If monitor creation/subscription fails, it attempts to close the allocated session and returns an error. If consent/preparation fails after monitoring begins, ordinary session cleanup stops and joins the monitor and closes the session; cleanup failures remain visible alongside the preparation error.

This ordering change does not request permission during startup or read-only capability probing. It does not enable unattended control of the user's desktop.

Validation here is source review and Linux compilation/Clippy. No consent, denial or revocation sequence has been exercised against GNOME/KDE native clients. A further open requirement is detecting portal **service-owner loss** during a long interval even if the session signal stream remains alive; the next failed input call is insufficient as the only detector. T04/T12/T15 remain open until these cases have native evidence.
