# Native Orca and AT-SPI follow-up — 2026-10-30

## Scope and artifacts

This is real AT-SPI/Orca coverage on a private Fedora 44 GNOME Wayland session, not reference Ubuntu/KDE or Windows/NVDA acceptance. GNOME Shell/Mutter 50.5, Orca 50.2, AT-SPI 2.60.7 and Speech Dispatcher 0.12.1 were installed. The headless display was 1280×800, using the full-feature WGPU release build. Source base was `ac12170b7f20d973caf96499491e460a3ae58b78`; the three UI source files in this change supply the correction.

| Artifact | SHA-256 |
|---|---|
| Before-fix application (unchanged `cc7bea0` product source) | `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d` |
| Corrected application tested here | `31b90cec3e2200ee6d0c9ddf321a034bc9fa033293f383ed318df0cec118c70e` |

Raw logs, screenshots, original binary and fixture scripts are retained locally under ignored `artifacts/wayland-a11y-20261030/`. They are not release assets.

## Isolation and fixture limitations

The compositor, D-Bus, AT-SPI bus/registry, Orca, configuration, runtime, PipeWire/Pulse and Speech Dispatcher were private. Input used the QA Clutter virtual controller bound explicitly to that private bus. No host input was injected. Speech used a dummy module and a private null sink; the results below prove generated speech text, not human listening or audible quality. Host Orca was absent before this session.

Automatic AT-SPI bus activation failed with Permission denied under the existing SELinux policy. Directly launching the private bus and registry worked; no security policy was changed. An initial wrong status-object path and initial audio-backend setup failures are rejected fixture attempts. The correct status object is `/org/a11y/bus`. Private settings only were changed. Orca's debug output is buffered: its final terminal-status line appeared when the private reader was terminated; a delayed log line is not a product or reader hang.

Rapid Tab batches coalesced before repaints and are rejected as navigation evidence. The accepted traversal uses one key, a fresh screencast frame and an AT-SPI focus query per step. These are functional checks, not timing measurements. An earlier consent timeout (`runs.jsonl`) is rejected as cancellation evidence; the accepted trial is `runs-raw.jsonl` below.

## Reproduced defects and correction

Before the correction, Theme and Mouse button exposed stable native names and ComboBox roles, but no selected child, no string Value interface and no selected-text description. Orca spoke Theme and its role without System. AccessKit's internal string value tests had therefore missed the native bridge behavior. On Linux the selectors now mirror their current text to the accessible description, while retaining their name and value. Other platforms keep their existing selector description behavior.

The status had a string value but an empty native name. The installed AccessKit AT-SPI adapter emits live announcements from the computed name; Role::Status does not derive that name from its value. Supplying the visible status explicitly as the label fixes native announcements while retaining a polite atomic live region. The progress counter remains ordinary, non-live text.

## Accepted native results

The corrected native status has name `Ready`. The collapsed selectors keep names Theme and Mouse button; their descriptions are System and Left respectively. They still have no selected child and do not implement a string Value interface; the description is the independently readable current text.

From Save, nine individual Tab steps reached exactly: Interval entry (text 100), Mouse button combo box (description Left), Until stopped radio, Count radio, Count entry (text 2), Configure in desktop… button, Start button, Theme combo box (description System), Save button. `navigation-verified.json` contains all nine asserted focus results and per-step timestamps. Reverse traversal, changed-selection speech and popup navigation are not accepted by this trial.

Orca generated the following output in `orca-restarted-debug.log` (session log clock):

| Time | Generated speech |
|---|---|
| 05:30:29 | Mouse button; combo box.; Left. |
| 05:30:41 | Count; entry; 2. |
| 05:30:53 | Theme; combo box.; System. |
| 05:35:15.617757 | Preparing desktop input… |
| 05:35:19.070986 | Could not complete the session |

Manual Start opened the real RemoteDesktop consent dialog. Cancel was clicked after about 3.4 seconds. `events.jsonl` records both status-name changes and corresponding `object:announcement` events with polite priority (`detail1=1`). The diagnostic receipt was `reason=failed`, attempts/sent/failures/skipped all zero. Screenshots `status-preparing.png` and `status-cancelled.png` show consent and terminal state. There were exactly two native announcement events during this accepted attempt, both status text; no counter announcement. This does not test active-run counter updates.

## Regression checks and review

The selector regression first failed because description was absent; the strengthened status test first failed because its label was absent. After the fix, all 81 Linux workspace/all-target tests passed. Tests cover all three buttons and all three themes independently of stable names, every existing lifecycle status in the minimum-window test, and the counter's non-live semantics. Linux and Windows MSVC workspace/all-target Clippy passed with warnings denied. The full-feature Linux release build passed with the existing linker deprecation warning. No Windows execution or new hosted CI is claimed for this correction. `graphify update .` completed; it reported its existing empty Cargo.toml node extraction and stale community labels, without a code-update failure.

Local review checked correctness against the native before/after receipt, Linux-only selector scope, retained names/values, status semantics, bounded work per frame and regression coverage. No dependencies or persistence/input behavior changed. No blocking review finding remains for this scoped fix; full migration acceptance remains open.

## Cleanup and remaining gates

`cleanup.json` confirms no private-runtime process remains. Private preferences stayed byte-for-byte equal to the original (SHA-256 `b79f252b85920a538447e5725e04b8a789ca297affe65c1db696ff12fcb40f65`), and host GNOME compositor PID 4780 remained active. No host reader/settings/audio or Windows VM state was changed.

Remaining reader work includes human listening, changed-value and popup/reverse navigation, other lifecycle announcements and active-count behavior, reference Ubuntu GNOME/KDE and Windows NVDA. Physical monitors, native timing/resource budgets and the paused software-RSS optimization remain separate gates. These results do not accept a release or retire C++.
