# Migration acceptance matrix

Reviewed against the repository plan on 2026-10-25. The Rust implementation and preview packages are available. **The migration is not accepted as a release.** Task checkboxes in [the backlog](../tasks/todo.md) retain their full cross-platform scope.

| Requirement | Evidence available | Evidence still required |
|---|---|---|
| Architecture, modules, phases and tasks | [Architecture](architecture/RUST.md), [plan](../tasks/plan.md), [backlog](../tasks/todo.md) | Final decisions remain subject to native results |
| Core correctness and cancellation | Deterministic scheduling/concurrency tests; retained run identities; finite counts; overflow/error handling | Adapter-specific native acceptance below |
| Windows 11 | MSVC hosted build/unit tests; independently verified preview ZIP/PDB | Actual SendInput capture, shortcuts/layout/conflicts, close/resource stability, timers, GUI/DPI/NVDA and clean reference execution |
| Ubuntu X11 | Actual product GUI and capture on private Xvfb; restricted Ubuntu container; three buttons, finite plans, shortcuts, preferences and lifecycle cases | Reference desktop/layout behavior, Orca, real monitor transitions and complete native performance protocol |
| GNOME Wayland | Production portal adapters; thirteen private-D-Bus lifecycle cases; passive native capture target prepared | Real consent/denial/revocation, desktop-assigned trigger, native Wayland target delivery, resource/timing and reader/DPI checks |
| KDE Wayland | Linux adapter/contract checks | Distribution/backend versions and the same native flows as GNOME |
| UI and preferences | English UI; accessible-name/keyboard/contrast tests; synthetic scaling; actual X11 Save/relaunch and finite completion | Full native matrix, remaining saved fields/error flows, large-count/rapid-state transitions |
| Performance | Private GPU cases meet recorded startup/CPU/RSS targets; X11 timing and Stop samples pass at their recorded commits | Software CPU/RSS budgets remain unmet; latest/full native protocol per backend and final renderer decision remain open |
| Distribution | Linux preview executes in restricted Ubuntu; archive hashes/notices/symbols verified; Windows archives structurally verified | Windows desktop execution, clean reference/native matrix and accepted final artifacts |
| Documentation and legacy retirement | README, changelog, evidence index, native QA and rollback guide | Final accepted matrix and separate C++ retirement after all gates close |

The [validation contract](VALIDATION.md) defines unchanged numeric budgets and conditions. GPU, software, mock, CI and native-desktop evidence are separate. A historic successful measurement does not accept a later source revision automatically. The C++ performance baseline remains unavailable in this Linux environment, as explicitly allowed and recorded in [the baseline](evidence/baseline.md).

## Current next actions

1. Confirm the narrow ESRCH observer fix in hosted CI. Source `d7230ce` passed Windows/renderer/MSRV and the Linux product GUI, capture target and startup checks, then failed on a disappearing `/proc` thread. Earlier source `88bd9b4` passed all four jobs. Both observer follow-ups are recorded in [fixture evidence](evidence/x11-owned-display.md).
2. Run the [native QA protocol](NATIVE_QA.md) on an available real desktop. Windows 11 and GNOME/KDE results cannot be supplied by Xvfb, D-Bus mocks or Windows Server unit tests.
3. Use those results to close reproducible defects and the renderer/per-backend budget decision. Do not lower budgets or remove functionality to turn missing evidence into acceptance.
4. Verify final artifacts on the reference installations, then complete migration review and legacy retirement. Until then retain the C++ reference and label bundles as previews.

This migration session has performed no merge or release publication. The authorized draft PR remains dependent on a GitHub connection with PR-write permission: the latest creation attempt was explicitly rejected by GitHub's `CreatePullRequest` permission check. Publishing the branch did not grant that API capability.
