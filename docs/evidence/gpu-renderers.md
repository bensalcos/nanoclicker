# Matched-source renderers on the private GPU display

Date: 2026-10-24. Both application builds and the expanded GUI fixture use source `e244efd5212c85fe024057de569b7f2284e6e933`, Rust 1.97.1, the standard portable release profile, 520×600 logical units, dark theme and 2 Hz progress. Later English-documentation commits do not change that source. No optimization flags, fonts, accessibility features or product budgets were removed or weakened.

The private environment is headless Weston 15 GL with a fake seat and rootful Xwayland 24.1.13, AMD Renoir 1002:15e7, Mesa 26.2.2 on Fedora 44. It has a fresh runtime/config directory and activation-disabled private D-Bus. It neither connects to the host compositor nor injects into the physical desktop. See [setup and isolation](gpu-display-setup.md).

Both executables pass the full GUI preflight, now including actual Save/relaunch and the saved 37 ms interval. The script then seeds a separate 10 ms benchmark plan, performs thirty warm launches, stabilizes for 60 seconds, measures idle for 60 seconds and active input for 60 seconds. Runs were sequential, wgpu then glow; no compiler or other heavy verification ran during measured phases. Each renderer has one complete run on a shared host using the powersave governor, so small differences are not evidence of statistical significance.

| Measurement | Glow | Wgpu Vulkan | Initial budget |
|---|---:|---:|---:|
| Warm startup p95, 30 starts | 117.712 ms | 71.110 ms | ≤500 ms |
| Idle CPU, one core, 60 s | 0.0167% | 0.0000% | ≤1% |
| Idle RSS after stabilization | 84.762 MiB | 62.676 MiB | ≤100 MiB |
| Active CPU, requested 100 pairs/s, 60 s | 2.3832% | 2.1333% | ≤5% |
| Active RSS | 87.047 MiB | 62.828 MiB | Informational |
| Matching captured down/up pairs | 6001 | 6001 | Recorded |
| Server interarrival absolute error p99 / max | 1 / 1 ms | 1 / 1 ms | Not engine deadline lateness |

Both meet the listed startup/CPU/idle-RSS budgets **in this environment**. Wgpu uses less RSS and has a lower sampled startup p95. This does not close software-rendering budgets, Windows/native Wayland timing, actual desktop monitor transitions or reader acceptance. It does not establish the whole-UI engine deadline p99 requirement: these capture timestamps have millisecond resolution and measure interarrival error instead.

## Graphics attestation and limits

Both invocations set `WGPU_BACKEND=vulkan` and `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json`; glow does not consume wgpu's backend selection. The pinned egui-wgpu 0.37 setup honors WGPU_BACKEND through Backends::from_env. The selected ICD's [Vulkan enumeration](gpu-matched-vulkan.txt) reports only AMD RADV Renoir, not llvmpipe. The measured wgpu process [maps libvulkan_radeon](gpu-matched-wgpu-drivers.txt); the glow process [maps GLX Mesa/libgallium](gpu-matched-glow-drivers.txt), and each display's GLX probe reports AMD accelerated rendering. The exact application AdapterInfo/GL renderer string was not queried; retain that distinction rather than substituting the display probe for application telemetry.

Each manual preflight required one fixture-only cursor reassertion after a fake-seat focus transition. That accommodation is logged and never runs during the measured active phase. The actual physical pointer, GNOME/KDE portal assignment and native Wayland input are outside this fixture's scope.

Decision at the time: retain glow provisionally pending current-source evidence. The subsequent comparison below supersedes that operational default, but not the remaining acceptance gates. The earlier [software comparison](renderer-comparison.md) remains a separate result.

Data: [glow CSV](gpu-matched-glow.csv), [wgpu CSV](gpu-matched-wgpu.csv), [environment, hashes and results](gpu-renderers.json), [glow GUI preflight](gpu-matched-glow-validation.txt), [wgpu GUI preflight](gpu-matched-wgpu-validation.txt). Local full logs remain in `/tmp/nanoclick-gpu.pIkwxf` and `/tmp/nanoclick-gpu.SqcfIo`; the committed data above is self-contained.

Reproduce after building and preserving each release executable:

```sh
renderer_dir="$(mktemp -d /tmp/nanoclick-renderers.XXXXXX)"
cargo build --release --locked -p nanoclick --no-default-features --features glow
cp target/release/nanoclick "$renderer_dir/nanoclick-glow"
cargo build --release --locked -p nanoclick
cp target/release/nanoclick "$renderer_dir/nanoclick-wgpu"
for renderer in wgpu glow; do
  WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json \
    scripts/measure-ui-gpu.sh /path/to/verified/weston/root/usr "$renderer_dir/nanoclick-$renderer"
done
```

Use the verified Weston prefix described in the setup document and run the full script separately for each binary. The script's --quick-profile is exploratory and cannot substitute for these 60-second phases.

## Current-source 1 Hz GPU follow-up — 2026-10-26

Both release binaries use the current 1,000 ms progress-publication source and passed the full GUI preflight on the same isolated AMD Renoir/RADV display described above. We ran two complete 60-second trials per renderer in alternating order. WGPU required `WGPU_BACKEND=vulkan` and `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json`; Glow used the same display. Candidate binary SHA-256: Glow `39b8aab3673ea4deb6523d3025e686d824d5e9750e5ba890064720cb8f926d84`; WGPU `e7b5af41a028a1b36b74afcae5e0d390ed5aa58ed408b8c0fbfac8466deebff5`.

| Measurement | Glow run 1 | Glow run 2 | WGPU run 1 | WGPU run 2 |
|---|---:|---:|---:|---:|
| Warm startup p95, ms | 110.392 | 113.732 | 53.221 | 53.346 |
| Idle RSS, MiB | 83.602 | 82.566 | 61.684 | 55.969 |
| Active CPU, one core | 1.3832% | 1.2666% | 1.3832% | 1.3666% |
| Active RSS, MiB | 84.324 | 83.223 | 63.105 | 57.410 |
| Captured pairs | 6001 | 6001 | 6001 | 6001 |
| Server interarrival absolute error p99 / max, ms | 0 / 1 | 0 / 2 | 0 / 0 | 0 / 2 |

WGPU's warm-start p95 averaged 53.284 ms versus Glow's 112.062 ms; idle RSS averaged 58.827 MiB versus 83.084 MiB (24.257 MiB, about 29%, lower). Active CPU averaged 1.375% for WGPU and 1.325% for Glow. All trials captured the expected 6001 pairs. Given the material hardware-rendered startup and memory improvement, WGPU is now the provisional Linux default; retain Glow as an explicit compatibility alternative. The separate Xvfb/lavapipe comparison shows a small active-CPU advantage for Glow and both remain over the software RSS budget, so this choice is not a claim that every driver performs better with WGPU.

This changes the build default only. Native Xorg/Wayland, Vulkan availability and driver compatibility across supported distributions, software-rendering budgets, monitor and screen-reader behavior, and the constrained Windows VM performance remain open acceptance gates. The physical-GPU result does not replace them.
