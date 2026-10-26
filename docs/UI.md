# NanoClick Rust UI

Initial specification; visual prototyping and native QA are required before acceptance.

The new application is English only: controls, states, errors, tooltips and accessible names. It has no language selector or translation module. This replaces the C++ version's bilingual behavior.

## Main flow

One compact window, initially 520×600 logical units, minimum 420×520 with scrolling when content does not fit. The header contains the NanoClick brand, system/light/dark theme selector and Save button for explicit preference persistence. Three sections cover timing/button, stop condition and shortcut. The footer contains status, session counter and a persistent primary action. Respect OS decorations for moving, resizing and closing.

```text
NanoClick                              Theme
Ready                         Windows / X11 / Wayland

Timing
Interval [100          ] ms     ≈ 10 clicks/s
Button    [Left ▾]

Repeat
(●) Until stopped   ( ) Count [10        ]

Global shortcut
[F6                       Change]  Registered

Session: 0 pairs sent
[                 Start                   ]
```

While Running, the primary action becomes **Stop**, active-plan editing is disabled and theme changes remain available. Preparing shows the permission request and Cancel. Stopping remains visible until the engine acknowledges termination. Completing N pairs immediately restores controls and shows “Completed”. Faulted shows an understandable cause and recovery while preserving entered values.

Manual Start must not let the immediate first click reactivate the UI beneath the pointer. Prepare input and complete any desktop permission first; then instruct the user to move the pointer and show a cancelable 3-second countdown. Shortcut Start retains an immediate first click. The manual delay is a deliberate improvement over C++; measure engine latency from countdown completion, separately from the startup experience.

Do not add sidebar navigation, live CPS charts, continuous animations, tray integration, macros or multiple profiles in V1. Do not minimize on startup. Show the shortcut actually registered, conflict messages and a manual alternative when shortcut support is unavailable.

## Visual system

- Base spacing 8, margins 24, section gaps 20; corner radius 10 and subtle borders.
- Readable typography at 14–16 logical units, title 24, stable numeral alignment. Bundle a font only when its license permits redistribution.
- Dark theme: graphite background, softly raised surfaces and blue accent; equivalent light theme. Reserve red for Stop/errors without relying on color alone.
- Persistent labels, visible units, field-local errors and no silent correction of invalid input. Integer count and interval; tooltip for the target period and limits.
- Textual and visual status: ready, preparing, running, stopping, completed, faulted or unavailable capability. Do not present theoretical precision as observed CPS.

## Accessibility and QA

Tab/Shift+Tab follow reading order; focus is visible; Space/Enter activate only the appropriate control. Escape cancels shortcut capture or preparation and stops active input when the window has focus. Enter while editing a field must not Start. Text contrast ≥4.5:1, identifiable controls and English accessible names. Disable unnecessary motion.

Test 100%, 150% and 200% DPI, the minimum window, long messages, complete keyboard operation, NVDA on Windows and Orca on Linux. Enabling AccessKit alone does not prove these tests pass. Save captures and per-OS/theme results; accept appearance on an actual desktop, beyond compilation or headless tests.

Save explicitly persists theme, configuration and requested shortcut. Show its result in the window. Use a versioned file, temporary writes and safe OS-specific replacement; recover from corrupt files without automatically overwriting them. Use standard configuration directories. Never persist Running state, permissions as if still valid, or counters that resume clicks on launch.

## Implemented layout and accessibility update

The initial 520×600 size and 420×520 minimum remain. Secondary padding is 14×6 with a 30-unit minimum target; the primary action remains 46 units high. Redundant section spacing was removed so help fits at default size. A nonfloating scrollbar appears when needed at smaller sizes.

On Windows, the app checks the active monitor's logical size and native pixel scale and reduces the window height and repositions its outer frame when the full-monitor bounds leave too little room for the title bar and taskbar. It keeps the existing 520-point minimum and pinned action footer; smaller displays scroll the form body. The adjustment can also apply after a monitor or scale transition.

The body is created before the footer button for reading-order keyboard traversal; footer space is reserved from wrapped status/progress measurements. Fields/selectors have explicit English accessible names and values. Session status is a polite atomic live region; the counter is not live. Palette tokens have automated text/control/focus contrast checks. These implementation checks do not close native DPI/NVDA/Orca acceptance; see evidence/ui-accessibility.md and evidence/ui-scaling.md.

Preferences schema v2 stores the remembered count independently from repeat mode. Selecting Until stopped and saving keeps that count for a later finite run. Version-1 files migrate in memory (a null count becomes the default remembered count of 10); only an explicit Save writes v2. Unsupported schema versions remain protected and never expose damaged-file recovery.

Unsaved preferences are compared with the last loaded or successfully saved values, including invalid edits. Show “Unsaved changes” beside the desktop status below Save. Closing stops input first, then presents Save / Discard / Cancel. Save closes only after successful publication; a failure leaves the dialog and edits intact. Discard closes without writing, and Cancel returns to the stopped app. Escape stops input and cancels an open close dialog. The close dialog supports Alt+S (Save), Alt+D (Discard), and Escape (Cancel).

A successful Wayland shortcut assignment automatically remembers only the intent to rebind it, preserving the last saved form values. Startup rebinds the stable `toggle` identifier after that explicit desktop assignment; a fresh installation opens no shortcut dialog automatically. The desktop remains authoritative and may display configuration/consent again. This flag is not a permission grant and never resumes clicks.
