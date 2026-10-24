# Hosted CI evidence

The user authorized pushing the migration branch and running hosted CI on 2026-10-24. Remote master remains the C++ reference; work is on `feature/rust-migration-preview`.

[Run 40090868511](https://github.com/bensalcos/nanoclicker/actions/runs/40090868511) at `e71fc32d568fd2dd8a21010d6463097ed9214e2f` completed successfully in all four jobs:

| Job | Verified scope |
|---|---|
| Ubuntu 24.04 desktop | fmt, Clippy, 39 Rust tests, 3 packaging tests, release build, isolated adapter/GUI X11 flow |
| Windows Server 2025 desktop | fmt, Clippy, 42 native Rust tests, 3 packaging tests, MSVC release build/link |
| Minimum Rust 1.95 | Workspace compile check |
| Alternative renderer | Wgpu feature Clippy and application tests on Linux |

The Windows logs confirm both blocked-parent rejection and first-save creation of nested directories. The first run failed on the former; the loader was fixed without weakening that assertion. See [preferences diagnosis](windows-preferences.md).

The `x11-evidence` artifact (id 12558943182, 1054 compressed bytes) was downloaded and both logs inspected. It records ten complete pairs for all three buttons, shortcut conflict/held-key behavior, and actual GUI countdown, minimum-size Start, no post-stop input and clean close. [Structured summary](ci-success.json); complete downloaded logs remain local under `artifacts/ci/`.

Windows headless tests include three native event/timer tests: per-run event isolation, cancellation of an armed long timer, and timer firing/rearming. That is not native SendInput into a target window, full Windows GUI/DPI/NVDA testing or timer latency acceptance. Xvfb does not establish Wayland/Orca support. Release binaries linking successfully do not certify installation or behavior on every machine.

The workflow uses contents:read and disables checkout credential persistence. Actions remain pinned to the verified upstream commits:

- [actions/checkout v7.0.1](https://github.com/actions/checkout/tree/3d3c42e5aac5ba805825da76410c181273ba90b1)
- [actions/upload-artifact v7](https://github.com/actions/upload-artifact/tree/cf430e030ddbb5b0abf93d22962f4752f3646cd9)

The draft PR is still blocked by the current GitHub credential's PR-write permission; its branch and successful push workflow are independently available. A separate GitHub connection authorization is pending.
