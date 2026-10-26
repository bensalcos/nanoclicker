# NanoClick Rust migration plan

## Objective

Preserve the C++ auto-clicker's functions and deliver a modern Rust UI for Windows and Linux, with fast Stop, stable scheduling and measured resource use. Implementation proceeds through small technical experiments and verifiable vertical slices. The current implementation and acceptance evidence are tracked separately: completed code does not imply completed platform acceptance.

Normative documents: [review](../docs/REVIEW.md), [architecture](../docs/architecture/RUST.md), [UI](../docs/UI.md), [validation](../docs/VALIDATION.md). Tasks and acceptance: [todo.md](todo.md). Current state: [STATUS.md](../docs/STATUS.md).

## Phases and milestones

| Phase | Tasks | Deliverable / exit condition |
|---|---|---|
| P0 — Reduce uncertainty | T01–T04 | Renderer selection and real Windows, X11 and portal evidence; explicit limitations |
| P1 — First complete capability | T05–T08 | UI Start/Stop for the left button on Windows/X11, cancelable waiting and CI |
| P2 — Parity and modern Linux | T09–T12 | Exact N, three buttons, shortcuts, English UI and integrated Wayland |
| P3 — Usability and recovery | T13–T15 | Accepted preferences, accessibility, failure handling and lifecycle |
| P4 — Optimization and delivery | T16–T18 | Accepted measurements, reproducible artifacts and user documentation |

Each milestone requires evidence against its criteria, not only checked tasks. Do not declare Linux complete based on X11 alone. If a portal is unavailable, record the environment blocker and continue independent tasks; do not automatically substitute privileged access.

## Current execution snapshot — 2026-10-26

The architecture, module boundaries, implementation phases and acceptance budgets are defined, and the planning-readiness audit below passes. Core scheduling, the desktop UI, Windows/X11/Wayland adapters, CI and preview packaging are implemented. The project remains in implementation and native acceptance; completion of source tasks does not close the platform milestones. The latest feature-graph review isolated Linux Glow from WGPU; the separated release passes the full X11 GUI flow and one quick profile, while RSS remains above the budget. The package command now reports renderer-specific runtime requirements and dependency inventory.

- P0 comparison protocol (T01) is complete. WGPU is now the provisional default on Windows and Linux, with an explicit Glow build alternative. Two current-source trials per renderer on the isolated AMD GPU display show materially lower WGPU startup and idle RSS with near-equal active CPU. Software rendering shows a small Glow CPU advantage; final renderer acceptance remains open because native Vulkan compatibility, DPI/accessibility and the complete reference-machine resource matrix are not complete.
- P1's basic Windows/X11 application flows exist and have functional evidence. Windows VM finite-count, long-wait lifecycle and exact-current-source manual Start/countdown receipts are available. Broader failure and native acceptance remain open.
- P2 parity is implemented for X11 and portal adapters. A read-only 2026-10-26 inventory confirms GNOME Shell 50.5 with RemoteDesktop v2/POINTER and GlobalShortcuts v1 available on the host; real GNOME consent, assigned-trigger activation, target receipt and KDE sessions are still required. P3/P4 retain accessibility, lifecycle, resource, timing and clean-install gates.
- Use the user's isolated Windows 11 VM for Windows functional/GUI checks. Its 2 vCPU, 4 GiB and Basic Display Adapter configuration is below the performance reference; it cannot close reference performance budgets.

The authoritative task order is in [todo.md](todo.md). Current active work is T16: investigate the software-rendering RSS excess and complete cross-renderer/backend performance acceptance; two full trials per cadence support the 1 Hz CPU comparison, the focused X11 observation confirms legible once-per-second feedback, two current-source software and physical-GPU trials compare Glow/WGPU, and the WGPU-default Linux bundle passes the restricted Ubuntu 24.04 GUI flow. A provisional Vulkan-only Linux WGPU feature probe compiles for Linux and the Windows MSVC target and passes Linux GUI acceptance; it reduces the executable by 2.49%, but the profile comparison is incomplete and its Windows VM execution remains pending. Next measure remaining performance cases and continue Windows functional QA in the user's VM, close remaining shortcut/error/recovery and DPI/accessibility cases, and maintain separate Wayland acceptance.

