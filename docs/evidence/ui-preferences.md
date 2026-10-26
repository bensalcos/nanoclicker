# GUI preferences Save and relaunch

Date: 2026-10-24. Production source: `0c39233`; release executable SHA-256 `317322b93cd353775828192b7b5f7f12ef14630f85fba8d266488fa64b845c6f`.

The X11 GUI fixture now operates the actual Save button and interval field. It uses only a fresh private XDG_CONFIG_HOME and the private Xvfb display created by `scripts/verify-ui-x11.sh`.

1. Launch the release app, click Save, and wait for the preferences file.
2. Edit the interval to 37 ms, click Save, and check the versioned JSON and six expected fields.
3. Edit to 91 ms without saving. Start and stop through F6, capture twelve press timestamps and complete ordered pairs, and verify the observed median is in 70–115 ms. This establishes that the unsaved edit reached the actual controller.
4. Close normally and verify the saved bytes are unchanged.
5. Relaunch, focus the capture target and observe 500 ms without input. Explicitly Start, capture twelve timestamps and ordered pairs, and verify a 25–65 ms median, distinguishing the saved 37 ms plan from both the 91 ms unsaved edit and the 100 ms default.
6. Remove only the fixture's known preferences file, click Save again, wait for a new publication and verify the original bytes. A stale matching file cannot satisfy this check.

The captured restored median is 37 ms. The broad ranges above discriminate plans; they are not deadline-lateness or performance acceptance thresholds. Theme is explicitly forced dark by the shared launcher, so the X11 case does not prove restoration of a saved theme. The Windows follow-up below verifies native restoration of theme, mouse button, count and shortcut. Native Wayland remains open.

## Windows 11 native desktop follow-up

Date: 2026-10-25. This used the isolated Windows 11 VM and the verified preview copy at `C:\NanoClickLab\NativeGraphics2\App\nanoclick.exe` (Windows preview executable SHA-256 `6c36688780f892be6f64d88fb64b56b0f97996ab4001627220d7ec60900a4be0`). The passive input target remained Ready in the guest; no synthetic click run was started during this preference check.

The actual Save button displayed “Preferences saved.” after saving 37 ms, Right, Count=7, Light and F7. After normal close and relaunch, all five values were restored; the UI showed Ready and zero pairs sent. Changing only the interval to 91 ms without saving, closing and relaunching restored 37 ms and the same other four saved values. This confirms the unsaved edit was discarded. Finally, System, Left, 100 ms, Count=10 and F6 were saved, then restored by a final close/relaunch; the app again opened Ready with zero pairs sent. The VM is left with those default values saved. Screenshots are in `artifacts/vm/windows11-lab/`: `preferences-saved.png`, `preferences-relaunch-saved2.png`, `preferences-unsaved-edit.png`, `preferences-unsaved-discard-relaunch.png`, `preferences-default-saved.png` and `preferences-default-relaunch.png`.

This verifies normal Windows desktop persistence and idle startup for all serialized preference fields. It does not cover invalid-file GUI recovery, blocked directories, native Wayland behavior or filesystem fault injection; those remain open.

The fixture initially missed the interval because its targeting raced the newly inserted Save-status line. It now waits for that frame and for text focus before sending keystrokes; no production delay or retry was added. The runner removes its own fresh configuration directory after the test. `serde_json` is a Linux development dependency for reading the saved document, already present at the locked version; runtime dependencies are unchanged.

Reproduce with:

```sh
cargo build --release --locked -p nanoclick
scripts/verify-ui-x11.sh target/release/nanoclick
NANOCLICK_UI_STRESS_CYCLES=20 scripts/verify-ui-x11.sh target/release/nanoclick 2
```

Local release flows pass at 100% and 200% scale, including the existing manual Start, complete-pair, minimum-window, Stop and active-close checks. The 200% run also includes twenty measured stress cycles. Xvfb proves this isolated X11 flow; native monitor/DPI, Wayland and Windows acceptance remain separate. Hosted run 40121663794 at e244efd also passes the release preferences case; its downloaded GUI log was inspected. See [CI evidence](ci.md).

## Windows 11 invalid-preferences recovery — 2026-10-26

