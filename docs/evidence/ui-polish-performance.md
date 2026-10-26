# Release check after accessibility/layout changes

Date: 2026-10-24. Application source: `fb36ed5`, default glow renderer, standard release profile. Input executable SHA256: `39cb222a455908f2979c8084dc71cfcaf6142d5a9d16d57b93ea51cb70fcb965`.

Command: `LIBGL_ALWAYS_SOFTWARE=1 scripts/measure-ui-x11.sh target/release/nanoclick`. Same host, private Xvfb/Mesa llvmpipe setup and 2 Hz progress publication as the earlier glow runs. No compiler or heavy verification process ran during measurement. The helper now derives the primary-action position from window geometry and explicitly fixes X11 scaling to 100%; startup observation still polls every 5 ms.

| Measurement | Result | Initial target |
|---|---:|---:|
| Warm startup p95, 30 starts | 73.712 ms | ≤500 ms |
| Idle CPU, one core, 60 s | 0.0000% | ≤1% |
| Idle RSS | 106.789 MiB | ≤100 MiB: **unmet** |
| Active CPU at 100 pairs/s, 60 s | 5.3499% | ≤5%: **unmet** |
| Active RSS | 106.914 MiB | Recorded separately |
| Complete received pairs | 6001 | 60 s observation |
| Interarrival absolute error p99 | 1 ms | Not deadline lateness |
| Interarrival absolute error maximum | 1 ms | Observed only |

[Numeric record](ui-glow-accessibility.csv). Zero idle CPU means no additional process tick was observed in that window. CPU uses aggregate process ticks with one core as 100%; RSS includes software graphics libraries.

The pre-polish matched-source glow run observed 84.481 ms startup p95, 107.230 MiB idle RSS and 5.2331% active CPU. This new single run has lower startup/RSS and slightly higher active CPU. Neither is a repeated-trial statistical comparison; do not claim a proven CPU improvement or no regression. The active CPU and memory gates remain open, without changing thresholds or removing accessibility features to make the measurements pass.

Next performance work needs profiling and measured changes, followed by the same functional/keyboard checks. Native input Stop, scheduler deadline lateness, loaded operation, physical GPU behavior, Windows and Wayland remain separate acceptance requirements.

## Current-source progress-refresh experiment — 2026-10-25

The current application source uses the same X11 2 Hz progress publication as `715b562`. A fresh full baseline with `scripts/measure-ui-x11.sh target/release/nanoclick` recorded startup p95 75.034 ms (30 launches), idle CPU 0.0000%, idle RSS 107.137 MiB, active CPU 4.6998%, active RSS 106.879 MiB and 6001 captured pairs over 60 seconds. This one run is below the CPU budget and above the unchanged RSS budget; it does not replace the earlier above-budget samples or establish repeated-trial acceptance. The measured thread sample attributed work to the NanoClick UI thread, click worker and llvmpipe threads.

A 1 Hz candidate changes only the progress publication interval to 1000 ms; first-click and terminal/lifecycle wakeups stay immediate. Two 10-second `--quick-profile` samples measured 2.5999% and 2.7996% active CPU, each with 1001 matching pairs and p99 interval error 0–1 ms. A 2 Hz 10-second quick sample measured 5.0981% CPU with 1001 pairs and p99 error 1 ms. RSS stayed within the same approximate 107.2–107.4 MiB range. These short samples are exploratory. Candidate binary SHA-256: `39b8aab3673ea4deb6523d3025e686d824d5e9750e5ba890064720cb8f926d84`.

The full candidate protocol then completed after two earlier attempts returned `No such file or directory` before active-phase output. The first 1 Hz run recorded startup p95 74.595 ms (30 launches), idle CPU 0.0000%, idle RSS 107.309 MiB, active CPU 2.8165%, active RSS 107.359 MiB, 6001 matching pairs over 60 seconds, p99 interval error 0 ms and maximum 1 ms. The 2 Hz full baseline immediately preceding it recorded 75.034 ms startup p95, 107.137 MiB idle RSS, 4.6998% active CPU, 6001 pairs and p99/max interval error 1/2 ms.

A second complete 1 Hz trial on the same candidate recorded startup p95 74.424 ms, idle CPU 0.0000%, idle RSS 107.059 MiB, active CPU 2.9833%, active RSS 107.102 MiB, 6001 matching pairs, p99 interval error 0 ms and maximum 1 ms. A second full 2 Hz control run then measured startup p95 74.462 ms, idle CPU 0.0000%, idle RSS 107.301 MiB, active CPU 4.9831%, active RSS 107.344 MiB, 6001 pairs and p99/max interval error 0/1 ms.

