# ThinLTO footprint screening: rejected

Hypothesis: cross-crate ThinLTO might reduce the application's mapped code enough to help the 100 MiB RSS budget. The [mapping profile](idle-mappings-baseline.csv) showed roughly 11 MiB from NanoClick and 47 MiB from Mesa/LLVM, so this was tested as a bounded build-only experiment, without changing features or rendering behavior.

Both binaries use the checked-pair X11 source at `5b55daf`, Rust 1.97.1, default glow, opt-level 3, 16 codegen units, line-table symbols and panic unwinding. Only LTO changes. The experimental command was:

```sh
CARGO_PROFILE_RELEASE_LTO=thin CARGO_TARGET_DIR=target/experiment-thin-lto \
  cargo build --release --locked -p nanoclick -j2
```

The clean experimental target built in 4m05s. The build's existing local linker warning about deprecated optimization setting '1' was not suppressed. No permanent Cargo profile setting was added. Cargo's [profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html#lto), checked 2026-10-24, distinguishes default thin-local LTO from cross-crate `lto="thin"`.

Both binaries passed the real isolated X11 GUI flow. Matching untraced quick runs used `LIBGL_ALWAYS_SOFTWARE=1 scripts/measure-ui-x11.sh PATH --quick-profile`: 10-second stabilization, 10-second idle and 10-second active observation. [Screening data](thin-lto-screening.csv):

| Measurement | Default release | ThinLTO |
|---|---:|---:|
| ELF text+data+BSS | 15,580,996 bytes | 15,670,924 bytes |
| Idle process RSS | 107.832 MiB | 107.816 MiB |
| Mapped executable RSS | 11,712 KiB | 11,660 KiB |
| Active CPU, one core | 4.5993% | 4.6986% |

There is no credible memory gain: process RSS differs by only 16 KiB, mapped code by 52 KiB, and deterministic ELF size increases by 89,928 bytes. The small CPU difference is also inside short-run variance. These are exploratory 10-second samples, **not** the required 60-second budget evidence; neither may be used to claim the 5% CPU gate passes.

Decision: reject the experiment and retain the existing release profile. Do not repeat this footprint idea without a meaningful source/compiler change or new profiling evidence. Panic isolation, accessibility, fonts, platform backends and both error checks remain intact.

Executable SHA256: default `f8305be9ca7709151b6e43ba087f90f2a70feaa2a00b25354cd70a1b36af8e9d`; ThinLTO `9a43b547e5211deb64bfc7860bc34cb8b409416015d66ee1848720a909e1ba16`. Preserved local executables are under `artifacts/performance/`.
