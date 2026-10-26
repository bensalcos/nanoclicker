# Local packaging and Linux runtime

## Implemented process

`python3 scripts/package.py --platform linux --preview` takes an already built release x86_64 ELF. It generates application and symbol tar.gz archives plus SHA256SUMS. The Windows variant prepares ZIPs and requires x86_64 PE/PDB inputs; its later hosted verification is recorded below.

The release profile retains line tables. The packager extracts nanoclick.debug and strips symbols from the distributable executable. For diagnostics, place nanoclick.debug alongside the executable to resolve its GNU debuglink. Complete files are published through hard links without overwriting destinations; a sidecar conflict preserves completed publications and requires retrying in another directory.

Tar/ZIP ordering and timestamps are normalized. The initial three helper tests passed: reproducibility despite mtime changes, wrong-architecture rejection and preservation of prior publication. This does not establish byte-identical Rust compilation on different machines.

## Licenses and provenance

Project notice: Copyright (c) 2026 bensalcos, through the user's delegation. Inventory follows the active target's Cargo tree, crate-source notices and font licenses. Original MPL dependency source is included as a .crate archive verified against Cargo.lock.

Missing notices were retrieved for 17 crate versions at 5 upstream revisions. `assets/third-party/manifest.json` retains URLs and SHA256. For egui 0.37.0, Cargo VCS revisions returned HTTP 404; notices come from the published 0.37.0 tag, whose peeled commit was verified with git ls-remote. The exception is recorded in revisions.json without claiming source equivalence between commits.

The completed preview inventory has no entry without a notice. This is neither an exhaustive legal review nor a substitute for platform checks. The PNG icon preserves the first PNG image's bytes from the original ICO without redesign.

## Artifact evidence

First local bundle: approximately 5.9 MiB compressed, with separate symbols approximately 19 MiB. The extracted executable passed the complete GUI flow in Fedora Xvfb and an Ubuntu 24.04 container running as user 65534, with read-only rootfs, no network during testing and no Linux capabilities. It was not installed on the user's desktop.

The first Ubuntu attempt identified a missing requirement: libxkbcommon-x11-0. The QA image, CI and runtime guide now include it. The second run passed outside-focus shortcut, complete pairs, no input after Stop, manual preparation and shutdown.

`ldd` lists libc/libm/libpthread/libdl; OpenGL/X11/Wayland are loaded dynamically. `readelf --version-info` found a maximum requirement of GLIBC_2.28. On Ubuntu 24.04 (glibc 2.39), the packaged artifact started and passed the GUI flow; this does not accept all distributions, GPUs or desktops.

QA base: ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac. Local graphics-runtime image: sha256:9ade4c21f404196c7d266261c24240fef808c42694402e29eba80e007e2207d5. QA apt packages follow security updates; record new versions when repeating.

## Remaining acceptance

Native Windows execution, Wayland GNOME/KDE, DPI/screen readers and complete per-backend budgets. The recorded audit found no known vulnerabilities and three maintenance warnings, detailed in dependency-audit.md. Do not publish an accepted release until the plan's milestones close. Windows ZIP/PDB structure is verified in the later evidence below.

## Preview verification at 207f636 (2026-10-24)

Clean packaging checkout: `207f6362e7b1333748c03dadf800334aaff44a5d`. The default glow executable was rebuilt immediately before packaging; input SHA256 `731e6d3a5a48043b9a891587406a760f6bcde4a5adfc8cf4bc6792fc5a316a67`. Local output: `dist/preview-current/` (ignored, not published).

- Application archive: 6,153,053 bytes; SHA256 `88e0d98a84285a931b3ca0d9d1b01237d523fd10aba690786ee44b8878e4ce1a`.
- Separate symbols archive: 19,326,477 bytes; SHA256 `870aed326cd3610bdeaee74fa14f6d6cca82d58d80c5f9f09255d623f9fc5679`.
- Both published checksums verified. No inventory entry lacks a notice.
- A second complete packaging run into `dist/preview-current-repro/` produced identical archive checksums. This proves deterministic packaging of the same input binary and checkout, not reproducible compilation across machines.
- The extracted application passed the full isolated Ubuntu 24.04 GUI flow again after the worker-wait and Wayland lifecycle changes. Container user/capability/network restrictions above were retained. The cached runtime image config was `sha256:304676374e4fe459b75c206cc40b5b48085860a523b51f580d509255bb8412c1`; BuildKit attestation/manifest-list hashes may vary between image exports.

`build.json` deliberately separates the packaging commit from binary provenance, records the input binary hash and never infers a compiled commit from an arbitrary binary path. Packaging itself does not compile or certify the executable.

## Accessibility UI preview