The exact `cfd4a64ff6f73d25ce3364dfd9795919aa1cef2f` Windows preview from hosted run [40313423347](https://github.com/bensalcos/nanoclicker/actions/runs/40313423347) was mounted read-only as E: in the isolated Windows 11 VM. The VM was Windows 11 Pro build `26300.9457`, 2 vCPU, 4 GiB RAM and 1280×800 at 100%. The preview package manifest reports `x86_64-pc-windows-msvc`, WGPU, a clean packaging tree and no unresolved dependency notices. The application SHA-256 is `95f86e882648fbee74f6f37106cbd83fa4cf615cae294d2bfce2f1d0b6717820`; the transfer ISO SHA-256 is `a11dcaeb01c228aed664445a951edcdf56363e2e28cda031f22f7137cb316e13`.

The test first copied `%APPDATA%\bensalcos\NanoClick\config\preferences.json` to `C:\NanoClickLab\preferences-original-cfd4a64.json`. Both files had SHA-256 `70B7FC620F1D03BE45E8286FDDBC21CD695823313C34419F4A851B3BE9C45A60`. The source file was then replaced with exactly three invalid bytes (`123,255,125`, SHA-256 `5B3430EE5EC57490D0E154755CDAE0C9A7791BE87E77B1F91A52F77676BED0C7`). On launch, NanoClick displayed the parse error, stated that it preserved the original, described safe defaults and exposed “Back up and replace damaged preferences.” Invoking it created `preferences.json.invalid-1793039418423792400-0.bak`; its bytes and SHA-256 exactly matched the three-byte damaged file. The replacement parsed as valid JSON and represented the current safe settings.

After closing NanoClick, the saved original was copied back to the active preferences path; readback SHA-256 matched the original. Relaunch showed the original System, Left, 100 ms, Count=10 and F6 values, Ready state, and no corruption message or recovery button. The generated `.invalid-*.bak` remains in the isolated guest overlay as test evidence. Relevant local screenshots are `recovery-corrupt-ui-ready-cfd4a64.png`, `recovery-success-cfd4a64.png`, `recovery-verify-cancelled-cfd4a64.png` and `recovery-restored-final-cfd4a64.png` in the ignored VM lab directory.

This accepts the Windows 11 invalid-file recovery success path and byte-preserving backup for this artifact. It does not exercise unsupported future schema recovery, backup/save failures, blocked directories, Windows filesystem fault injection, Linux native GUI recovery, native Wayland or reader/DPI acceptance.

## Windows 11 recovery and protected-path regression — 2026-10-26

The exact `fe807374640647381a20ae0d01c96688b975ffb3` Windows preview from hosted run [40318748629](https://github.com/bensalcos/nanoclicker/actions/runs/40318748629) was mounted read-only as E: in the same isolated Windows 11 Pro VM (build `26300.9457`, 2 vCPU, 4 GiB RAM, 1280×800 at 100%). The verified executable SHA-256 is `fd4f1d0425db1d0bed81122c84a78039454382b0244dec7667cae7b285e90017`; the transfer ISO SHA-256 is `a3a8ecc8d335faac6a1eccac74d53b955fb332d98d4c6563e49317033d3744ee`.

First, the canonical preferences file was replaced with exactly `[123,255,125]` (SHA-256 `5B3430EE5EC57490D0E154755CDAE0C9A7791BE87E77B1F91A52F77676BED0C7`). NanoClick showed “Back up and replace damaged preferences.” Clicking it saved valid JSON and created `preferences.json.invalid-1793041836088932600-0.bak`; the backup bytes were exactly `7B-FF-7D` and its SHA-256 matched the damaged input. The saved document parsed successfully. The original preferences and a temporary snapshot both had SHA-256 `70B7FC620F1D03BE45E8286FDDBC21CD695823313C34419F4A851B3BE9C45A60`.

Next, the complete `config` directory was copied to a temporary guest snapshot, moved aside, and replaced at its original path with a regular file. On launch, the UI reported “A parent of the preferences path is not a directory. The original path was preserved.” The damaged-preferences recovery button was absent. Clicking Save produced the protected-path message: “The original preferences file is protected because it could not be loaded. Move it aside and restart NanoClick before saving new preferences.” No recovery or replacement was offered for this path-obstruction state.

After the app was closed, the obstruction file was removed and the original `config` directory moved back. The restored preferences SHA-256 again matched `70B7FC620F1D03BE45E8286FDDBC21CD695823313C34419F4A851B3BE9C45A60`; the JSON and the earlier three-byte `.invalid-*.bak` remained in the directory. The temporary snapshots were removed, NanoClick relaunched and loaded System, Left, 100 ms, Count=10 and F6 with Ready status and no recovery warning. Screenshots in the ignored VM lab directory: `fe80737-corrupt-ui.png`, `fe80737-corrupt-recovered.png`, `fe80737-corrupt-verify.png`, `fe80737-obstruct-ui2.png`, `fe80737-obstruct-save-error.png`, `fe80737-obstruct-restored2.png`, `fe80737-final-relaunch.png` and `fe80737-cleanup-result.png`.

The same artifact was tested with a well-formed version-99 document (94 bytes). NanoClick reported “Preferences version 99 is not supported. The file was preserved.” and offered recovery. Recovery backed up the exact original document (SHA-256 `75DC060E2CF0BDBF71EA188A6D37E5A2829955CE4BF9D109E1BB917B5F4F5CDC`), wrote valid JSON and returned to the normal UI. The original profile was restored with the expected SHA; the future-version backup remains in the guest as evidence.

To exercise backup creation failure, the test wrote the known three-byte corrupt input and applied a temporary ACL denying only `WD` (add file) on the config directory to the guest Administrator. The app could still read the damaged file and showed recovery. Clicking it produced “Preferences could not be read or saved: Access is denied. (os error 5)” for its temporary file path; the recovery action remained available, and the corrupt source bytes remained exactly `7B-FF-7D` (SHA-256 `5B3430EE5EC57490D0E154755CDAE0C9A7791BE87E77B1F91A52F77676BED0C7`). No new backup appeared. The ACL was removed, the original preferences were restored with SHA-256 `70B7FC620F1D03BE45E8286FDDBC21CD695823313C34419F4A851B3BE9C45A60`, and the app relaunched normally. Screenshots: `fe80737-future-ui.png`, `fe80737-future-recovered.png`, `fe80737-future-verify.png`, `fe80737-future-restored.png`, `fe80737-acl-narrow-ui.png`, `fe80737-acl-recovery-error.png`, `fe80737-acl-restored.png`, `fe80737-final-clean.png` and `fe80737-final-state.png`.

This verifies, on the exact `fe80737` Windows artifact, bounded corrupt-file recovery, future-version recovery, blocked-parent protection, failed backup creation with source preservation, ACL restoration and normal relaunch. Save-path obstruction reports an error without replacing preferences. Native Linux/Wayland write-error presentation and reader/DPI acceptance remain open.

## Linux X11 recovery — 2026-10-26

The same `cfd4a64` application source was built into a temporary Linux X11-only variant on Fedora 44 because this host lacks the Wayland development metadata required by the complete build. In a throwaway source copy, the `eframe` feature list omitted `wayland` and `accesskit`; the preference and UI source files were unchanged. Cargo recalculated that copy's lockfile offline for the reduced feature graph. The WGPU release binary SHA-256 was `d230985293e5f7af3a6c6e32bcb29a848da9994e206f2c95dcd6597551e1c2dc`. The app ran on an owned, authenticated 1024×768 Xvfb display, with no connection to the user's desktop and a fresh temporary `XDG_CONFIG_HOME`.

The test began with `config/nanoclick/preferences.json` containing exactly `123,255,125` (SHA-256 `5b3430ee8e5c7490d0e154755cdae0c9a7791be87e77b1f91a52f77676bed0c7`). The GUI displayed its invalid-file explanation and recovery button. Clicking the button showed “Current settings saved. The original file was backed up to …”; the generated backup matched the damaged bytes exactly and had the same SHA-256. The new preferences document parsed successfully as `{"version":1,"interval_ms":100,"button":"left","count":null,"theme":"system","shortcut":"F6"}` (SHA-256 `ee39faf935e0e0cb45f3d1fec65f59866f22951e189c099e122b9c0d2e6d78a0`). A second launch loaded System, Left, 100 ms, Until stopped, F6 without a recovery warning. Screenshots are `linux-x11-recovery-invalid-cfd4a64.png`, `linux-x11-recovery-saved-cfd4a64.png` and `linux-x11-recovery-relaunch-cfd4a64.png` in the ignored VM artifact directory.

This accepts the invalid-file success path for the Linux X11 GUI source variant. It is not a full-feature Linux package or accessibility check: Wayland and AccessKit were excluded from this temporary build because the host lacks their development metadata. Native Wayland recovery, full-feature Linux release acceptance, unsupported-version recovery and backup/save error presentation remain open.