Across two full trials per cadence, 1 Hz active CPU ranged from 2.8165% to 2.9833% (mean 2.8999%), while 2 Hz ranged from 4.6998% to 4.9831% (mean 4.8415%). All four trials preserved exact captured counts; each 1 Hz result was about 40.1% lower CPU than the paired 2 Hz result. Startup remained within 74.424–75.034 ms and RSS within 107.059–107.359 MiB, with no material cadence-related memory change. Keep 1 Hz provisionally: repeatable CPU reduction is supported in this environment and the focused X11 feedback observation is legible, while software RSS remains above 100 MiB and broader native acceptance remains open.

An isolated X11 visual observation at 100 clicks/s captured the actual progress footer at 0.2–8.2 second offsets. It showed the immediate first pair, then readable roughly 100-pair increments at one-second refreshes while Running; see [feedback evidence](ui-progress-feedback.md). Screen capture disturbed that short sample's timing, so it does not replace the full performance protocol. This closes the focused X11 feedback-usability check only; native Wayland, accessibility-reader and other-renderer feedback remain separate.

Two matched full WGPU/lavapipe runs at the current 1 Hz cadence measured active CPU 3.2998%/3.4998% and idle RSS 106.141/105.945 MiB, with 6001 captured pairs each. The two current-source Glow runs measured lower active CPU (2.8165%/2.9833%) but about 1.14 MiB higher idle RSS. Both renderers remain above the software RSS target. Repeated current-source hardware GPU runs favor WGPU on startup and memory, so it is now the provisional Linux default with Glow retained as an alternative; software and native compatibility gates remain open. See the [current-source software comparison](renderer-comparison.md#current-source-1-hz-follow-up--2026-10-25) and [GPU comparison](gpu-renderers.md#current-source-1-hz-gpu-follow-up--2026-10-26).

## Software-renderer memory triage — 2026-10-26

Rebuilt the current WGPU-default Linux release with `cargo build --release --locked -p nanoclick`. The executable SHA-256 was `2cab232157fc957bbfaceca1909f9816ab0ed9c47d28ea110b6cded435828031` (143,934,504 bytes). The linker emitted its existing deprecated optimization-setting warning; the build completed successfully.

Exploratory command for the Vulkan/lavapipe baseline:

```sh
LIBGL_ALWAYS_SOFTWARE=1 WGPU_BACKEND=vulkan \
  VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json \
  scripts/measure-ui-x11.sh target/release/nanoclick --quick-profile
```

The harness stabilized for 10 seconds, then measured 10-second idle and active windows. Baseline idle CPU was 0.0000%, RSS 105.258 MiB, active CPU 3.5986%, active RSS 105.387 MiB, and the passive X11 target received 1001 pairs (interarrival absolute-error p99/max 1/1 ms). RSS mappings included 48,396 KiB from `libLLVM.so.22.1`, 15,656 KiB from the executable, 13,500 KiB heap, 7,660 KiB from `libvulkan_lvp.so`, 6,608 KiB from a deleted allocation memfd and 6,512 KiB anonymous mappings.

With `MALLOC_ARENA_MAX=1` applied to the profile process tree, the same binary measured 104.832 MiB idle RSS, 3.4985% active CPU, 104.988 MiB active RSS and 1001 captured pairs (p99/max error 0/0 ms). This single short sample saves 0.426 MiB idle RSS, leaving the 100 MiB limit unmet; it does not justify a product environment override or establish a repeated-profile improvement.

With `WGPU_BACKEND=gl` and software rendering, the same quick protocol measured 117.746 MiB idle RSS, 4.2992% active CPU, 117.852 MiB active RSS and 1001 captured pairs (p99/max error 1/1 ms). The GL route mapped 51,644 KiB of LLVM and 11,756 KiB of Gallium. It used 12.488 MiB more idle RSS than the Vulkan/lavapipe sample, so it is rejected as a software-renderer fallback.

All three runs are exploratory single quick profiles, not the 60-second acceptance protocol. Software RSS remains above budget. Keep the 1 Hz progress publication and WGPU Linux default decisions separate from this unresolved memory gate; do not remove accessibility features or change the stated RSS limit to make the table pass.

### glibc trim-threshold probe — 2026-10-26

Tested `GLIBC_TUNABLES=glibc.malloc.trim_threshold=0` against the same release executable and Vulkan/lavapipe quick protocol. The first attempt exited before active sampling and is discarded. One complete repeat stabilized, sampled idle/active windows and received the expected 1001 pairs. It measured 104.719 MiB idle RSS, 104.871 MiB active RSS, 3.5986% active CPU and 0/0 ms p99/maximum interarrival absolute error. The baseline quick profile on this executable was 105.258 MiB idle RSS, 105.387 MiB active RSS, 3.5986% CPU and 1/1 ms interarrival error. This single repeat suggests about 0.54 MiB lower idle RSS, but still misses the 100 MiB target and does not establish a repeated 60-second result. Do not set the allocator tunable in production; continue profiling the application and renderer footprint.

### LLVMpipe worker-count probe — 2026-10-26

Mesa documents `LP_NUM_THREADS` for its LLVMpipe driver; the Vulkan/lavapipe run does not use this setting. Tested it on the existing WGPU OpenGL software path with `LP_NUM_THREADS=1`. One 10-second quick profile measured 117.105 MiB idle RSS, 117.211 MiB active RSS, 2.8986% active CPU and 1000 received pairs (p99/max interarrival error 0/1 ms). The prior WGPU OpenGL software quick sample measured 117.746 MiB idle RSS and 4.2992% active CPU with 1001 pairs. The 0.641 MiB single-sample RSS reduction leaves the same renderer far above budget, and the pair counts differ; this is exploratory and not a matched performance result. Reject as a product setting. See [Mesa environment-variable documentation](https://docs.mesa3d.org/envvars.html#llvmpipe-driver-environment-variables).

### Vulkan-only WGPU feature probe — 2026-10-26

Configured the Linux WGPU build with `wgpu_no_default_features` and the explicit `wgpu` `vulkan`/`wgsl` features, excluding WGPU's GLES backend. `cargo tree --locked --offline -e features --target x86_64-unknown-linux-gnu -i wgpu` showed Vulkan/WGSL without GLES. An Ubuntu 24.04 container completed `cargo check --locked --offline -p nanoclick` and a release build. The candidate executable (`9d3045780ef2af4acb7c2199974c2b8a88e47016998c55a28b937c74175b87b5`) is 140,344,080 bytes, 3,590,424 bytes (2.49%) smaller than the prior WGPU-default artifact (`2cab232157fc957bbfaceca1909f9816ab0ed9c47d28ea110b6cded435828031`, 143,934,504 bytes). The candidate passed isolated X11 GUI acceptance, including global shortcut, manual Start/countdown, stop/close behavior, finite runs for all three buttons and saved preferences.

One 10-second candidate profile on Vulkan/lavapipe measured 109.980 MiB idle RSS, 110.137 MiB active RSS, 3.5994% CPU and 1001 received pairs (p99/max interarrival error 1/1 ms). It remains above the unchanged 100 MiB RSS target. A contemporaneous profile of the older baseline binary exited before active sampling and is invalid for comparison; the historical baseline was 105.258 MiB. Therefore this probe supports a smaller executable and successful Linux GUI operation, but does not establish an RSS, CPU or timing improvement. Keep it provisional pending repeatable matched profiles, package/runtime-requirement review and the current-source Windows VM check. Linux installations without Vulkan must use the separately packaged Glow build if this backend selection is retained.

### Renderer feature-isolation follow-up — 2026-10-26

A Cargo feature-tree audit found that the target-specific Linux `eframe` dependency enabled `wgpu_no_default_features` unconditionally, so the documented Glow override still compiled `egui-wgpu`/`wgpu` even though it did not enable a WGPU backend. Removed the duplicate target-specific `eframe` entries; the app's `wgpu` feature now enables both `eframe/wgpu_no_default_features` and the platform-specific optional `wgpu` backend. The Glow graph now contains only eframe's `glow`, `x11`, `wayland`, `accesskit` and font features; `cargo tree ... -i wgpu` reports no WGPU dependency. `cargo check --locked --offline --target x86_64-unknown-linux-gnu --no-default-features --features glow -p nanoclick` passes, as does the Windows MSVC target check.

The separated Glow release executable (`5544069d449923cb0d36ba94c380356fe772becdd1b4a70b5eeef4953a78241d`, 114,721,064 bytes) passed `scripts/verify-ui-x11.sh`: outside-focus shortcut, manual countdown, Stop, close during Preparing/Running/Stop request, N=1/7/1000 for all three buttons, and saved preferences. One 10-second quick profile measured 107.305 MiB idle RSS, 107.410 MiB active RSS, 3.0999% CPU, 1001 received pairs and p99/max interval error 1/1 ms. This exploratory sample remains above the 100 MiB RSS target and does not replace full repeated profiles. The current-source Windows VM execution and native Linux compatibility matrix remain pending.
