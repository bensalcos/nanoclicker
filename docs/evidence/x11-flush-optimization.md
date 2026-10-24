# X11 checked-pair batching

The input adapter already queued down/up together and checked both cookies. Its additional explicit flush sent that pair before `down.check()` added and flushed the synchronization request. Removing the explicit flush allows x11rb to send the pair and synchronization together. Both cookies remain checked, and pending-button recovery is unchanged.

The pinned x11rb 0.13.2 implementation was inspected: `RustConnection::check_for_raw_error` inserts the synchronization when needed and flushes pending bytes before checking the response. It is not necessary to flush first to retain error detection.

## Native syscall evidence

Fedora 44, Mesa 26.2.2, LLVM 22.1.8, Rust 1.97.1; private Xvfb, software glow renderer, 10 ms click period and 2 Hz progress. Strace 7.2 was extracted into `/tmp` from Fedora's signed `strace-7.2-1.fc44.x86_64.rpm` after `rpm -K` verified signatures/digests; no system package or new application dependency was installed.

Each run discarded a warmup, stabilized/observed idle for 10 seconds each, and captured input for 10 seconds. Two runs per executable use `strace -f --seccomp-bpf -c -e trace=sendmsg,recvmsg,poll,ppoll`. The syscall summaries include startup and idle as well as the active phase. Raw local summaries are under `artifacts/performance/`; [structured counts](x11-flush-syscalls.csv) are retained here.

| Variant | Trial | Complete pairs | sendmsg | ppoll |
|---|---:|---:|---:|---:|
| Explicit flush | 1 | 1001 | 2169 | 6058 |
| Explicit flush | 2 | 1001 | 2169 | 6058 |
| Checked pair only | 1 | 1001 | 1169 | 5057 |
| Checked pair only | 2 | 1000 | 1168 | 5052 |

The consistent difference is approximately one send and one poll per pair. The one-pair observation difference in the last run accounts for five polls and one send. This is a measured transport-work reduction, not evidence that total UI CPU or deadline lateness meets the product budget.

The wrapper process is strace, so the harness's `/proc` CPU/RSS fields in these tracing runs describe the tracer and **must not** be used as NanoClick performance results. Tracing also perturbs scheduling. Nonblocking `recvmsg` error counts include normal EAGAIN flow, not failed input pairs. Use untraced runs for CPU, RSS and timing claims.

Executable SHA256:

- Baseline: `39cb222a455908f2979c8084dc71cfcaf6142d5a9d16d57b93ea51cb70fcb965`.
- Candidate: `f8305be9ca7709151b6e43ba087f90f2a70feaa2a00b25354cd70a1b36af8e9d`.

Verification after the change: 38 Rust tests and Linux Clippy pass; isolated X11 received ten ordered pairs for Left/Middle/Right and passed held-key/conflict checks. The actual release GUI passed shortcut outside focus, minimum-size Start, manual countdown, no input after Stop and clean close. The optimization is retained for its repeatable syscall reduction; CPU/RAM gates remain open.

## Untraced whole-application check

A full release run after the change ([data](ui-noflush.csv)) measured startup p95 128.262 ms, idle RSS 106.918 MiB, zero observed idle CPU ticks, and active CPU 5.4499% of one core over 60 seconds. It received 6001 complete pairs with 1 ms p99/max interarrival absolute error. The CPU and memory budgets still fail.

Earlier UI-polish measurements were 73.712 ms startup p95 and 5.3499% active CPU. Startup before any clicking cannot be improved or regressed by this flush change; the shared host exhibits substantial run-to-run variation. These samples do not establish a total-CPU or startup gain. The retained benefit is the repeated native syscall reduction, not an assertion that the full application is now within budget.
