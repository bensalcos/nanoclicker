# Native Wayland protected preference files and paths — 2026-10-29

## Artifact and scope

The full-feature Linux WGPU product is the unchanged `cc7bea099a679dcdef540585a4282dd805d52a67` artifact, SHA-256 `260f647d4752f99dd3c1489d484f78704df192d162795a8789feaf787015572d`. Live version checks confirmed GNOME Shell/Mutter 50.5 and xdg-desktop-portal 1.22.1. The native Fedora 44 Wayland session has a 1280×800 virtual monitor and private D-Bus, runtime, config, data and cache directories. It reuses the [independent private Clutter controller](wayland-native-revocation.md), not the user's desktop or a physical input device.

All fixtures, receipts and screenshots are ignored local artifacts under `artifacts/wayland-protected-20261029/`. The tested paths belong only to that lab. No click run was started: all five per-process diagnostic files remain empty, and the inspected UIs show Ready/zero pairs. No source build, code change, broad test/CI rerun, Windows VM action, push, merge or release was performed for this continuation. These are private Fedora GUI receipts, not Ubuntu/KDE or complete migration acceptance.

## Future-version protection

The fixture contains only schema version 99 and an extra future field, without any current-schema preference fields. The app must recognize an unsupported version before strict current-field decoding. Initial launch showed `Preferences version 99 is not supported. The file was preserved.` and offered no damaged-file recovery action (`future-initial.png`).

The real Interval field was edited from 100 to 37, then Save was clicked. The UI retained Unsaved changes and 37, and displayed the protected-file error (`future-save-accepted.png`). No unique backup or replacement was created. The original and retained SHA-256 are both `0bea5401e4bfe21acb7fa97012a8f4647761dc391d46b52b805ec2f9e6bc2739`.

Native close decisions also preserve the file and edits:

- Alt+F4 opened Save/Discard/Cancel. Alt+S attempted Save, retained the dialog/error and kept the process alive (`future-close-save-failed.png`).
- Escape canceled close, retained 37 and Unsaved changes, and kept the process alive (`future-cancel-retains.png`).
- Alt+F4 followed by Alt+D exited without replacing the file. A new process again showed unsupported version 99, safe Interval=100, Ready and zero sent pairs; the unsaved 37 was not persisted (`future-reload.png`). Closing that unedited process also left the original bytes unchanged.

`future-result.json` and five entries in `future-checks.jsonl` record hashes, no backups and live/exited process state at the relevant boundaries. The first attempted edit/Save moved into GNOME's overview, so `future-save-protected.png` and its unchanged-file assertion do not establish a Save attempt. That setup attempt was rejected; the accepted action was repeated after returning to the actual application and is separately pictured in `future-save-accepted.png`.

## Existing parent path is a regular file

After closing the future-version process, its original directory was preserved separately. A regular sentinel file was placed at the exact location where NanoClick expects its preferences directory. Initial launch reported `Not a directory (os error 20)` and offered no recovery action (`blocked-initial.png`).

Editing Interval to 37 and clicking Save showed the protected-file error while retaining the edit. The sentinel remained a regular file with identical bytes, SHA-256 `c154d350495305237a4bafa9332a5ab019ddb14382b635b4c6a4fc85d7a21793`. No directory, preferences file or backup replaced it (`blocked-save-protected.png`).

Alt+F4/Alt+S retained the failed-Save dialog, live process and original sentinel. Escape returned to editing with 37 intact. Alt+F4/Alt+D then exited without changing the obstruction. See `blocked-close-save-failed.png`, `blocked-cancel-retains.png`, `blocked-result.json` and five `blocked-checks.jsonl` entries. The recorded matching path list remains exactly the sentinel path throughout these checks.

## Explicit path repair and relaunch

With the blocked process stopped, the test moved the sentinel aside without altering its bytes and created a valid schema-v2 preferences directory. A fresh app loaded Interval=100 and showed Ready without a protected-file error (`repaired-initial.png`). Editing to 37 and clicking Save succeeded, cleared the dirty indicator and showed `Preferences saved.` (`repaired-save.png`).

The written schema-v2 file has SHA-256 `527667468edeadb8405278e7e5579c1b9c4386ee58895718f47071929a3636fe`. A further fresh process loaded 37 and remained idle with zero sent pairs (`repaired-reload.png`). No backup was generated. `repaired-save.json` and `repaired-result.json` record the saved fields, preserved original fixtures and empty run logs. This checks the documented move-aside/restart route for this parent obstruction; it does not turn a version-99 file into a recoverable damaged file.

## Cleanup and remaining acceptance

`cleanup.json` confirms that no process with the private runtime remains and the user's compositor PID 4780 remains running. The future fixture was restored byte-for-byte to its initial preference pathname. The obstruction sentinel remains byte-exact at `blocked-parent-preserved.bin`, and the successfully saved file is retained in `repaired-preferences-directory/`. All QA configuration and the controller extension remain confined to the ignored lab.

The same runtime now has native future-version protection, parent obstruction, failed Save-on-close, Cancel/Discard and repaired-path persistence receipts, in addition to the [earlier native Save/backup denial and damaged-file recovery](wayland-native-revocation.md). Reference Ubuntu GNOME/KDE, other path/error variants, persistent-permission reset, readers, physical monitor/DPI transitions and full timing/resource budgets remain separate gates. The C++ reference and preview status are retained; general software-RSS optimization remains paused.
