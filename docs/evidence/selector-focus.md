# Selector focus after dismissal — 2026-10-30

## Reproduction and correction

The [extended native Orca trial](native-orca.md#changed-selections-popup-traversal-and-reverse-navigation--continuation) exposed a real focus loss in application source `a68fe3f`: Escape closed either selector's popup and left the native focus on the frame. The next Tab restarted at Theme, even if the user had been editing Mouse button.

A full-window regression reproduced the empty focus before the fix. Both selectors now request focus on their own trigger only when their popup was rendered and Escape was pressed. Pointer dismissal retains normal egui handling. No persistence, engine, key interception, dependencies or visual layout changed. The fix applies to both platform builds, but native acceptance below is Fedora GNOME only.

Two regression tests cover Escape from each selector followed by the next Tab, and clicking Interval outside an open selector without stealing focus back or leaving the popup open. All 83 workspace/all-target Linux tests passed (37 application tests). Linux and Windows MSVC workspace/all-target Clippy passed with warnings denied. The full-feature Linux release build passed with the existing linker deprecation warning. `graphify update .` completed. No Windows runtime, hosted CI or push is claimed.

## Native validation

The corrected full-feature WGPU executable SHA-256 is `e3aafa12184acae9ac50246eae7ecf143d1db4324040ee6c7ba9d6c26c5f08f2`. Its source is the two UI files in this change on base `21fb21e`; those files were otherwise clean before the correction. Fedora 44 GNOME Shell/Mutter 50.5, Orca 50.2, AT-SPI 2.60.7 and Speech Dispatcher 0.12.1 match the preceding trial.

Raw evidence is in ignored `artifacts/wayland-a11y-focus-20261030/`. The compositor, runtime, configuration, buses, reader, audio null sink and virtual input controller were private as documented in the earlier Orca evidence. The AT-SPI bus/registry were launched directly without security-policy changes. The stabilized per-key pointer-motion/repaint procedure was reused. Initial checks before raising the app had no native focus or an unexpected already-stored focus; those are rejected setup probes. Accepted cases begin at the separately asserted Theme trigger.

| Action | Asserted native result |
|---|---|
| Enter Theme; Down to System; Escape | Theme combo box, description System |
| Next Tab | Save button |
| Enter Mouse button; Down to Left; Escape | Mouse button combo box, description Left |
| Next Tab | Until stopped radio |
| Reopen Mouse button; click Interval outside it | Interval entry, text 100; no list box remains |
| Next Tab | Mouse button combo box |

`accepted.json` validates roles/names for these transitions and includes the separately captured pointer-dismissal receipt. The open/closed trees prove that the outside click actually dismissed an existing list box. Orca generated Theme/System after the first Escape and Mouse button/Left after the second. Its log at `14:01:20.928572` names Mouse button, followed by combo box and Left; after the outside click it generated Interval, entry, 100. These are generated-speech results through a private dummy/null backend, not human listening.

## Review, preservation and scope

Local review checked the Escape-only condition, the popup-visible guard, both trigger IDs, ordinary pointer behavior, unchanged accessible names/values and bounded per-frame work. The failing regression and native receipt match the same focus-loss behavior. No blocking finding remains for the scoped correction.

`cleanup.json` records zero diagnostic run rows, byte-exact original private preferences (SHA-256 `b79f252b85920a538447e5725e04b8a789ca297affe65c1db696ff12fcb40f65`), no remaining private-runtime process, and the original host compositor PID 4780 still active. No engine session or host input injection occurred.

This closes the reproduced Escape focus-loss case on the private Fedora GNOME backend. It does not claim Windows/NVDA or reference Ubuntu/KDE runtime validation, human listening, other lifecycle/active-counter announcements, physical monitors, timing/resource budgets or release acceptance. The general software-RSS work remains paused.

Raw receipt integrity:

- `accepted.json`: `f2dc78dee1e0beb3254643bc131bba1ebafd952a45cd3b69777ec91fbf351608`.
- `orca-debug.log`: `b384b97144bd64b966181e955a13aca8a0f2f7b4fb53e9f3d716f709396cbee1`.
- `steps.jsonl`: `7e8d2736b4c2caf9ab065044fbce2fdd9558208e9dd927556fbc006cf31e96fe`.
- `pointer-open-tree.json`: `1920e1da832dcad8d4883ce897b3ae4ed1cd68b4f78e8c2fc1309de454974119`.
- `pointer-closed-tree.json`: `2b92d05998f995a25a4514f114e48d58d7c5c1e9847bac118c119adfbeb4beac`.
- `cleanup.json`: `6d378c7bd7be98fe225758da8f1d39675965cdc56ab4d7c22dc535365ab0854e`.
