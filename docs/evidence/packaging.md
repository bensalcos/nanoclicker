# Empaquetado local y runtime Linux

## Proceso implementado

`python3 scripts/package.py --platform linux --preview` toma un ELF x86_64 release ya construido. Genera tar.gz de aplicación, tar.gz de símbolos y SHA256SUMS. La variante Windows prepara ZIP y exige PE x86_64/PDB; falta ejecutarla en Windows.

El perfil release conserva tablas de líneas en el build. El empaquetador extrae nanoclick.debug y quita símbolos del ejecutable distribuible. Para diagnóstico, extraer/copiar nanoclick.debug junto al ejecutable para que resuelva su GNU debuglink. Los archivos se publican completos mediante hard link sin sobrescribir destinos; si un sidecar entra en conflicto, se conservan las publicaciones completas y debe reintentarse en otro directorio.

Los tar/ZIP usan orden y timestamps normalizados. Tres tests del helper pasan: reproducibilidad frente a mtime, rechazo de arquitectura incorrecta y preservación de publicación previa. No se afirma aún que dos compilaciones Rust realizadas en máquinas distintas sean idénticas byte por byte.

## Licencias y procedencia

Aviso del proyecto: Copyright (c) 2026 bensalcos, por delegación del usuario. Inventario basado en el árbol activo de Cargo para la plataforma, con avisos de las fuentes de los crates y licencias de fuentes tipográficas. El código fuente original de dependencias MPL se incluye como archivo .crate verificado contra Cargo.lock.

Se recuperaron avisos ausentes de 17 versiones de crates en 5 revisiones upstream. `assets/third-party/manifest.json` conserva URL y SHA256. Para egui 0.37.0, las revisiones Cargo VCS devolvieron HTTP 404; se usan los avisos de la etiqueta publicada 0.37.0, cuyo commit pelado fue verificado con git ls-remote. La excepción queda registrada en revisions.json y no afirma equivalencia de código entre commits.

El inventario del preview final ya no tiene entradas sin algún aviso. Esto no convierte automáticamente el inventario en una revisión legal exhaustiva ni sustituye las verificaciones de plataforma. El icono PNG conserva los bytes del primer PNG del ICO original, sin rediseñar la imagen.

## Evidencia del artefacto

Primer bundle local: aproximadamente 5.9 MiB comprimido, símbolos aparte aproximadamente 19 MiB. El ejecutable se extrajo y pasó el flujo gráfico completo en Xvfb Fedora y en un contenedor Ubuntu 24.04, usuario 65534, rootfs de solo lectura, sin red durante la prueba y sin capacidades Linux. No se instaló sobre el escritorio del usuario.

El primer intento Ubuntu identificó un requisito faltante: libxkbcommon-x11-0. La imagen QA, CI y la guía de runtime lo incluyen ahora. La segunda ejecución pasó atajo fuera de foco, pares completos, ausencia de input tras Stop, preparación manual y cierre.

`ldd` del binario lista libc/libm/libpthread/libdl; OpenGL/X11/Wayland se cargan dinámicamente. `readelf --version-info` encontró como máximo GLIBC_2.28. En Ubuntu 24.04 (glibc 2.39) arrancó el artefacto empaquetado y pasó la prueba gráfica; esto no equivale a aceptar todas las distribuciones, GPUs o escritorios.

Base QA: ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac. Imagen local con runtime gráfico: sha256:9ade4c21f404196c7d266261c24240fef808c42694402e29eba80e007e2207d5. Los paquetes apt de QA siguen actualizaciones de seguridad; registrar nuevas versiones al repetir.

## Pendiente

Windows ZIP/PDB y ejecución nativa; Wayland GNOME/KDE; DPI/lectores de pantalla; presupuestos completos por backend. La auditoría no encontró vulnerabilidades conocidas y sí tres avisos de mantenimiento, descritos en dependency-audit.md. No publicar una release como aceptada hasta cerrar los hitos del plan.

## Current preview verification (2026-10-24)

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
