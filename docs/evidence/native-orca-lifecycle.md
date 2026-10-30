# Native Orca lifecycle and progress — 2026-10-30

The unchanged `5f3e82c689ffd72b9fbfbb9436505aa4dd2f8869` full-feature WGPU application (SHA-256 `e3aafa12184acae9ac50246eae7ecf143d1db4324040ee6c7ba9d6c26c5f08f2`) ran in a new private Fedora 44 GNOME/Wayland session with Orca 50.2, GNOME Shell/Mutter 50.5, AT-SPI 2.60.7 and Speech Dispatcher 0.12.1. Private configuration set Count=6 and interval=1000 ms; these were fixture defaults, not edits to host preferences. Raw artifacts are under ignored `artifacts/wayland-a11y-lifecycle-20261030/`.

All input and reader/audio services were confined to that private compositor and buses. After explicit native RemoteDesktop consent, the virtual pointer was placed at (650,190), an inert part of the private product window. The actual click engine ran there while the application retained focus. This is an accessibility-state test; no independent input receiver was used and no delivered-pair or physical timing budget is accepted by this receipt.

## Results

The first session completed six producer attempts/sends, with zero failures/skipped periods. A second manual Start restored permission without another dialog, reached Running, and Escape stopped it after two attempts/sends, again with zero failures/skips. The diagnostics contain `reason=completed` and `reason=stopped` respectively.

Native polite `object:announcement` events and Orca's generated output both include:

- Preparing desktop input…
- Starting in 3, 2 and 1 · move the pointer to your target (full sequence in the first trial)
- Running
- Completed
- Releasing input…
- Ready after the explicitly stopped run

The stopped terminal state is presented as Ready; the diagnostic reason independently records stopped. Across these trials no live announcement contained the progress counter text (`pairs sent`). The counter was observed changing during the active run. This supplements the existing non-live-counter regression with native execution coverage; it does not accept every possible reader setting or lifecycle/error state.

Orca timestamps include Running at 19:57:57.645129, Completed at 19:58:02.682681, Running at 19:58:19.968409, Releasing input… at 19:58:21.883029 and Ready at 19:58:32.815384. These clocks are retained evidence, not a UI/Stop latency acceptance: headless rendering sometimes required a fresh pointer event to settle the display. Snapshots named completed/stopped can contain the preceding frame; terminal conclusions come from matched diagnostic, AT-SPI and flushed reader logs, not filenames.

## Preservation and limits

An early overview/focus setup attempt did not start a run and is excluded. A permission-reset helper was accidentally invoked through the private environment wrapper; its safety assertion rejected the call before any D-Bus operation. No reset is claimed. The granted permission remains only in the ignored private QA data directory, whose bus/session is now stopped.

`cleanup.json` verifies no private-runtime process remains and fixture preferences stayed byte-exact (SHA-256 `0bc09d550420b474b8bb9de8096393e084386f303ef94240018187d72a0521e0`). The historical host compositor PID 4780 was stale before cleanup verification; the current host compositor PID 6086 was checked independently. No host input, configuration or audio was targeted. No claim of comparing the old PID across this session is made.

Speech used a private dummy/null output backend. This verifies generated announcement text, not human listening. Reference Ubuntu/KDE, Windows/NVDA, other error/revocation announcements and physical performance/monitor acceptance remain separate. The 83-test/build/Clippy receipt for the unchanged source is reused; no repeated local broad tests were needed for this runtime-only follow-up.

Raw receipt integrity:

- `accepted.json`: `08d07ed6f7c188d9a9227c2695c25e234dea804159e67106f5d03a32d3d1aec9`.
- `events.jsonl`: `ea73edab4679f40aa0af9296d16b60584cda51fcb14f9e98111b7a773e128e03`.
- `orca-debug.log`: `854777e4a04a46a2fc916b5245fb1b8eb7793db738f6d760902b38ce88a7af93`.
- `runs.jsonl`: `f58967106e3f3aebbc986f8e4dac80d412a4dc7693cf32e8e95bbe54a562f0f8`.
- `cleanup.json`: `9b2884c74d851164a1562845569c469efcbea12f1ec5b826f26c63da36f151cb`.
