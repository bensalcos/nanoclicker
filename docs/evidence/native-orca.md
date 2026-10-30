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

Remaining reader work includes human listening, popup focus restoration, other lifecycle announcements and active-count behavior, reference Ubuntu GNOME/KDE and Windows NVDA. The continuation below supersedes the initial changed-value and reverse-navigation gaps. Physical monitors, native timing/resource budgets and the paused software-RSS optimization remain separate gates. These results do not accept a release or retire C++.

## Changed selections, popup traversal and reverse navigation — continuation

The unchanged application from `a68fe3fedf84fcd94b5c1e205385e1c1402e1224` was rechecked against SHA-256 `31b90cec3e2200ee6d0c9ddf321a034bc9fa033293f383ed318df0cec118c70e`. No application source changed, no rebuild or broad tests were repeated, and no hosted CI, Windows runtime or push occurred. Installed versions match the initial trial. New raw evidence is under ignored `artifacts/wayland-a11y-followup-20261030/`.

The new private session initially lacked the copied desktop identity file. That registration-error startup is excluded; after copying the existing QA desktop file and restarting the application, the normal controls were available. A private speech module/null sink and explicitly addressed QA controller were reused. Orca logged AT-SPI cache-signature warnings; these do not invalidate the independently captured focus/description and generated-speech receipts, but they are not a clean full reader-compatibility acceptance.

### Accepted results

`accepted.json` asserts the native role and name at every reverse step. From Theme, nine separate Shift+Tab presses reached Start button, Configure in desktop… button, Count entry (text 2), Count radio, Until stopped radio, Mouse button combo box, Interval entry (text 100), Save button and Theme combo box. Count entry and Count mode were checked separately by role.

Enter opened each selector. Arrow Down reached the options as native toggle buttons; Enter selected an option. Escape explicitly dismissed the popup. After Tab re-entry, the collapsed selector retained its stable name and exposed the new current description. The UI screenshots agree with those descriptions. No Save was used.

| Changed selection | Asserted native description on re-entry | Orca generated speech |
|---|---|---|
| Theme: System → Light | Light | `13:47:46.676541`, Light. |
| Theme: System → Dark (stabilized repeat) | Dark | `13:50:20.591468`, Dark. |
| Mouse button: Left → Right | Right | `13:52:27.196040`, Right. |
| Mouse button: Right → Middle | Middle | `13:53:11.196740`, Middle. |

Orca also named individual popup options and generated pressed/not-pressed toggle state text. This accepts the observed default and changed-value output text, not human listening or all native selection semantics.

### Rejected probes and retained limitation

Early Dark probes observed focus/state one frame behind and selected System rather than Dark. They remain exploratory in `steps.jsonl` and do not accept Dark. The stabilized repeat places two harmless pointer-motion events within NanoClick's header, 300 ms apart after each key release, before taking a fresh screenshot and querying AT-SPI. It then asserts the resulting selection after dismissal and re-entry; the validated Dark receipt and speech line above come from that repeat. This is a fixture stabilization technique, not a proven root-cause diagnosis or performance result.

The first script also incorrectly expected Enter to return focus immediately to the collapsed selector. In this native run the popup remained present after selection. Escape dismissed it but left AT-SPI focus on the frame; the next Tab returned to Theme, including when the dismissed selector was Mouse button. Re-entry can reach the correct value, but focus restoration to the originating selector remains an explicit usability follow-up. The accepted traversal does not close that issue.

### Preservation and verification

`cleanup.json` confirms zero diagnostic run rows, byte-exact original preferences with the same SHA-256 listed above, no remaining private-runtime process and the original host GNOME compositor still active. No click-engine session or host desktop injection was used. The changes were unsaved and the private application was terminated during cleanup.

Local evidence checks asserted all nine reverse roles/names, all four changed descriptions and all four generated speech values after Orca's buffered log was flushed. These checks supplement the earlier 81-test/build/Clippy evidence for the unchanged application; they do not rerun or extend Windows acceptance.

Raw receipt integrity:

- `accepted.json`: `f566a692176aa1bc020ff90dcdbcffdd7eb5592f9fc680db2498358230d90b1c`.
- `orca-debug.log`: `34c42086766914fda102e1d5107f31a340b4e82de0650115d11cdd8de2995e77`.
- `steps.jsonl`: `e2a665658de0e2f68acd03a7d5317188732e4a9eb3ba141759d36dcee02871fe`.
- `cleanup.json`: `6d378c7bd7be98fe225758da8f1d39675965cdc56ab4d7c22dc535365ab0854e`.
