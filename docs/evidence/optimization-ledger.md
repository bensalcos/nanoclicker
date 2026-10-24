# Performance experiment ledger

Keep the original budgets in VALIDATION.md. A reduction in one internal cost does not close a whole-application or native-platform gate.

| Attempt | Evidence | Decision |
|---|---|---|
| Progress snapshots at 2 Hz instead of 10 Hz | Whole GUI active CPU fell from 19.67% to about 5.23% on software X11; immediate first pair and lifecycle transitions retained | Kept in b0d52d8; CPU/RAM still exceeded targets |
| Use `/proc/PID/io` syscr/syscw as native X11 send counts | 1001 pairs reported only 770 write-family calls; x11rb uses sendmsg, which these counters do not fully represent | Rejected as a socket-send metric; temporary counter code removed |
| Read per-mapping RSS | [Mapping sample](idle-mappings-baseline.csv): LLVM about 46.7 MiB, application executable about 10.9 MiB; 10-second stabilization is exploratory | Kept in the isolated harness, outside measured CPU windows; full RSS budget still uses 60-second runs |
| Remove explicit flush before checked X11 cookies | [Two native traces per build](x11-flush-optimization.md) save about one sendmsg and one ppoll per complete pair; both error checks and recovery retained | Kept after tests and real release GUI verification; no total-CPU claim from traced runs |

The mapping sample uses Mesa 26.2.2 and LLVM 22.1.8. Shared executable/library pages contribute to RSS; do not substitute PSS or private memory for the existing RSS target. The software driver dominates the current footprint, but the application's own mapped code is still a valid candidate for measured release-build optimization. Do not remove accessibility, fonts, backends or checks merely to make a number pass.
