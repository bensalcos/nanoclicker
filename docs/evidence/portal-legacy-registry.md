# Older portal frontend compatibility — native Ubuntu GNOME

## Reproduction and change

An Ubuntu 24.04 package environment reproduced a real compatibility regression in the `5f3e82c` production source: the application unconditionally required `org.freedesktop.host.portal.Registry`, which the installed xdg-desktop-portal 1.18.4 frontend does not advertise. NanoClick showed the missing-interface error and disabled Start even though RemoteDesktop v2 and pointer support were available.

The connection helper now retains per-peer explicit registration when available and continues to the frontend's normal capability/consent checks only when the Registry interface is absent. Other errors, including explicit AccessDenied, still propagate. The [Registry protocol](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.host.portal.Registry.html) describes explicit identity registration for unsandboxed peers; the locked ashpd 0.13.13 source supplies the exact typed errors used here. No identity, authorization or restoration success is assumed by the fallback, and no persistent-registration cache or new dependency was added. Missing RemoteDesktop and denied consent remain failures. This is a Linux-only module; the separate [Windows NVDA receipt](windows-nvda.md) remains scoped to `5f3e82c`.

The private wire fixture now includes a frontend without Registry that permits ordinary legacy requests. The new case failed with UnknownInterface before the fix; afterward probe, one balanced input pair and a shortcut session pass without registration. The existing modern registration/restoration tests still require registration, and explicit registration denial is checked for both RemoteDesktop and GlobalShortcuts, with no created session or input. All 83 workspace/all-target tests and Linux/Windows MSVC Clippy pass; platform Clippy with portal-tests enabled and the full isolated wire lifecycle suite also pass. The full-feature Fedora host release also builds (SHA-256 `eb361f9ce50c62a17d49a19838cdb132857ec157aef654b2929868b16bcf3452`), and `graphify update .` completed. The first local RED attempt failed at linking because a previous session's temporary runtime links were gone; after recreating links/metadata to installed libraries, the meaningful RED reproduced missing Registry. No host graphics packages or policies changed.

## Environment and scope

Real GNOME Shell 46.0/Mutter 46.2, xdg-desktop-portal 1.18.4, GNOME backend 46.2 and PipeWire 1.0.5 from Ubuntu 24.04 packages run inside a rootless Podman container with its own actual systemd, logind, system/session buses, configuration and headless 1280×800 Wayland display. Network is disabled for runtime tests, no host display/session bus/input device is mounted, and the private Clutter virtual controller addresses only its own session bus. The receiver is the passive `input_capture` example, also built in Ubuntu, with actual backend Wayland and scale 1.0.

This accepts the observed backend protocol/GUI cases in that isolated package environment. It does not accept a normal logged-in reference installation, physical drivers or timing/resource budgets. GDM, Settings and PolKit are absent and produce explicit fixture warnings. Failed preparation attempts without a system bus/logind and one overlong socket path are excluded. A real systemd/logind container and short internal paths resolved the startup failure without modifying GNOME or replacing portal services. The QA extension additionally activates/moves only NanoClick-titled test windows; it does not inject host input.

| Artifact | SHA-256 |
|---|---|
| Ubuntu WGPU application before fix (`5f3e82c`) | `b48737b320b30329cdccfba0174072623a8517673a5c4e57627ba51de72affd6` |
| Ubuntu WGPU application with connection correction | `8d23815bbb8ee138a7305187d528cdb5a36c6c71c700b582f3dc73d9d405841b` |
| Ubuntu passive receiver (`5f3e82c`) | `7c3647ade6affb5bef8f69a2ef3ffda0876cd4878c30440cc90771b3cd60fe3b` |
| Corrected `portal_connection.rs` | `53f1ab80b17ddcbc7b0af20f796bbce299e0834059f46e4e819a2be720348e69` |

The application source was exported from `5f3e82c` and only the corrected production connection file was copied into that export. The tests and evidence in this change supplement that exact production file. Raw logs, screenshots and package/image preparation are retained under ignored `artifacts/reference-desktops-20261030/`.

## Accepted native receipts

The same running process now starts with Start enabled. The receiver is explicitly foregrounded, armed with Space, moved into a known in-client position and observed in a fresh frame before each run. Product starts/consent controls are operated outside the receiver; after grant the pointer is placed back at (900,400), the receiver client (416,172), before the manual countdown ends.

| Trial | Producer | Independent receiver |
|---|---|---|
| Fresh native Cancel (run/session 2) | Failed preparation, attempts/sent/failures/skips all zero | Zero raw/egui button edges, zero pairs, balanced |
| Explicit native grant (run/session 3) | Completed, attempted=sent=2, failures/skips=0 | Left down/up/pairs=2/2/2, four egui edges, balanced, no other button/anomaly/held/overflow |
| Same-process restoration (run/session 4) | Completed, attempted=sent=2, failures/skips=0 | Same exact two-pair receipt, no new consent dialog |

The target's WAYLAND_DEBUG trace contains exactly eight ordered Left button events, four complete down/up pairs across the two accepted input sessions, matching both receiver reports. All target sessions retain scale 1.0. This is finite-delivery acceptance, not a latency/jitter claim. An earlier setup dialog exceeded the consent window (session 1 elapsed about 136 seconds); it is rejected as native Cancel evidence even though it recorded zero input. The accepted fresh Cancel session lasted about 11 seconds and has its own dialog/cancellation screenshots.

The frontend introspection confirms RemoteDesktop v2 and the absence of both Registry and GlobalShortcuts. Global-shortcut acceptance is therefore unavailable on this specific GNOME 46 backend; the manual path works. Modern supported-backend GlobalShortcuts behavior remains covered separately, and KDE validation is still pending.

Raw receipt integrity:

- `gnome-qa/accepted.json`: `0447831579ebe4934cd50fae48d90337eb3b6763ee94a574e58227bb2b260746`.
- `gnome-qa/raw-input-summary.json`: `b6b4997725b203a4229f208891faa95874755172a3e4e0a085292083a43bdda4`.
- `gnome-qa/capture.jsonl`: `bcc012084119d87988c035441e7a41652cd6a584c2e4172a62a7d107b1b70b76`.
- `gnome-qa/runs.jsonl`: `737240c6674f2add84b9321515473ec08971fd554ffd730311a5adef718e1ebf`.
- `gnome-qa/portal-interfaces.txt`: `7d95ef7e362e40b5b579085d2257cbf60e48350a58a60fe541775cbc0e15a0e3`.

Local review checked missing-interface-only fallback, continued propagation of registration denials, per-peer identity scope, unchanged consent/restoration/input paths and the positive/negative native receipts. No blocking finding remains for this scoped Linux compatibility fix.

The environment is retained temporarily for the next KDE/reader cases. A final cleanup receipt must be recorded before calling the complete desktop session closed; this document does not claim cleanup yet.
