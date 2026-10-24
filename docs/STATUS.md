# Estado de NanoClick

Actualizado: 2026-10-24.

- **Plan:** arquitectura, UI, calidad, fases y backlog documentados; T01 completada con baseline Windows explícitamente pendiente.
- **Idioma:** nueva aplicación únicamente en inglés.
- **Implementado:** workspace Rust de tres crates; dominio validado, scheduler monotónico, worker cancelable y snapshots; UI inicial de temas claro/oscuro, tres botones, infinito/N, cuenta regresiva manual, atajos Windows/X11 y configuración mediante portal Wayland.
- **Verificado:** 33 tests (22 core, 2 controller, 6 de preferencias y 3 helpers Windows), Clippy Linux y Windows sin warnings; capturador Xvfb recibió 10 pares ordenados para cada botón y comprobó atajo fuera de foco, tecla sostenida y conflicto. Seis capturas Xvfb de ambos temas a 100/150/200% revisadas: controles principales visibles; fragmento de ayuda en el borde de scroll a 200% pendiente. Ver evidence/ui-scaling.md.
- **Atajos:** listeners propios por eventos en X11 y Windows; biblioteca global-hotkey solo como parser. La prueba del ejecutable completo pasó en Xvfb (atajo, cuenta regresiva, Stop y cierre).
- **Espera del worker:** puerto Waiter integrado; Condvar Linux y temporizador Windows de alta resolución con evento cancelable por ejecución. Errores de espera/cleanup cubiertos, ver evidence/native-waiter.md.
- **Windows:** cargo check para x86_64-pc-windows-msvc terminó; no hay ejecución nativa ni binario enlazado validado. Clippy Windows también pasó tras corregir el warning de campo exclusivo de Linux.
- **Wayland:** interfaces reales RemoteDesktop v2 y GlobalShortcuts v1 disponibles en GNOME local; adaptadores implementados, sin sesión de inyección autorizada/validada. KDE pendiente. Monitor Closed instalado antes de solicitar permisos; detección explícita de pérdida del servicio y QA nativo aún pendientes (evidence/wayland-lifecycle.md).
- **Preferencias:** guardado explícito con Save; esquema versionado y reemplazo atómico, tests Linux de corrupción, versión futura y fallo de publicación. Compila para Windows; ejecución allí pendiente.
- **CI:** workflow Windows/Linux, renderer alternativo y MSRV definido; YAML parseado y acciones fijadas por commit. Ejecución hospedada pendiente.
- **Licencia del proyecto:** aviso MIT completado como Copyright (c) 2026 bensalcos, tras delegación explícita del usuario.
- **Packaging:** preview Linux con licencias, fuentes MPL verificadas, checksums y símbolos separados; ejecutable extraído validado en Xvfb Fedora y Ubuntu 24.04 aislado. Windows ZIP/PDB pendiente. Ver evidence/packaging.md.
- **Auditoría:** cero vulnerabilidades conocidas y tres avisos de mantenimiento upstream, registrados sin ignores en evidence/dependency-audit.md.
- **Rendimiento:** release glow/Xvfb: arranque p95 88.575 ms, CPU reposo 0.0167%, RSS 106.965 MiB; CPU activo 5.2330% tras reducir progreso a 2 Hz. RSS y CPU activo todavía exceden presupuestos; ver evidence/ui-performance.md.
- **Pendiente:** comparación glow/wgpu y benchmarks nativos release; CI hospedado; QA nativo/DPI/NVDA/Orca y aceptación por plataforma. No retirar C++ todavía.
- **Siguiente:** ejecutar espera nativa Windows, comparar renderers (MSRV 1.95 ya pasó check local) y ampliar las mediciones nativas disponibles. El núcleo ya tiene 4000 mediciones release de Stop con sink sin entrada; no sustituyen métricas de producto.

Ver [evidencia](evidence/implementation.md), [plan](../tasks/plan.md) y [tareas](../tasks/todo.md). Los hitos de plataforma permanecen abiertos aunque exista código compilable.
