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

The isolated NanoClick Windows 11 VM offers 100% and 125% scale at 1280×800. At 125%, both the already-running and freshly relaunched `0a3380c` preview clipped the bottom of the window, including part of Start. The first size-clamp preview reduced the form body but a persisted frame position still left Start clipped. Windows also displayed its notice that some apps need to be restarted after scale changes.

The current correction constrains inner height against the active monitor, reserves 112 physical pixels for window chrome, and repositions the outer frame above the taskbar using a 72-pixel reserve. It preserves the 520-point minimum; focused geometry tests cover the 125% case and unchanged geometry on a standard monitor. The exact `c2f21a163b2b3ad25e63c7c3d11e63679b61245c` Windows artifact was built and packaged by hosted run [40306920310](https://github.com/bensalcos/nanoclicker/actions/runs/40306920310), which passed all four jobs. Its preview ZIP SHA-256 is `0dcafbe487f1bb19ba061d0a173551e1738f33ef6b1dcd88ad058dd671deb864`; the read-only VM ISO `artifacts/vm/windows11-lab/nanoclick-preview-c2f21a1.iso` has SHA-256 `5ebef09d67823e55dabf4d86d7cc9163a21a46c5df754ed1dbf5765ec55d3c9d`.

The guest executable hash `B6F33B00DB19707B211D39D2AECF49F292921DB121C051ED5F7DD69030C5257B` matches the packaged binary and embedded build metadata. The passive QA executable hash `51E35F6F4C6051B5354626A4D3145A37C864DA38C27B2220C7788E6170BFA7A1` matches `qa/build.json`. On the exact artifact, Start is fully visible at 125% on the 1280×800 desktop, with the frame above the taskbar: [125% screenshot](../../artifacts/vm/windows11-lab/c2f21a1-launch-125.png), [Settings showing 125%](../../artifacts/vm/windows11-lab/c2-settings-open.png). While NanoClick remained open, changing 125%→100% and then 100%→125% kept Start fully visible both times: [100% after changing down](../../artifacts/vm/windows11-lab/c2f21a1-app-100.png), [125% after changing up](../../artifacts/vm/windows11-lab/c2f21a1-app-100-to-125.png). At 100%, Start remained visible and Settings confirmed the scale: [Settings showing 100%](../../artifacts/vm/windows11-lab/c2f21a1-scale-100.png). The VM has been restored to 100%: [restored 100% screenshot](../../artifacts/vm/windows11-lab/c2f21a1-app-125-to-100.png).

This accepts the same-monitor Windows 11 1280×800 visual check at 100% and 125%, including live transitions in both directions. Multiple-monitor movement, other Windows display configurations, the wider 420×520/200% matrix and NVDA remain open; this is not full T14 acceptance.
