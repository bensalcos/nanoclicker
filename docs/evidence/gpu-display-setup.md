# Isolated GPU display for measurement

The previous Xvfb runs used software rendering. This machine also exposes AMD render device 1002:15e7 through amdgpu. A private headless Weston 15.0.1 compositor and rootful Xwayland now provide a second measurement environment without opening the host desktop or attaching physical input devices. This is an experimental test environment, not the application's Wayland portal backend.

```sh
scripts/measure-ui-gpu.sh /path/to/extracted/usr target/release/nanoclick --validate-only
scripts/measure-ui-gpu.sh /path/to/extracted/usr target/release/nanoclick --quick-profile
scripts/measure-ui-gpu.sh /path/to/extracted/usr target/release/nanoclick
```

The prefix must contain a Fedora-style Weston 15 lib64 installation. This host used Fedora 44 weston/weston-libs 15.0.1-2, downloaded with needed dependencies from only fedora/updates, verified with `rpm -K`, and extracted into a temporary directory with rpm2cpio/cpio. Nothing was installed system-wide. These tools are test prerequisites and are not shipped with NanoClick. Rootful Xwayland, glxinfo, dbus-run-session, Python and the Rust toolchain are also required. The application argument must be an x86_64 ELF binary; non-ELF script wrappers are rejected.

The script creates a private runtime directory and D-Bus configuration without activation directories. It starts only Weston's headless GL backend with a fake seat and a 2048x1536 virtual output. Rootful Xwayland runs as a client of that private compositor, with TCP listening disabled. Its display number comes from the owned child process's displayfd, and the executable identity is checked before input is enabled. GLX must report accelerated rendering. Application/library probes do not inherit Weston's temporary library/module paths. Child status is recorded even on setup errors, and a bounded timeout terminates the owned process group. Logs remain in the reported temporary evidence directory; runtime sockets are removed afterward.

The launcher first runs the complete release-UI acceptance flow, then writes a private 10 ms settings fixture and executes the existing performance harness. Quick mode uses ten-second phases and cannot accept the sixty-second budgets. Full mode performs thirty warm-start samples plus sixty-second stabilization, idle and active phases. Validation-only mode collects no performance figures.

## Fixture findings and limits

Rootless Xwayland reported failed EI input-emulation setup and did not deliver the injected shortcut in this configuration; it is not used for accepted measurements. Inspection confirmed NanoClick was reparented beneath a manager frame, so the discovery helper now searches a bounded tree rather than only direct root children. Capture targets are painted, unmanaged and explicitly raised. Manual Start must be acknowledged before the fixture changes focus.

The reusable rootful setup also exposed a cursor reset to (100,100) during the manual countdown, outside the target at (1400,100). Only the GPU fixture reasserts the synthetic cursor when it drifts; its correction count is logged. Three fresh-session validation runs and the final full benchmark preflight pass with that accommodation. The ordinary Xvfb test explicitly clears the GPU marker and does not hold the pointer on its target. This is not physical-pointer QA, and the exact cause of the emulated reset was not fully localized. The application itself sends button requests rather than pointer-motion requests.

Performance collection does not run the cursor guard: it positions the pointer before shortcut Start and captures the actual resulting input. The hardware-backed display must not be substituted for native GNOME/KDE consent/revocation, a real Xorg desktop, monitor transitions or screen-reader acceptance. Software-rendering failures remain open; no threshold or application feature was removed.

Primary reference: [Weston running/backends/renderers](https://wayland.pages.freedesktop.org/weston/toc/running-weston.html). The installed Xwayland manual also documents rootful geometry/fullscreen mode and warns nested compositors against enabling the portal input-emulation flag; no such flag is passed here.
