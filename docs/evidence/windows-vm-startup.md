# Windows 11 VM startup

Date: 2026-10-25. The user's existing Model O lab remains unchanged. NanoClick uses a separate overlay backed by its immutable Windows base, with private UEFI/TPM state and read-only QA media. Network, shared folders and physical USB are disconnected.

Environment: Windows 11 Pro 10.0.26300.9457, QEMU/KVM, two vCPU, 4 GiB RAM, 1280×800 at 100%, Microsoft Basic Display Adapter driver 10.0.26100.1. This is functional VM evidence, not reference-hardware performance acceptance.

## Missing runtime reproduced and fixed

Producer 42abf7d executable SHA-256 `ef2feacca2028f4644468e10217a6bf25992a63b6e2b114ae6f2cd0692d5ed10` exits before Rust main even with `--help`: -1073741515 (`0xC0000135`). It imports `VCRUNTIME140.dll`; that DLL is absent in this guest. Hosted CI had not exposed the missing dependency.

Source 441e679 selects static CRT linkage. Hosted run 40231110831 produces a Windows preview whose imports pass the new portable-runtime check. Its executable SHA-256 is `0975a1608494ff9c6d7a7cd2ba8b0195404fc47c94b6597c27ce95b135e50c26`. The copied guest executable matches the bundle's build metadata. In the same guest, without installing a redistributable, `--help` now returns 0 and prints usage. The preview ZIP SHA-256 is `e22085bc5854217eaf6050717c516cfb22d6db4ec52643940b7c37d0d9f7ac43`.

## Graphics incompatibility exposed

The same corrected executable's GUI exits with code 1: `OpenGL version is too low (1.1.0). egui requires OpenGL 2.0+.` This is a separate renderer issue, not a recurrence of the runtime-loader failure. Windows' default renderer subsequently changed to the existing wgpu backend; Linux retains glow. The replacement now opens in this VM; see [functional evidence and remaining discrepancies](windows-vm-functional.md).

Local ignored startup evidence: `artifacts/vm/windows11-lab/resume-runtime-evidence.png`, `static-crt-startup.png`, `pe-imports.txt`, the QA scripts/media and the guest's `C:\NanoClickLab\StaticCRT\help-result.json` and `startup-result.json`. Subsequent actual app-generated input is recorded separately in the functional evidence.
