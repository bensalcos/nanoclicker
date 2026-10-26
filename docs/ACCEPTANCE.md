# Migration acceptance matrix

Reviewed against the repository plan on 2026-10-26. The Rust implementation and preview packages are available. **The migration is not accepted as a release.** Task checkboxes in [the backlog](../tasks/todo.md) retain their full cross-platform scope.

| Requirement | Evidence available | Evidence still required |
|---|---|---|
| Architecture, modules, phases and tasks | [Architecture](architecture/RUST.md), [plan](../tasks/plan.md), [backlog](../tasks/todo.md) | Final decisions remain subject to native results |
| Core correctness and cancellation | Deterministic scheduling/concurrency tests; retained run identities; finite counts; overflow/error handling | Adapter-specific native acceptance below |
| Windows 11 | MSVC build/unit/package checks; [Windows 11 VM startup, Count=1/10/1000 per button, native preference Save/relaunch/discard, F6 outside focus, long-wait Stop/restart/active close, diagnosed capture-position setup, and current `68cd895` focused-target F6 Count=10 receipt](evidence/windows-vm-functional.md) | Matching manual-countdown completion receipt, partial-insertion/release failures, shortcut layouts/conflicts, resource/timing budgets, DPI/monitor transitions and NVDA |
| Ubuntu X11 | Actual product GUI and capture on private Xvfb; extracted WGPU-default release bundle passes the restricted Ubuntu 24.04 flow for three buttons, N=1/7/1000, shortcuts, preferences, manual preparation and lifecycle cases | Hardware Vulkan drivers, reference desktop/layout behavior, Orca, real monitor transitions and complete native performance protocol |
| GNOME Wayland | Production portal adapters; thirteen private-D-Bus lifecycle cases; passive native capture target prepared | Real consent/denial/revocation, desktop-assigned trigger, native Wayland target delivery, resource/timing and reader/DPI checks |
| KDE Wayland | Linux adapter/contract checks | Distribution/backend versions and the same native flows as GNOME |
| UI and preferences | English UI; accessible-name/keyboard/contrast tests; synthetic scaling; actual X11 Save/relaunch and finite completion; Windows 11 Save/relaunch restores all fields and discards an unsaved interval edit | Full native DPI/reader matrix, invalid-file recovery/error flows and remaining Windows manual/rapid-state transitions |
| Performance | Private GPU cases meet recorded startup/CPU/RSS targets; X11 timing and Stop samples pass at their recorded commits; two full UI trials per cadence captured 6001 pairs each; current-source 1 Hz software and physical-GPU Glow/WGPU comparisons and feedback observation are recorded | Software RSS remains above budget; validate feedback on native renderers/backends and accessibility paths; verify Vulkan compatibility across supported Linux installations; the Windows VM is below reference resources; full native protocol per backend and final renderer decision remain open |
| Distribution | WGPU Linux preview executes in restricted Ubuntu; renderer-aware Glow preview has a matching feature/license inventory and OpenGL/EGL README and passes isolated X11 GUI acceptance; Windows archives pass structural verification and the prior app/QA target execute in the user's Windows 11 VM | Clean reference installations, current-source Windows artifact execution and the remaining native release matrix |
| Documentation and legacy retirement | README, changelog, evidence index, native QA and rollback guide | Final accepted matrix and separate C++ retirement after all gates close |

The [validation contract](VALIDATION.md) defines unchanged numeric budgets and conditions. GPU, software, mock, CI and native-desktop evidence are separate. A historic successful measurement does not accept a later source revision automatically. The C++ performance baseline remains unavailable in this Linux environment, as explicitly allowed and recorded in [the baseline](evidence/baseline.md).

## Current next actions

1. Continue the [Windows 11 VM QA](evidence/windows-vm-functional.md), first obtaining matching producer and capture evidence for manual countdown completion; cancellation already passes and recent completion probes are inconclusive.
2. Close remaining Windows failure, shortcut, preference-recovery, DPI/accessibility and stability cases in the VM. Use reference hardware for performance budgets the constrained VM cannot accept.
3. Run native GNOME/KDE portal consent, target delivery, revocation, accessibility and timing flows; Xvfb and D-Bus mocks do not substitute for compositor acceptance.
4. Close the software-rendering RSS finding and rerun per-backend measurements, verify WGPU/Glow compatibility and final artifacts on reference installations. Retain the C++ reference and label bundles as previews until those gates pass.

This migration session has performed no merge or release publication. The authorized draft PR remains dependent on a GitHub connection with PR-write permission: the latest creation attempt was explicitly rejected by GitHub's `CreatePullRequest` permission check. Publishing the branch did not grant that API capability.
