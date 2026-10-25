# Resume handoff

## State at handoff — 2026-10-25

The migration goal has resumed. The earlier pause snapshot is superseded.

- Branch: `feature/rust-migration-preview`.
- Last development commit before this guidance update: `2842c7d7b45e514b36053bc332d4af98b9182483` (large finite-count fixture/evidence).
- Latest producer change: `42abf7dac1f9b7ddb728b940fad2c543f516ba38` (queued shortcut delivery and explicit Stop priority).
- All four hosted jobs pass for producer source 42abf7d in run 40177331385. The subsequent test-only run 40178377214 also passed all four jobs.
- Local checks: 63 Rust tests, release GUI flows and N=1000 for all three buttons pass. Numeric budgets and native gates remain as recorded in `VALIDATION.md` and `ACCEPTANCE.md`.
- Available producer previews: `dist/preview-shortcut-queue/` (Linux) and `dist/preview-windows-42abf7d/` (Windows). Provenance/checksum scope is in `evidence/preview-shortcut-queue.json`.
- The approved draft PR remains uncreated because GitHub rejected `CreatePullRequest` permission. Its prepared body is `/tmp/nanoclick-rust-migration-pr.md`; that temporary file may not survive a host restart.

## Next work on resume

1. Use the existing Windows VM documented in `~/Git/ModelO_1-0-9/docs/windows-vm.md`. The lab directory is `~/Git/ModelO_1-0-9/artifacts/vm/windows11-lab/`; do not treat Windows testing as blocked by lack of a machine.
2. Inspect current VM/process state and preserve the existing Model O lab. Prepare an isolated NanoClick working overlay/configuration under this project's ignored artifacts directory, following `AGENTS.md`. Do not boot or mutate the source lab blindly.
3. Transfer the matching Windows preview and required QA assets through separate read-only media, keeping the guest offline and without physical USB/proxy access. Do not replace the Model O tools media or run its firmware/software for these tests.
4. Run the Windows functional/GUI cases in `NATIVE_QA.md`: startup/runtime dependencies, actual SendInput delivery and paired counts, F6 outside focus, finite/infinite modes, Stop/restart/close, preferences and scaling/accessibility. Check the virtual graphics backend before diagnosing renderer failures. Record actual Windows/VM versions and limitations.
5. Use the VM results to fix concrete defects and update the acceptance matrix. Keep physical-GPU/performance, Linux Wayland and other unverified gates separate; do not lower their budgets or accept them from VM/CI results alone.

The last read-only VM inspection found no QMP socket or QEMU process. That is a past observation, not a guarantee of its state at resume. No VM was started or changed during this handoff update. The unrelated `.aws` entry remains untouched.

## Windows VM continuation

The isolated NanoClick lab now exists at `artifacts/vm/windows11-lab/`, with its own overlay, UEFI/TPM state and read-only QA media. Recheck its live status before continuing. Windows 11 Pro build 26300.9457, two vCPU, 4 GiB RAM and Microsoft Basic Display Adapter were observed. No physical USB or network was attached. The source Model O lab was preserved.

Producer 42abf7d executable SHA-256 `ef2feacca2028f4644468e10217a6bf25992a63b6e2b114ae6f2cd0692d5ed10` fails before Rust main: `--help` exits with -1073741515 (0xC0000135); VCRUNTIME140.dll is missing. Evidence: ignored `artifacts/vm/windows11-lab/probe-console-02.png` and `pe-imports.txt`. The replacement build must use static CRT linkage and pass the new PE import check, then repeat startup in this VM before input/GUI acceptance. Do not install a redistributable to mask the portable-package defect.
