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
