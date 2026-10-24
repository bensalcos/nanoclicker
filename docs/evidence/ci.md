# Hosted CI evidence

The user authorized pushing the migration branch and running hosted CI on 2026-10-24. Remote master remains the C++ reference; work is on `feature/rust-migration-preview`.

[Run 40092934864](https://github.com/bensalcos/nanoclicker/actions/runs/40092934864) at `acc817c9f11714d8a74576d1dedda4512e237963` completed successfully in all four jobs:

| Job | Verified scope |
|---|---|
| Ubuntu 24.04 desktop | fmt, Clippy, 42 Rust tests, 8 packaging tests, release build, isolated adapter/GUI X11 flow |
| Windows Server 2025 desktop | fmt, Clippy, 45 native Rust tests, 8 packaging tests, MSVC release build/link, verified preview ZIP/PDB |
| Minimum Rust 1.95 | Workspace compile check |
| Alternative renderer | Wgpu feature Clippy and application tests on Linux |

The Windows logs confirm both blocked-parent rejection and first-save creation of nested directories. The first run failed on the former; the loader was fixed without weakening that assertion. See [preferences diagnosis](windows-preferences.md).

The `x11-evidence` artifact (id 12559950979, 1055 compressed bytes) was downloaded and both logs inspected. It records ten complete pairs for all three buttons, shortcut conflict/held-key behavior, and actual GUI countdown, minimum-size Start, no post-stop input and clean close. [Structured summary](ci-success.json); complete downloaded logs remain local under `artifacts/ci/`.

The Windows preview artifact (id 12560216356) was downloaded and independently verified on Linux: archive hashes, packaged executable input hash/PE architecture, matching provenance, PDB signature, required files and zero unresolved notices. Its dependency inventory contains 128 entries. This verifies bundle structure, not application execution.

Windows headless tests include three native event/timer tests: per-run event isolation, cancellation of an armed long timer, and timer firing/rearming. That is not native SendInput into a target window, full Windows GUI/DPI/NVDA testing or timer latency acceptance. Xvfb does not establish Wayland/Orca support. Release binaries linking successfully do not certify installation or behavior on every machine.

The workflow uses contents:read and disables checkout credential persistence. Actions remain pinned to the verified upstream commits:

- [actions/checkout v7.0.1](https://github.com/actions/checkout/tree/3d3c42e5aac5ba805825da76410c181273ba90b1)
- [actions/upload-artifact v7](https://github.com/actions/upload-artifact/tree/cf430e030ddbb5b0abf93d22962f4752f3646cd9)

The draft PR is still blocked by the current GitHub credential's PR-write permission; its branch and successful push workflow are independently available. A separate GitHub connection authorization is pending.

Follow-up [run 40094952287](https://github.com/bensalcos/nanoclicker/actions/runs/40094952287) at `0a4837e0937b3805603f5191bc017bc48b579f24` also passes all four jobs and adds eight private-D-Bus lifecycle scenarios on Ubuntu. Its `x11-evidence` artifact (id 12560264318, 1413 compressed bytes) was downloaded; x11.log, ui-x11.log and portal.log were inspected. The Windows artifact verification above remains specifically tied to run 40092934864. Follow-up [run 40096403001](https://github.com/bensalcos/nanoclicker/actions/runs/40096403001) at `0a08a08fb604c50addb1fb0000a306a00487a0e7` passes all four jobs. Its downloaded GUI log confirms the release executable, ten warm-up plus twenty measured Start/Stop cycles, complete pairs, stable descriptors, and graceful close during preparation, running and the Stop-request race. The portal log confirms all eight mock scenarios. Sampled GUI RSS was 126680 KiB, with 18 descriptors and 15 threads at both samples; this is lifecycle evidence, not a RAM-budget pass. These logs remain under `artifacts/ci/linux-run-40096403001/`.
