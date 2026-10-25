# NanoClick project instructions

## Windows validation

- Use the user's existing Windows 11 QEMU/KVM lab as the primary environment for Windows functional and GUI tests. Do not assume Windows is unavailable merely because the host is Linux.
- Lab source: `~/Git/ModelO_1-0-9/artifacts/vm/windows11-lab/`.
- Before using it, read `~/Git/ModelO_1-0-9/docs/windows-vm.md` and inspect `~/Git/ModelO_1-0-9/scripts/vm/windows_lab.py` and the lab's actual configuration/status.
- Preserve the Model O lab. Prefer a NanoClick-specific overlay and copies of its documented checkpoint's UEFI/TPM state, created while the source VM is stopped. Preserve the immutable base and existing Model O overlay.
- Verify that every writable disk, UEFI/TPM, socket, PID and log path in the derived configuration belongs to the NanoClick lab; copying a manifest or changing `--directory` alone does not establish isolation.
- Keep network, shared folders, physical USB passthrough and the Model O USB proxy disconnected. A purely virtual guest pointer/controller is allowed. Never run firmware updates or modify the physical mouse for NanoClick testing.
- Inject test input only into the guest/test target, never the host desktop. Record Windows build, VM resources, display/backend, artifact commit/hash and results.
- Use the VM for real Windows input, shortcuts, counts, preferences, shutdown, scaling and accessibility checks. VM timing/GPU results do not automatically accept physical-hardware performance budgets.
- Keep GitHub Actions as complementary build/unit/packaging coverage. Reuse existing results/artifacts; avoid repeated full CI runs for documentation-only updates or as a substitute for the available VM.

## Resume context

- Read `docs/RESUME.md`, `docs/STATUS.md` and `docs/ACCEPTANCE.md` before continuing migration work. Revalidate their snapshots rather than restarting completed work.
- Keep application text and project documentation in English.
- The unrelated untracked `.aws` entry is outside this task: do not read, modify, remove, stage or commit it. Stage explicit project paths only.