## Planning readiness audit — 2026-10-26

Planning deliverables requested for this rewrite are present and cross-linked:

- [x] Original application inventory, parity scope and technical risks: [review](../docs/REVIEW.md).
- [x] Rust workspace, crate/module boundaries, execution contracts and platform adapters: [architecture](../docs/architecture/RUST.md).
- [x] English UI flow, theme, keyboard, scaling, preference and accessibility requirements: [UI specification](../docs/UI.md).
- [x] Windows/Linux environment matrix, numeric budgets, verification strategy and distribution constraints: [validation contract](../docs/VALIDATION.md).
- [x] Ordered P0–P4 phases, milestones, dependencies, risks and decision gates: this plan.
- [x] Task-level scope, acceptance criteria, dependencies, verification steps and likely files: [backlog](todo.md).
- [x] Current evidence, unresolved acceptance gates and next actions: [acceptance matrix](../docs/ACCEPTANCE.md), [status](../docs/STATUS.md) and [resume handoff](../docs/RESUME.md).

**Planning is complete enough to guide implementation without another design phase.** Renderer, desktop, reference-hardware, packaging and legacy-retirement decisions remain explicit implementation/acceptance gates; they are not silently assumed complete by this audit.

## Order and dependencies

```text
T01 ─ T02/T03/T04 (independent experiments)
 └─ T05 ─ T06 ─ T07 ─ T08 ─ T09 ─ T10 ─ T11
             T04 + T08 + T10 ─ T12
             T11 + T12 ─ T13 ─ T14 ─ T15 ─ T16 ─ T17 ─ T18
```

T05 can proceed once the UI approach in T02 is selected; T07 requires T03. P0 experiments are disposable and must not freeze APIs for convenience. Do not maintain two engines connected through FFI. Keep the C++ reference buildable until final acceptance. Afterwards, remove legacy source/build files in a separate change that references the historical commit.

## Risks and mitigations

| Risk | Treatment |
|---|---|
| Incomplete or slow portals | Test GNOME/KDE early; report explicit capabilities and per-backend measurements |
| Excessive renderer cost or GPU failures | Compare glow/wgpu, select one and retain explicit review criteria |
| Attractive but inaccessible UI | Keyboard navigation and AccessKit from the prototype; NVDA/Orca before acceptance |
| Optimization harms energy use or Stop | Cancelable waiting without spinning; measure CPU, jitter and Stop together |
| Unnecessary Rust complexity | Three crates, a standard-library worker, executors only where platform APIs require them |
| Windows VM is below reference resources and lacks the full hardware/display matrix | Use the user's VM for functional and GUI validation; reserve performance, monitor-transition and reader acceptance for suitable reference installations |
| Target application rejects events | Report backend semantics without promising receipt or bypassing restrictions |

## Decisions and remaining evidence

- Toolchain, MSRV and target dependencies are recorded in the workspace manifests and lockfile; Rust 1.95 compatibility is checked in CI.
- The provisional renderer default is wgpu on Windows and Linux; an explicit Linux-only `--no-default-features --features glow` build remains available, and Windows always selects WGPU. T02/T16 require remaining native Vulkan compatibility, accessibility and performance evidence before final acceptance. Reconsider the UI framework only if those criteria cannot be met.
- Cancelable native waiting and portal lifecycle handling are implemented; T03/T04 still require the corresponding desktop and timing acceptance.
- The initial comparison protocol and available-system inventory are recorded in T01. Complete the KDE desktop/version matrix in T04 and the reference-machine performance matrix in T16 before claiming full platform coverage.
- Measure the unchanged validation budgets in T16. Any proposed budget change requires a documented decision supported by results.

Technical experiments can proceed without asking the user to resolve routine implementation choices. Signing/publication information and license attribution are distribution concerns. The MIT attribution was resolved through the user's explicit delegation; publishing a release remains a separate action from producing a preview.
