# Review of the original project

Static review: 2026-10-23. Base: `3fd5cd8`. The Windows binary was not executed and performance was not measured. The tree was clean at the start of that review. That original revision had no versioned tests or CI automation.

## Inventory and parity

| Component | Original implementation | Behavior to preserve |
|---|---|---|
| Startup | `Autoclicker.cpp`, Win32 and common controls | One desktop window |
| Engine | `AutoclickerEngine.{h,cpp}`, persistent thread | Immediate first click, millisecond interval, current pointer position |
| Input | `SendInput`, separate down/up calls | Left, right and middle buttons |
| Stop | Atomic flag and `int` count | Manual Stop or completion after N whole clicks |
| UI | `MainWindow.{h,cpp}` | Configuration disabled while running |
| Shortcut | `RegisterHotKey`, default F6 | Toggle Start/Stop outside focus |
| Language | ES/EN strings inside the window | The rewrite is English only by user decision |
| Resources | `.rc`, two icons, menu | NanoClick identity |
| Build | `.vcxproj`, `.slnx` | Windows only, toolset v145, C++20 |

Defaults: 100 ms interval, count 10, infinite mode, left button, Spanish and F6. There is no persistent configuration to import. Fixed positioning, macros, double-click mode, tray integration and an updater are not implemented.

## Findings

1. **Concurrency:** `Start()` writes configuration fields without the mutex used by the worker; rapid Stop/Start cycles may overlap reads from the previous run. The condition protocol also fails to protect all predicate transitions with the same mutex, risking lost notification. Redesign ownership and synchronization rather than translating this scheme into Rust.
2. **Cancellation:** `Stop()` does not wake `Sleep(clickRate)`. Shutdown joins the worker and may block for the remaining interval, which the UI allows up to 999999 ms.
3. **Automatic completion:** the worker sets `isRunning=false` before the callback; `StopAutoclicker()` returns when already stopped without calling `UpdateControlStates()`. Controls may remain disabled.
4. **Injection:** both `SendInput` return values are ignored. Failed attempts are counted; an accepted down followed by a failed up requires explicit recovery.
5. **Timing:** `Sleep(interval)` adds injection cost to the period and accumulates drift. `timeBeginPeriod(1)` remains active for the process lifetime and its result is ignored. Clock resolution does not guarantee precision.
6. **Shortcuts:** initial registration and changes ignore errors; the previous shortcut is removed before checking the new one. The control is polled every 500 ms. Held-key repeat suppression is missing.
7. **Validation:** `_wtoi` does not distinguish invalid input; a zero interval silently becomes one, and count zero finishes after the first attempt. The new version explicitly rejects invalid values.
8. **UI and resources:** coordinate-based layout and fixed minimum height lack DPI, keyboard and screen-reader evidence. The class brush has no visible release. The resource menu declares About, but the window implements no corresponding action.
9. **Documentation/build:** README names `.sln` although the project has `.slnx`, says C++17 although it uses C++20, and recommends VS2022 although it configures v145. Zero-lag and precision claims lack measurements.
10. **License:** the initial MIT notice contained `[year] [fullname]`. The user delegated the decision; attribution became `Copyright (c) 2026 bensalcos`, using the repository owner's identifier.

## Migration implications

Use a small modular rewrite without FFI into the C++ engine. Retain the original version as a reference until parity is validated. A language change alone does not prove improvement over C++: measurable objectives are lower drift, Stop latency and resource use, alongside broader compatibility.