Application code: `fb36ed5`; clean packaging checkout: `41c703a`. Local output `dist/preview-ui-accessibility/`. Input executable SHA256 `39cb222a455908f2979c8084dc71cfcaf6142d5a9d16d57b93ea51cb70fcb965` matches the measured release executable.

- Application archive: 6,154,251 bytes, SHA256 `50a46b8cb343b4ba9c7d02978c4f23b73bac0c368e5ff25845d299ea03c3f76e`.
- Symbols archive: 19,336,358 bytes, SHA256 `cf23539b7b19383921429e9dc5919d551c311279c268403c12e06e77c293ce67`.
- Both checksums verified; zero inventory entries without a notice. The three packaging tests pass.
- The extracted binary passed isolated Ubuntu GUI start/stop outside focus, complete pairs, no post-stop input, minimum-size action visibility, manual preparation and clean exit. The private test screen is now 2048×1536 so the capture target remains outside the application at 200% scaling; the Ubuntu run itself uses 100% scaling.

This preview includes the field names, keyboard order, contrast, scrolling and polite-status changes. Windows/Wayland native behavior, readers and performance budgets remain open.

## Checked-pair X11 preview

Local output `dist/preview-x11-checked-pairs/`, packaging checkout `e71fc32`. It includes the native-write optimization and Windows preference-parent fix, retaining the default release profile after rejecting ThinLTO. The extracted binary passed the full restricted Ubuntu GUI flow; both archive checksums verified and no dependency inventory notice is missing.

- Application archive: 6,155,694 bytes, SHA256 `772105cf5ceb5f6c6f05e647e21624f904e83da1c7c7c39af746c87c05f94e82`.
- Symbols archive: 19,338,857 bytes, SHA256 `6fdce44ac6f7d21eb4ebea4b3b875a64778f750e757899cece19b92f58908e74`.

The packaging metadata retains its conservative dirty-tree marker because an unrelated untracked entry exists in the shared workspace. Only the explicit executable, license, icon and dependency-notice/source inventory inputs are packaged. This remains a preview; native desktop and resource-budget acceptance is incomplete.

## Hosted Windows preview verification

The desktop CI job now packages its MSVC release executable and PDB with the existing packager, checks both archive hashes, verifies the packaged PE architecture and input hash, and requires a matching packaging commit with no unresolved dependency notices. It checks the separate PDB signature and retains the ZIPs/checksum manifest for 14 days. This is a preview artifact, not a GitHub release.

Seven local Python tests pass, including corruption, missing-symbol-checksum and wrong-provenance failures. The initial hosted packaging result is pending. Archive verification does not execute the application or prove native GUI, input, DPI or screen-reader acceptance.

### Windows checkout integrity regression

Run 40092266378 reached Windows preview packaging but failed on the cached AccessKit notice hash. Reproducing `git -c core.autocrlf=true cat-file --filters` locally produced CRLF bytes and a different hash from the verified upstream LF file. The new `.gitattributes` rule marks only `assets/third-party/**` as `-text`, preserving exact upstream bytes; the packager still requires the original hashes. A temporary-repository regression test failed before the rule and passes afterward. All 32 cached notice references also match under Windows checkout filters. Eight packaging tests now pass locally. Hosted packaging verification must still succeed.

A Windows-target notice preflight then found one unresolved crate, `accesskit_windows 0.35.1`. Its registry `.cargo_vcs_info.json` points to `ce8164ba92995cfa86005b6259115e08c8244253`, the exact AccessKit revision already cached. The manifest now maps that crate to the existing verified MIT/Apache notices at that revision; `fetch_notices.py` also includes the Windows adapter so regeneration preserves coverage. No notice hash, required check or release gate was relaxed.

### Windows hosted preview verified

Run 40092934864 at `acc817c9f11714d8a74576d1dedda4512e237963` passes all four jobs, including native Windows packaging. Artifact `windows-preview` (id 12560216356) was downloaded and independently checked on Linux. The executable ZIP is 3,532,360 bytes, SHA-256 `7572265011ca1e17185a995631929f817bc35eaf66b20197bbadb1e06064f153`; separate PDB ZIP is 8,294,819 bytes, SHA-256 `2a29950a02a1bbe3f902e183ee74b766c8429df2f81bf1376630263ad066ea4b`. The input executable hash is `0b62af96a6437133a97e0c86a2fd708af33f4fa15d41259a643d834034a7926c`. Metadata reports a clean packaging tree, preview=true, 128 dependency inventory entries and no unresolved notices.

Archive hashes, provenance, PE architecture, required files and the separate PDB signature pass. This is not proof of executable/PDB debug-identity matching, desktop execution, installer behavior or Windows 11 GUI/input acceptance. The hosted artifact has a 14-day retention policy and is not a published release.

