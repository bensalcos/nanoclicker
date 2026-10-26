# Synthetic X11 scaling and minimum-size review

Updated 2026-10-24. Visual polish: `68380d4`; live-status and scaled-flow follow-up: `fb36ed5`. Glow renderer on a private Xvfb display. Default-size captures use a debug build with eframe's screenshot hook; runtime captures use the normal executable. These are layout/interaction checks, not native monitor-DPI or screen-reader acceptance.

```sh
cargo build --locked -p nanoclick --features eframe/__screenshot
scripts/capture-ui.sh /tmp/nanoclick-dark-2x.png dark 2
cargo build --locked -p nanoclick
scripts/verify-ui-x11.sh target/debug/nanoclick 2
```

The scale argument accepts `1`, `1.5` or `2` using winit's X11 override. The default window stays 520×600 logical units, producing 520×600, 780×900 or 1040×1200 physical pixels. The native-flow harness uses a private 2048×1536 screen and also resizes to 420×520 logical units.

| Scale | Dark | Light | Result |
|---|---|---|---|
| 100% | [capture](screenshots/linux-x11-dark-1x.png) | [capture](screenshots/linux-x11-light-1x.png) | Main controls, complete help text and primary action visible |
| 150% | [capture](screenshots/linux-x11-dark-1.5x.png) | [capture](screenshots/linux-x11-light-1.5x.png) | Same logical layout and complete help text |
| 200% | [capture](screenshots/linux-x11-dark-2x.png) | [capture](screenshots/linux-x11-light-2x.png) | Same layout; previous help-text fragment fixed |

All six refreshed images were visually inspected. Secondary buttons/fields retain 30 logical units of minimum height, and the primary action retains 46. Tighter secondary padding and removal of redundant section spacing make the help fit without increasing the window or hiding instructions.

At 100%, 150% and 200%, the actual executable passed: shortcut start/stop outside focus, complete input pairs, no input after observed Stop, manual countdown, Start at minimum size and clean close. At minimum size, a visible scrollbar exposes the remaining form; scrolling preserves the main action. Captures:

- [Minimum size](screenshots/linux-x11-dark-minimum.png), [scrolled](screenshots/linux-x11-dark-minimum-scrolled.png).
- [Minimum at 200%](screenshots/linux-x11-dark-minimum-2x.png), [scrolled at 200%](screenshots/linux-x11-dark-minimum-2x-scrolled.png).
- [Running](screenshots/linux-x11-dark-running.png), [preparing at minimum](screenshots/linux-x11-dark-minimum-preparing.png), [preparing at minimum and 200%](screenshots/linux-x11-dark-minimum-2x-preparing.png).

Still pending: physical monitor scaling/transitions, native Windows/Linux desktop review, light-theme active states and native NVDA/Orca use. See [accessibility evidence](ui-accessibility.md) for the automated tree, keyboard and contrast checks. T02/T14 remain open.

## Windows 11 VM DPI probe — 2026-10-26

The isolated NanoClick Windows 11 VM offers 100% and 125% scale at 1280×800. At 125%, both the already-running and freshly relaunched `0a3380c` preview clipped the bottom of the window, including part of Start. Windows also displayed its notice that some apps need to be restarted after scale changes. Fresh-process evidence is `artifacts/vm/windows11-lab/current-dpi-fresh-app-125.png`; the exact-source preview predates the current adaptive sizing change.

The current source now constrains the app's inner height against the active Windows monitor and native scale, reserving 112 physical pixels for the taskbar and window chrome while preserving the existing 520-point minimum. The helper has focused tests and the Windows target passes Clippy/type-checking. A rebuilt artifact still must verify that Start remains fully visible at 125%, that 100% remains unchanged, and that moving/scaling onto a smaller monitor applies the clamp. The VM was restored and visually verified at 100% in `current-dpi-restored-100-reprobe.png`. Native monitor transitions beyond the VM and screen-reader checks remain open.
