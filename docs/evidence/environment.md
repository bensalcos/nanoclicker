# Observed development environment

Date: 2026-10-23. Available session: Fedora Linux 44 Workstation, kernel 7.2.6-200.fc44.x86_64, GNOME Wayland. AMD Ryzen 5 5625U CPU, 6 cores/12 threads, 14.4 GiB RAM. Other applications are active; this is not a controlled benchmark machine.

Available Rust: rustc/cargo 1.97.1; initially installed target x86_64-unknown-linux-gnu. Xvfb is available for isolated X11 testing, along with libX11, libXtst and libwayland-client.

Actual read-only D-Bus query: RemoteDesktop version=2, AvailableDeviceTypes=7 (includes POINTER); GlobalShortcuts version=1. This establishes interface presence, not authorization, injection or effective shortcuts. No control of the user's desktop was requested.

At this initial inventory, Windows, Ubuntu 24.04, KDE, NVDA and Orca had no native evidence. Record exact compositor/portal and GPU versions for measurements. Later hosted/container/GPU evidence is tracked separately in STATUS.md and must retain its scope.

## Capture fixture and isolation

Run the X11 capture fixture in private Xvfb with a focused receiver window, collecting ButtonPress/ButtonRelease; never use DISPLAY=:0 for automated input tests. Windows capture requires a dedicated desktop/VM. Wayland requires a native client and desktop consent; Xvfb/XWayland cannot replace that evidence.

For each run retain commit, toolchain, release build, backend, renderer, versions, parameters, events, monotonic timestamps and result. Date and session state alone do not establish performance.