## Refreshed setup/diagnostics preview

The latest local Linux preview is `dist/preview-88bd9b4/`, rebuilt and packaged at source `88bd9b49bd94f977e21143f87b40e6a8ca8b5c28`. It includes the terminal diagnostics exporter and X11 protocol/setup cancellation fixes. Input executable SHA-256: `f5565bf1e5ef6f2cc9fc3a084bffacb64472693ff2f506d8ea620590de7ec413`.

- Application archive: 6,166,394 bytes; SHA-256 `4872ec7f00d40bc2441ae138d60d9d3912468d9f885497b432dbc23e36194ab6`.
- Symbols archive: 19,375,685 bytes; SHA-256 `1b254877f30b36cc3eda65f88522be48daeee5c917f11803d86931604204e1e4`.
- Both checksums verify, all 211 inventory entries have resolved notices, and a second packaging pass in `dist/preview-88bd9b4-repro/` produces identical archive hashes. This remains deterministic packaging of one input, rather than independent reproducible compilation.
- Independent extraction verifies the GNU debuglink filename and CRC32 against the supplied `nanoclick.debug` (`a957bcff`); [validation record](preview-88bd9b4-validation.json). The stripped executable's maximum referenced GLIBC symbol version is still 2.28; this is not proof of support for every glibc 2.28 distribution.
- The extracted executable passes the expanded full GUI harness in the restricted Ubuntu 24.04 container: outside-focus shortcut, minimum size, manual countdown, close cases, all three finite N=1/7 plans, and Save/relaunch. [Runtime log](preview-88bd9b4-ubuntu.txt), [package versions](preview-88bd9b4-ubuntu-packages.txt). Image config is `sha256:810d30b8438c3e2ead0c55ca62cf98c5b9d2326955ee23a1bdc31b5b0a80c9bf`; the same pinned base and non-root/read-only/no-network/no-capabilities protocol are retained.

The metadata conservatively marks the packaging tree dirty because of the unrelated untracked workspace entry. That entry is not read or included: packaging inputs remain explicit. The CLI help also runs from the extracted artifact. No install or autostart registration was performed.

