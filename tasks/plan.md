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
| No Windows desktop available | Continue core tests/CI; native desktop support remains pending evidence |
| Target application rejects events | Report backend semantics without promising receipt or bypassing restrictions |

## Decisions and remaining evidence

- Toolchain, MSRV and target dependencies are recorded in the workspace manifests and lockfile; Rust 1.95 compatibility is checked in CI.
- Glow remains the provisional default; wgpu is an optional comparison build. T02/T16 require the remaining native graphics and performance evidence. Reconsider the UI framework only if the acceptance criteria cannot be met.
- Cancelable native waiting and portal lifecycle handling are implemented; T03/T04 still require the corresponding desktop and timing acceptance.
- Complete the KDE/version and reference-machine matrix in T01 before claiming full platform coverage.
- Measure the unchanged validation budgets in T16. Any proposed budget change requires a documented decision supported by results.

Technical experiments can proceed without asking the user to resolve routine implementation choices. Signing/publication information and license attribution are distribution concerns. The MIT attribution was resolved through the user's explicit delegation; publishing a release remains a separate action from producing a preview.
