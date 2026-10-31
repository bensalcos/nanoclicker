# Ubuntu GNOME and KDE package follow-up — 2026-10-31

## Scope and artifacts

This extends the [older-frontend correction](portal-legacy-registry.md) at `3a9a38b`. The same corrected Ubuntu WGPU binary (`8d23815bbb8ee138a7305187d528cdb5a36c6c71c700b582f3dc73d9d405841b`) and passive receiver (`7c3647ade6affb5bef8f69a2ef3ffda0876cd4878c30440cc90771b3cd60fe3b`) run in a rootless, network-disabled Ubuntu 24.04 Podman environment with actual systemd/logind, private session buses, configuration, display and virtual input. No host desktop, bus, input, GPU or physical device is passed through. Software rendering and this minimal package environment do not accept a clean reference installation, hardware compatibility or performance budgets.

Versions: GNOME Shell 46.0, Mutter 46.2, Orca 46.1, PipeWire 1.0.5, xdg-desktop-portal 1.18.4, GNOME backend 46.2, KWin and KDE portal backend 5.27.11. System Settings is 5.27.11; installing its required panel also installs plasma-desktop/data 5.27.12. Package download occurs in a separate setup container; the runtime remains offline. Raw files are under ignored `artifacts/reference-desktops-20261030/`.

## KDE native input and preferences

KWin runs a 1024×640 nested Wayland output inside the independent headless GNOME display. The accepted profile runs as unprivileged UID 1000 with new configuration and bus. The shipped KWin executable has a capability outside the rootless bounding set; a byte-identical lab copy without its capability xattr starts successfully. KWin uses its QPainter software backend; NanoClick and the receiver use WGPU. The initial plasma-workspace-only profile lacks shell data and cannot start Plasma Shell; after panel/data installation, no complete Plasma desktop is claimed.

Each session explicitly focuses and arms the receiver, confirms a fresh zero count and positions the pointer at client (233,128). A native manual Start is followed immediately by pointer movement back into the receiver before the countdown ends. Product clicks never operate the receiver controls. The independent KWin scripting API only activates/positions known NanoClick test windows; actual input comes from the private outer compositor's virtual controller and the real portal.

| Button | Producer | Independent receiver |
|---|---|---|
| Left | Completed, attempts=sent=2, failures=skips=0 | 2 down, 2 up, 2 pairs |
| Right | Completed, attempts=sent=2, failures=skips=0 | 2 down, 2 up, 2 pairs |
| Middle | Completed, attempts=sent=2, failures=skips=0 | 2 down, 2 up, 2 pairs |

Every receiver session has four egui button edges, no other button, anomaly, held state, overlap or overflow. The raw Wayland trace contains exactly twelve ordered edges: two complete pairs each of evdev buttons 272, 273 and 274. These accept finite manual delivery, not timing.

This KDE backend did not display an additional consent dialog for the pointer-only request, on either the initial root profile or the fresh unprivileged profile. No fresh denial/cancellation acceptance is inferred. An initial root trial failed because the test pointer remained over Stop and stopped after one request; that setup is excluded. The unprivileged trials correct the pointer position before input begins.

Native Save stores Middle and relaunch restores it. A controlled, unprivileged write denial (preferences directory 0555, file 0444) then exercises Save on close with an unsaved 200 ms edit: the application remains open, displays Permission denied and retains the edit; the saved file remains byte-exact (`dd69669ff40245381df27a3bb858fae049dda3d295f4dc40c5f76b1d18eddd0f`). Cancel returns to the edited form. After restoring directory/file permissions, Save on close persists 200 ms/Middle and exits. All modifications are confined to the private fixture; its original bytes are restored afterward.

An Alt+F4 trial was intercepted by the outer GNOME compositor and closed nested KWin; its immediate relaunch failed with NoCompositor and is excluded. A later fully initialized session uses the inner window's native close control, verifies the loaded Middle value, and supplies the accepted denial/cancel/retry receipt. A second early relaunch before the Wayland socket was ready is also excluded.

## KDE shortcut limitation

The frontend advertises GlobalShortcuts v1. The application sends stable id `toggle`, description and preferred F6. The KDE backend returns no assigned shortcut; NanoClick displays “The desktop did not assign a shortcut” and “Not registered · manual control available.” System Settings opens but has no NanoClick assignment to edit. F6 activation, restoration or rebind is **not accepted** on this backend.

