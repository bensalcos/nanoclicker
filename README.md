# NanoClick

A focused desktop autoclicker being rewritten in Rust for Windows and Linux. The interface is English-only, with light, dark and system themes.

**Development status:** the first Rust implementation is available. Core tests and isolated X11 input/shortcut checks pass. Hosted Windows Server 2025 passes 50 native Rust tests and builds the preview ZIP/PDB; native Windows desktop/input/DPI/screen-reader acceptance remains pending. Wayland portal integration is implemented; consent, revocation and target delivery still need native acceptance. Performance budgets are targets, not verified product claims.

## Run the Rust application

Install Rust through [rustup](https://rustup.rs/), then run:

```sh
cargo run --locked -p nanoclick
```

The repository pins Rust 1.97.1. The declared minimum is Rust 1.95; the workspace has also passed a local check with Rust 1.95.0. Windows uses the MSVC target and requires Visual Studio Build Tools with the C++ workload. Linux needs a graphical X11 or Wayland session and OpenGL/EGL runtime libraries for the default renderer. On Ubuntu 24.04, X11 also requires `libxkbcommon-x11-0`; the clean runtime check installs `libx11-6`, `libxkbcommon0`, `libxkbcommon-x11-0`, `libgl1`, `libegl1`, `libxi6`, `libxrandr2`, `libxcursor1` and `libxinerama1`.

```sh
cargo run --locked -p nanoclick -- --theme light
cargo run --locked -p nanoclick -- --theme dark
cargo build --release --locked -p nanoclick
```

The default renderer is glow. The alternative wgpu build is available for comparison with `--no-default-features --features wgpu`; its native acceptance is pending.

## Use

- Set an interval from 1 to 999999 milliseconds and select Left, Right or Middle.
- Repeat until stopped, or send a specified count from 1 to 999999.
- **Start** prepares desktop input and any permission request, then provides three seconds to move the pointer to the target. **Cancel** interrupts that countdown.
- The global shortcut starts immediately or stops the active session. F6 is the initial request. On Windows/X11, apply another shortcut such as `Ctrl+Shift+F6`. Wayland delegates assignment to the desktop through **Configure in desktop…**.
- **Save** stores the current valid preferences for the next launch. Opening NanoClick always starts idle. Invalid or unsupported preferences files are preserved; move the file aside and restart to recover with defaults.
- **Escape** stops while the NanoClick window has focus. Closing the application requests cancellation and releases its input resources.

Clicks are sent at the current pointer position. “Pairs sent” describes backend submission, not proof that the target application processed them. A 1 ms requested interval does not guarantee 1000 clicks per second. Windows may block input into applications running at higher integrity. Wayland requires the desktop's RemoteDesktop portal and pointer permission; no root daemon is required. XWayland is not a substitute for native Wayland support.

Optional local aggregate timing export is available through `--diagnostics NEW_JSONL_PATH`; it never overwrites an existing path. See [diagnostic scope and benchmark use](docs/DIAGNOSTICS.md).

## Verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/verify-x11.sh
python3 scripts/verify-x11-stall.py
scripts/verify-ui-x11.sh
scripts/verify-ui-x11.sh target/debug/nanoclick 2
scripts/verify-portal.sh
```

The local X11 scripts require Python 3.11+, Xvfb and Linux /proc. Their shared launcher obtains a new display through Xvfb's descriptor, verifies ownership of both socket transports and supplies a private authentication cookie. It never selects a display by a free-number guess. The input script captures the three mouse buttons, and checks shortcut activation, held-key suppression and conflicts. It never sends test input to the user's desktop. The UI script also starts the actual executable and verifies outside-focus shortcuts, manual preparation, input cessation, the 420×520 logical minimum size, shutdown, finite N=1/7/1000 plans for all three buttons, and explicit Save/relaunch without automatic Start. Its optional second argument selects 100%, 150% or 200% X11 scaling (`1`, `1.5`, `2`).

The stalled-server script uses the same owned-display helper and pauses only its own Xvfb, covering protocol deadlines, cancellation and registration shutdown. It also creates its own authority FIFO to verify setup cancellation/timeout without touching personal authentication files. See [failure-test scope](docs/evidence/x11-stalled-server.md).

The portal script requires dbus-daemon and runs thirteen wire-level mock lifecycle scenarios on a private activation-disabled bus, without connecting to a compositor. It does not establish native GNOME/KDE acceptance.

For an isolated visual snapshot:

```sh
cargo build -p nanoclick --features eframe/__screenshot --locked
scripts/capture-ui.sh /tmp/nanoclick-dark.png dark
scripts/capture-ui.sh /tmp/nanoclick-dark-2x.png dark 2
```

The screenshot feature is for verification, not release packaging.

## Project documentation

- [Current status](docs/STATUS.md)
- [Changelog](CHANGELOG.md), [acceptance matrix](docs/ACCEPTANCE.md) and [rollback guide](docs/ROLLBACK.md)
- [Architecture](docs/architecture/RUST.md), [UI specification](docs/UI.md) and [validation budgets](docs/VALIDATION.md)
- [Migration phases](tasks/plan.md) and [task acceptance criteria](tasks/todo.md)
- [Implementation evidence](docs/evidence/implementation.md)
- [Accessibility checks](docs/evidence/ui-accessibility.md), [scaling captures](docs/evidence/ui-scaling.md) and [software-rendered UI measurements](docs/evidence/ui-polish-performance.md)

The C++/Win32 sources remain as the migration reference at commit `3fd5cd8`. They use C++20 and Visual Studio toolset v145 through `Autoclicker.slnx`; they will be removed only after platform acceptance. The MIT notice attributes the project to bensalcos (2026). Platform acceptance and the dependency maintenance warnings remain separate release checks.

Native desktop validation can use the passive capture target described in [Native QA](docs/NATIVE_QA.md). It receives window events without injecting input.
