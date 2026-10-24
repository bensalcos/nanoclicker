# NanoClick

A focused desktop autoclicker being rewritten in Rust for Windows and Linux. The interface is English-only, with light, dark and system themes.

**Development status:** the first Rust implementation is available. Core tests and isolated X11 input/shortcut checks pass. Windows code passes cross-target checks, but native Windows testing is pending. Wayland portal integration is implemented; consent, revocation and target delivery still need native acceptance. Performance budgets are targets, not verified product claims.

## Run the Rust application

Install Rust through [rustup](https://rustup.rs/), then run:

```sh
cargo run --locked -p nanoclick
```

The repository pins Rust 1.97.1. The declared minimum is Rust 1.95; the workspace has also passed a local check with Rust 1.95.0. Windows uses the MSVC target and requires Visual Studio Build Tools with the C++ workload. Linux needs a graphical X11 or Wayland session and OpenGL/EGL runtime libraries for the default renderer.

```sh
cargo run --locked -p nanoclick -- --theme light
cargo run --locked -p nanoclick -- --theme dark
cargo build --release --locked -p nanoclick
```

The default renderer is glow. The alternative wgpu build is available for comparison with `--no-default-features --features wgpu`; its native acceptance is pending.

## Use

- Set an interval from 1 to 999999 milliseconds and select Left, Right or Middle.
- Repeat until stopped, or send a specified count from 1 to 999999.
- **Start** provides three seconds to move the pointer to the target. **Cancel** interrupts that countdown.
- The global shortcut starts immediately or stops the active session. F6 is the initial request. On Windows/X11, apply another shortcut such as `Ctrl+Shift+F6`. Wayland delegates assignment to the desktop through **Configure in desktop…**.
- **Save** stores the current valid preferences for the next launch. Opening NanoClick always starts idle. Invalid or unsupported preferences files are preserved; move the file aside and restart to recover with defaults.
- **Escape** stops while the NanoClick window has focus. Closing the application requests cancellation and releases its input resources.

Clicks are sent at the current pointer position. “Pairs sent” describes backend submission, not proof that the target application processed them. A 1 ms requested interval does not guarantee 1000 clicks per second. Windows may block input into applications running at higher integrity. Wayland requires the desktop's RemoteDesktop portal and pointer permission; no root daemon is required. XWayland is not a substitute for native Wayland support.

## Verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/verify-x11.sh
scripts/verify-ui-x11.sh
```

The X11 script requires Xvfb and xauth. It starts a private display, captures the three mouse buttons, and checks shortcut activation, held-key suppression and conflicts. It never sends test input to the user's desktop. The UI script also starts the actual executable and verifies outside-focus shortcuts, manual preparation, input cessation and window shutdown.

For an isolated visual snapshot:

```sh
cargo build -p nanoclick --features eframe/__screenshot --locked
scripts/capture-ui.sh /tmp/nanoclick-dark.png dark
```

The screenshot feature is for verification, not release packaging.

## Project documentation

- [Current status](docs/STATUS.md)
- [Architecture](docs/architecture/RUST.md), [UI specification](docs/UI.md) and [validation budgets](docs/VALIDATION.md)
- [Migration phases](tasks/plan.md) and [task acceptance criteria](tasks/todo.md)
- [Implementation evidence](docs/evidence/implementation.md)

The C++/Win32 sources remain as the migration reference at commit `3fd5cd8`. They use C++20 and Visual Studio toolset v145 through `Autoclicker.slnx`; they will be removed only after platform acceptance. The MIT notice currently contains owner/year placeholders that must be resolved before distribution.
