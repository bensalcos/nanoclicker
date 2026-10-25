# Native desktop QA

NanoClick previews are available; native acceptance still needs real Windows 11 and GNOME/KDE Wayland sessions. The passive capture example provides a target without injecting input or requesting desktop-control permission.

Build the target from the same checkout as the app:

New Windows CI preview artifacts also include `qa/input_capture.exe` and its commit/hash metadata. Use the license and dependency notices in the accompanying preview ZIP. The target uses the same platform renderer selection as NanoClick.

```sh
cargo build --release --locked -p nanoclick --example input_capture
```

On Linux, launch `target/release/examples/input_capture NEW_CAPTURE.jsonl`. On Windows, launch `target\release\examples\input_capture.exe NEW_CAPTURE.jsonl`. Choose a new filename in an existing writable directory; existing files are never overwritten. The Linux release target is also available locally at `target/release/examples/input_capture`.

1. Open NanoClick and select Count, N=10 and a 100 ms interval. Pick one button. Enable its global shortcut, completing any desktop permission/configuration dialogs manually.
2. Focus the capture window and move the pointer inside it. Check its actual backend: Windows requires `windows`; a native Wayland case requires `wayland`. `x11` on a Wayland desktop is an XWayland result.
3. After foreground activation, move the pointer within the target's client area, press Space and verify that Recording is visible with reset counters before using NanoClick's registered shortcut. Wait for the observed result before changing focus or pointer location. Avoid physical mouse clicks in the target; use Alt+Tab when changing focus. A producer count alone does not prove target delivery; retain and investigate mismatches.
4. After NanoClick completes, press Space to save the capture session. Require exactly ten down/up/pairs for the chosen button, zero for other buttons, no anomalies/held buttons, `balanced=true`, `unexpected_buttons=false` and `overflowed=false`. Repeat for each supported button. Session IDs increase and counters reset.
5. Repeat N=1, an infinite run stopped by the shortcut, cancellation during preparation and close while active. Record each result separately. For Wayland, also test denial and desktop revocation where available; record zero target pairs after denial and the application's visible terminal outcome after revocation.
6. Test the actual NanoClick UI with keyboard navigation, the platform screen reader and real monitor/DPI transitions. The capture target does not accept those product checks on its own.

Escape or closing the target saves an active session; a forced kill may leave no summary. Output failures are visible and cause a nonzero process exit after closing. The JSONL contains backend/version metadata and per-session aggregates, including start/end GUI scale. No key text, pointer coordinates, device identifiers or per-event traces are written. File permissions are 0600 on Unix.

These are GUI event counts, not native event timestamps, global physical button-state proof or performance measurements. Compare the target counts with NanoClick's terminal count; the producer's optional `--diagnostics NEW_JSONL_PATH` can retain that terminal snapshot. Record OS/build, desktop/compositor/portal, GPU/driver, app commit/hash, effective shortcut, permission decisions, scales and any mismatch with the logs. Keep native and isolated/mock evidence separate.
