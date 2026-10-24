# Synthetic X11 scaling review

Date: 2026-10-24. Application source: `0435c9e`. Renderer: glow, debug build with eframe's screenshot hook. Private Xvfb screen: 2048×1536, no input session started, temporary configuration directory. This is a layout check, not a performance measurement or native DPI acceptance.

Reproduce:

```sh
cargo build --locked -p nanoclick --features eframe/__screenshot
scripts/capture-ui.sh /tmp/nanoclick-dark-2x.png dark 2
```

The optional third argument accepts `1`, `1.5` or `2` and sets winit's documented X11 scale override. Window content remains 520×600 logical units, producing 520×600, 780×900 or 1040×1200 physical pixels. The screenshot hook exits after the second frame; it does not test keyboard navigation or screen readers.

| Scale | Dark | Light | Visual result |
|---|---|---|---|
| 100% | [capture](screenshots/linux-x11-dark-1x.png) | [capture](screenshots/linux-x11-light-1x.png) | Three control sections, Save/theme and main action visible |
| 150% | [capture](screenshots/linux-x11-dark-1.5x.png) | [capture](screenshots/linux-x11-light-1.5x.png) | Same logical layout, readable text |
| 200% | [capture](screenshots/linux-x11-dark-2x.png) | [capture](screenshots/linux-x11-light-2x.png) | Main controls fit; partial help-text edge visible above action panel |

All six images were visually inspected. No main control or title is cropped at the default window size. However, the explanatory paragraph below the shortcut section is below the scroll fold; at 200% a thin text fragment is visible at the clipping boundary. This remains visual debt to fix before UI acceptance. The text-field/radio outlines also need a measured non-text contrast review, and accessible field names need completion.

Still pending: minimum-size 420×520 interaction and scrolling, keyboard focus/order, long errors, active states, monitor changes, native Windows/Linux scaling, NVDA and Orca. AccessKit is enabled but that alone does not demonstrate accessibility. T02/T14 remain open.
