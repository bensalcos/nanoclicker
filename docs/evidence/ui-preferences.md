# GUI preferences Save and relaunch

Date: 2026-10-24. Production source: `0c39233`; release executable SHA-256 `317322b93cd353775828192b7b5f7f12ef14630f85fba8d266488fa64b845c6f`.

The X11 GUI fixture now operates the actual Save button and interval field. It uses only a fresh private XDG_CONFIG_HOME and the private Xvfb display created by `scripts/verify-ui-x11.sh`.

1. Launch the release app, click Save, and wait for the preferences file.
2. Edit the interval to 37 ms, click Save, and check the versioned JSON and six expected fields.
3. Edit to 91 ms without saving. Start and stop through F6, capture twelve press timestamps and complete ordered pairs, and verify the observed median is in 70–115 ms. This establishes that the unsaved edit reached the actual controller.
4. Close normally and verify the saved bytes are unchanged.
5. Relaunch, focus the capture target and observe 500 ms without input. Explicitly Start, capture twelve timestamps and ordered pairs, and verify a 25–65 ms median, distinguishing the saved 37 ms plan from both the 91 ms unsaved edit and the 100 ms default.
6. Remove only the fixture's known preferences file, click Save again, wait for a new publication and verify the original bytes. A stale matching file cannot satisfy this check.

The captured restored median is 37 ms. The broad ranges above discriminate plans; they are not deadline-lateness or performance acceptance thresholds. Theme is explicitly forced dark by the shared launcher, so this case does not prove restoration of a saved theme. Changed mouse-button/count/shortcut restoration, invalid-file GUI recovery and native Windows desktop Save remain open.

The fixture initially missed the interval because its targeting raced the newly inserted Save-status line. It now waits for that frame and for text focus before sending keystrokes; no production delay or retry was added. The runner removes its own fresh configuration directory after the test. `serde_json` is a Linux development dependency for reading the saved document, already present at the locked version; runtime dependencies are unchanged.

Reproduce with:

```sh
cargo build --release --locked -p nanoclick
scripts/verify-ui-x11.sh target/release/nanoclick
NANOCLICK_UI_STRESS_CYCLES=20 scripts/verify-ui-x11.sh target/release/nanoclick 2
```

Local release flows pass at 100% and 200% scale, including the existing manual Start, complete-pair, minimum-window, Stop and active-close checks. The 200% run also includes twenty measured stress cycles. Xvfb proves this isolated X11 flow; native monitor/DPI, Wayland and Windows acceptance remain separate. Hosted run 40121663794 at e244efd also passes the release preferences case; its downloaded GUI log was inspected. See [CI evidence](ci.md).
