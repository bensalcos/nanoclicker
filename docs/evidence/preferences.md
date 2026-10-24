# Versioned preferences

Schema v1: interval, button, finite/infinite repetition, theme and requested shortcut. Save is explicit; loading occurs at startup without persisting active sessions, counters or current permissions.

Location uses directories 6.0.0 and standard OS directories. Write a temporary file in the same directory, sync_all its content, publish with tempfile 3.27.0 `persist` (atomic replacement under its contract), and sync the directory on Unix. Publication failure does not remove the destination. This does not promise persistence through every hardware/filesystem failure.

Reads are limited to 64 KiB. Unknown versions, additional fields and invalid values are rejected. Read failure uses defaults for the session and protects the original from replacement; the user can move it aside and restart. There is no automatic overwrite.

Initial local evidence: six Linux tests passed for roundtrip, valid replacement, corruption/future version, absence of active state, path failure, actual persist failure against an incompatible destination and invalid size/values. The then-current full suite passed 22 tests. Windows workspace Clippy passed, but native file replacement was untested at this stage. Later native filesystem and GUI evidence is recorded in windows-preferences.md and ui-preferences.md.

Capture and GUI acceptance scripts use temporary XDG_CONFIG_HOME, avoiding personal preferences during tests/captures.