The [KDE Plasma 5.27 backend source](https://github.com/KDE/xdg-desktop-portal-kde/blob/Plasma/5.27/src/globalshortcuts.cpp) restores actions from CreateSession options, and BindShortcuts opens System Settings and returns existing descriptions without using its incoming shortcuts list. The observed empty assignment is consistent with that implementation; this is an inference, not an upstream root-cause confirmation. No nonstandard request or manufactured shortcut success was added. Manual fallback is permitted by the existing architecture; advertised capability alone does not prove assigned-trigger support.

The first panel startup failed at the container's real init.scope TasksMax=307. Raising only that private, runtime-scoped limit to 1024 resolved the thread-creation error. This is excluded setup evidence, not a product defect or performance result.

## Ubuntu Orca 46 native checks

A separate headless GNOME profile runs its normal private XWayland service because shipped Orca 46 requires DISPLAY for its modifier map. NanoClick itself remains a Wayland client. The original no-X11 fixture causes Orca's KeyError for DISPLAY and is rejected. The accepted profile uses its actual Mutter-advertised :0 and generated private Xauthority; it does not connect to a host X server. A private PipeWire Pulse server with auto_null sink and Speech Dispatcher dummy module allows generated-speech inspection without host audio. The initial speech attempt without that audio service is rejected.

Real AT-SPI/Orca debug output verifies Mouse button/Left, changed Right, Theme/Light, Interval=1000 and Count=6. Eight forward steps from Save through the controls and back to Theme, and nine reverse steps from Mouse button back to itself, match independent focused-object/role inspections. The Count radio and entry remain distinct. Keyboard popup traversal exposes System as a toggle option; Escape restores Theme with its Light description. Default System is exposed in AT-SPI, but this receipt does not claim its spoken output. Rapid exploratory multi-key sequences are excluded from traversal acceptance; the recorded steps wait between individual transitions.

After actual native portal grant, one reader-only run completes six requests; another restored run stops after three with Escape. Both have zero failures/skips. The pointer is over an inert label in the private product window: these are producer/lifecycle checks, not independent delivery proof. Native announcements and generated speech contain Preparing, Starting in 3/2/1, Running, Completed, Releasing input and Ready. Progress labels change while no generated utterance contains “pairs sent” or “periods skipped.” Unsaved Light/Right changes are discarded using the native close dialog. Orca shuts down normally and restores its private modifier map; the final flushed log is used, rather than a partially buffered live tail.

This is real generated-reader output. Human listening, complete assistive-technology acceptance, KDE reader behavior and physical keyboard/monitor coverage remain open.

## Integrity and cleanup

| Raw artifact | SHA-256 |
|---|---|
| `kde-user/accepted.json` | `7d185e682aa09751d7bd6769e66013d81aa531b868484c0effbba8c2fd8ac68c` |
| `kde-user/capture.jsonl` | `8f10e0479c7fe03406c087149599f1201b32980e6cfc2243f743bbae558da0ca` |
| `kde-user/runs.jsonl` | `e72073392e25e19a92e48a9a703e2bd8c38afc460e0659d73599d5577dd514f8` |
| `kde-user/preferences-retry.json` | `e3808d86167eb42d6a011f1e3f2bfcb76fa6c0fdca3988acc9d0a6364a59af30` |
| `gnome-reader/accepted.json` | `9f9659e9bd03cd84a5d843551fd56a9e8e43fe454c8c043b8f65a2838e7abeac` |
| `gnome-reader/events.jsonl` | `c3d59a6dede1542c7553884df43d4df0eb86c6846a180aef117b8cd73224c6cd` |
| `gnome-reader/orca-debug.log` | `168bc789e6e24ff2c52ad90f5199de1e96a3cdc6d466b4beaff8aeb7601e7e01` |
| `gnome-reader/runs.jsonl` | `8f199d973c3fa97eaf85993c2ff7b34b0d6f623e9d48a86268f28b9935dc09b6` |
| `cleanup.json` | `29ec23f0d0df00cb9f67405cb96284ecfaa314b5e1568c920a19919b924b11c4` |

The GNOME input, GNOME reader, KDE root and KDE user profiles all restore exact original preference bytes (SHA-256 `b79f252b85920a538447e5725e04b8a789ca297affe65c1db696ff12fcb40f65`). The outer runtime permission is restored to 0700. Scoped process inspection finds no process for any of those four runtimes. The UID-1000 artifact directory is returned to the workspace owner. Finally the entire systemd container stops and Podman reports `exited`, `Running=false`. Private logs and configuration are retained as ignored artifacts; no system or host permission records were changed or claimed reset. The Windows VM and source Model O lab remain stopped and untouched by these desktop cases.
