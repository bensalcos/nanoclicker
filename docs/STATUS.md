# Estado de NanoClick

Actualizado: 2026-10-24.

- **Plan:** arquitectura, UI, calidad, fases y backlog documentados; T01 completada con baseline Windows explícitamente pendiente.
- **Idioma:** nueva aplicación únicamente en inglés.
- **Implementado:** workspace Rust de tres crates; dominio validado, scheduler monotónico, worker cancelable y snapshots; UI inicial de temas claro/oscuro, tres botones, infinito/N, cuenta regresiva manual, atajos Windows/X11 y configuración mediante portal Wayland.
- **Verificado:** 22 tests (14 core, 2 controller y 6 de preferencias), Clippy Linux y Windows sin warnings; capturador Xvfb recibió 10 pares ordenados para cada botón y comprobó atajo fuera de foco, tecla sostenida y conflicto. Capturas finales Xvfb de ambos temas revisadas, con los tres bloques completos.
- **Windows:** cargo check para x86_64-pc-windows-msvc terminó; no hay ejecución nativa ni binario enlazado validado. Clippy Windows también pasó tras corregir el warning de campo exclusivo de Linux.
- **Wayland:** interfaces reales RemoteDesktop v2 y GlobalShortcuts v1 disponibles en GNOME local; adaptadores implementados, sin sesión de inyección autorizada/validada. KDE pendiente.
- **Preferencias:** guardado explícito con Save; esquema versionado y reemplazo atómico, tests Linux de corrupción, versión futura y fallo de publicación. Compila para Windows; ejecución allí pendiente.
- **CI:** workflow Windows/Linux, renderer alternativo y MSRV definido; YAML parseado y acciones fijadas por commit. Ejecución hospedada pendiente.
- **Pendiente:** comparación glow/wgpu y benchmarks nativos release; CI hospedado; QA nativo/DPI/NVDA/Orca; packaging y aceptación por plataforma. No retirar C++ todavía.
- **Siguiente:** continuar revisión de los atajos Windows, comprobar MSRV/renderers y ampliar las mediciones nativas disponibles. El núcleo ya tiene 4000 mediciones release de Stop con sink sin entrada; no sustituyen métricas de producto.

Ver [evidencia](evidence/implementation.md), [plan](../tasks/plan.md) y [tareas](../tasks/todo.md). Los hitos de plataforma permanecen abiertos aunque exista código compilable.
