# Auditoría de dependencias

Fecha: 2026-10-24. Herramienta: cargo-audit 0.22.2, ejecutada contra Cargo.lock completo. Base RustSec actualizada 2026-10-23, commit 8a5b8090ef20c9c1b425f3373f28b0b23cc5c7a0. Resumen estructurado: dependency-audit.json; salida completa local en artifacts/cargo-audit.json.

Resultado: **0 vulnerabilidades conocidas y 3 avisos de mantenimiento**, no un resultado sin avisos:

- RUSTSEC-2026-0371: zbus_names 4.3.5 no mantenido como crate independiente; integrado en zbus 6.
- RUSTSEC-2026-0372: zvariant_derive 5.15.1 no mantenido como crate independiente; macros integradas en zbus 6.
- RUSTSEC-2026-0370: zvariant 5.15.1 no mantenido como crate independiente; formato de transporte integrado en zbus 6.

Se consultaron versiones publicadas: ashpd 0.13.13, eframe 0.37.0 y accesskit_unix 0.24.0 son las disponibles y usan la familia D-Bus anterior. No se forzó una sustitución semver incompatible ni se añadieron ignores. Revisar las migraciones upstream antes de aceptar la distribución y mantener estos avisos visibles.

Instalación de la herramienta en este entorno: el wrapper cc del usuario ejecuta zig cc y falló ante un --target de aws-lc-sys. Se reprodujo/localizó el error y se compiló cargo-audit usando CC=/usr/bin/gcc, CXX=/usr/bin/g++ y linker /usr/bin/gcc solo para ese comando. No se modificó el compilador global ni se debilitó la auditoría. La herramienta vive en /tmp/nanoclick-tools.
