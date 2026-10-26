# Changelog

## Unreleased — Rust migration preview

### Added

- Three-crate Rust workspace separating scheduling, platform adapters and the desktop UI.
- English interface with light/dark/system themes, three mouse buttons, finite/infinite repetition, a cancelable manual countdown and global shortcuts.
- Windows SendInput and X11 XTEST adapters; Wayland RemoteDesktop and GlobalShortcuts portal adapters with lifecycle monitoring.
- Monotonic scheduling, exact finite counts, skipped-period metrics, priority cancellation and retained terminal snapshots.
- Versioned explicit preference saving and opt-in terminal timing diagnostics.
- Windows/Linux CI, preview archives with dependency notices and separate symbols, isolated integration checks and a passive native capture target.

### Changed

- Linux now selects WGPU provisionally by default after repeated GPU trials showed materially lower startup time and memory use; Glow remains a Linux build alternative. Windows always selects WGPU. Package instructions now describe each platform's graphics requirements.
- The current WGPU-default Linux preview passes the restricted Ubuntu 24.04 X11 GUI flow. Native Wayland, hardware compatibility across distributions, and full platform acceptance remain open.

### Fixed

- Windows selects wgpu by default to support the inbox display driver that exposed OpenGL 1.1 in the VM; the replacement and passive QA target now open in the Windows 11 VM.

- Windows builds select the static C runtime to avoid requiring a separately installed Visual C++ redistributable; the rebuilt preview now passes `--help` in the Windows 11 VM without the redistributable.

- Batched shortcut presses no longer cancel each other out; queued restarts wait for Stop acknowledgement, and explicit Stop/Close clears older requests.

- Windows preference-directory replacement and cancellation-event reuse between runs.
- X11 shortcut invalidation after pointer-only mapping changes and handling of real keyboard-map changes.
- Unbounded X11 protocol waits and caller waits during connection/authentication setup; blocked OS work is contained by a fixed helper limit.
- Minimum-window scrolling and action visibility.
- Concurrent Xvfb display selection in the test launchers; startup observation now retains owned-server diagnostics.

### Acceptance status

These are preview changes, not an accepted release. Native Windows/Wayland, real DPI/screen-reader validation and the full per-backend performance matrix remain open. Software-rendered measurements exceed the CPU/RSS budgets. The C++ reference remains available at `3fd5cd8863b316437e3e50339a1dce8b84aad215`.

See the [acceptance matrix](docs/ACCEPTANCE.md), [current evidence](docs/STATUS.md) and [rollback guide](docs/ROLLBACK.md).