An independently downloaded Windows preview is from all-green run [40146057324](https://github.com/bensalcos/nanoclicker/actions/runs/40146057324), source `9051e65a23a8f5bf8d40025f87b5667c3bb9de58`, artifact 12580961927. Local directory: `dist/preview-windows-9051e65/`.

- Executable ZIP: 3,538,516 bytes; SHA-256 `aa9aecf85521b5f2ea2e487b048f52edec086fa14ae75aaf39628670d4b35612`.
- PDB ZIP: 8,324,785 bytes; SHA-256 `afbaae3fd95cdcfd0f83e55473d1d5aba24775641497136d8a70f3e1f8969c2b`.
- Input executable SHA-256: `44cebe60db1fd4875907b6953c7ad518fb68b884e44c5b5cb7a0eaf765dcd541`; clean packaging metadata, zero unresolved notices. The independent verifier passes checksums/provenance, required archive structure, PE architecture and the PDB signature. [Verifier output](windows-preview-9051e65-verification.json).

The two previews have different recorded checkout commits. Windows verification did not execute the executable or prove its PDB debug identity. Both archives are previews; native desktop, reader, monitor and broader timing/resource gates remain open. [Combined provenance/checksum record](preview-validation-2026-10-24.json).

## Current WGPU-default Linux bundle — 2026-10-26

The current release build selects WGPU by default, and Windows always selects WGPU even when the Glow feature is requested. Default and Glow Linux Clippy, Windows default and Glow-requested cross-target checks all pass. Current input executable SHA-256: `05fc23a9bf8fbb4e9ae2de55f079e648d22a4240328f666fdb4962278265468a`. Local preview output: `dist/preview-linux-wgpu-final-20261026/`. Application archive SHA-256: `18fff4a3b751f6aca7a28da42a42fd467049c7a6666116378532aff40acaf3fd`; symbols archive: `e5d04f497a508a9ad4844380afb95108183d751fad8a9fe2136f6af2de415541`. Dependency inventory reports zero unresolved notices. The packaging checkout was dirty, so no clean source commit is inferred for the binary.

The extracted archive's README correctly requires a Vulkan loader and working driver for the default Linux WGPU build; it names `libvulkan1` and `mesa-vulkan-drivers` for Ubuntu X11 and OpenGL/EGL only for a Glow build. Its stripped executable SHA-256 is `5e3e4c69631544a398cb1ea9bc2bdb7e5d932cb770cbe58fb617d37996a04cf4`. This exact executable passed `scripts/verify-ubuntu.sh` in the pinned Ubuntu 24.04 container (image config `304676374e4fe459b75c206cc40b5b48085860a523b51f580d509255bb8412c1`): non-root user 65534, read-only root filesystem, no network and no Linux capabilities. The full GUI flow passed outside-focus shortcut start/stop, exact finite N=1/7/1000 for all three buttons, no post-Stop input, minimum window size, manual preparation, close while Preparing/Running/Stop-request race, and Save/relaunch. Captured terminal telemetry identifies renderer `wgpu`, backend `x11`; [runtime log](preview-wgpu-default-ubuntu.txt).

This validates the software-rendered WGPU package path on Ubuntu/X11. It does not establish hardware Vulkan performance, other distributions, native Wayland, screen readers or the remaining release matrix. The Xvfb log includes EGL/DRI3 diagnostic warnings from the headless environment; the WGPU GUI flow and renderer telemetry still pass.

### Matching-checkout Windows preview and hosted validation

Run [40146929746](https://github.com/bensalcos/nanoclicker/actions/runs/40146929746) at `88bd9b49bd94f977e21143f87b40e6a8ca8b5c28` passes all four jobs. Downloaded logs confirm the owned-display unit tests, six actual finite GUI runs, twenty measured GUI stress cycles, eight blocked-resource cases and thirteen portal mock scenarios. The stress sample has unchanged RSS 126108 KiB, 16 descriptors and 15 threads; stable RSS still exceeds the 100 MiB budget and does not accept memory performance.

The Windows preview from this same checkout is now downloaded in `dist/preview-windows-88bd9b4/`, artifact 12581028582, and passes the independent verifier. Application ZIP: 3,538,514 bytes, SHA-256 `c5e1c5abfd15cb257b3d3ac53f943d9c4d870d88469b4318e7a675205a4b4504`. Separate PDB ZIP: 8,313,413 bytes, SHA-256 `0006ea0cd32764236f032a0e6631dc40e3aa1e8d637de006442171eb2de3fee2`. Input executable SHA-256: `580e7ad542b55dcaccef9cf57aac0ac3b173c069b27654c0245b2ad473f05a8a`. Metadata is clean and reports zero unresolved notices. [Verifier output](windows-preview-88bd9b4-verification.json); the combined provenance record above now points to this preview. The preceding 9051e65 record remains historical evidence.

Linux and Windows previews now share checkout 88bd9b4. The hosted GUI/input capture remains Linux/Xvfb; native Windows desktop execution and all other release gates remain separate.

## Shortcut delivery fix previews

Producer source `42abf7dac1f9b7ddb728b940fad2c543f516ba38` passes all four hosted jobs in run 40177331385. Linux output `dist/preview-shortcut-queue/` includes the shortcut queue/Stop-priority fix; its input executable is `ce31f3f2ebd14ed3c97534d99aca3aec341ba84c8189870d3b592f6705e64ced`. Both archive hashes verify, notices have no unresolved entries, and extracted CLI help runs. The matching unstripped release passes the expanded product GUI checks. This preview has not repeated the restricted-container GUI or full performance protocol.

The matching Windows preview is independently downloaded into `dist/preview-windows-42abf7d/` and passes the existing archive verifier. [Verifier output](windows-preview-42abf7d-verification.json). It is not native Windows desktop acceptance. [Archive sizes, hashes and scope](preview-shortcut-queue.json). The earlier 88bd9b4 previews remain historical artifacts; these directories contain the shortcut fix.

## Renderer-aware Linux preview — 2026-10-26

The package command now accepts `--renderer wgpu|glow` (default `wgpu`), records the selection in `build.json`, and uses that renderer's Cargo feature set for the dependency/license inventory. Glow packages receive a renderer-specific `-glow` filename and OpenGL/EGL runtime instructions; Windows rejects Glow because its app build requires WGPU.

Built a Glow release (`5544069d449923cb0d36ba94c380356fe772becdd1b4a70b5eeef4953a78241d`) and packaged it to `dist/preview-glow-feature-isolation/`. The application archive is `nanoclick-0.1.0-linux-x86_64-glow-preview.tar.gz`, SHA-256 `3162dc359d66a187bfb1179f58bfac30fc9f4a94fd62c5065cbf395ebaa34711`; the symbol archive SHA-256 is `ac284c99e5f4c6098c249d53775d2f3ded5074639f26b79e617d8a3a970c59c6`. Its README names OpenGL/EGL and does not require Vulkan; `build.json` says `renderer=glow`; the 211-entry dependency inventory contains no WGPU/egui-wgpu packages and has zero unresolved notices. The renderer feature and packaging unit tests pass. This local preview's full GUI run and quick profile are recorded in [performance evidence](ui-polish-performance.md#renderer-feature-isolation-follow-up--2026-10-26); clean-reference and native Windows acceptance remain open.
