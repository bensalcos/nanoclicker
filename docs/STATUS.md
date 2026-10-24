# Estado de NanoClick

Actualizado: 2026-10-24.

- **Plan:** arquitectura, UI, calidad, fases y backlog documentados; T01 completada con baseline Windows explícitamente pendiente.
- **Idioma:** nueva aplicación únicamente en inglés.
- **Implementado:** workspace Rust de tres crates; dominio validado, scheduler monotónico, worker cancelable y snapshots; UI inicial de temas claro/oscuro, tres botones, infinito/N, cuenta regresiva manual, atajos Windows/X11 y configuración mediante portal Wayland.
- **Verificado:** 16 tests (14 core y 2 controller), Clippy Linux y Windows sin warnings; capturador Xvfb recibió 10 pares ordenados para cada botón y comprobó atajo fuera de foco, tecla sostenida y conflicto. Capturas finales Xvfb de ambos temas revisadas, con los tres bloques completos.
- **Windows:** cargo check para x86_64-pc-windows-msvc terminó; no hay ejecución nativa ni binario enlazado validado. Clippy Windows también pasó tras corregir el warning de campo exclusivo de Linux.
- **Wayland:** interfaces reales RemoteDesktop v2 y GlobalShortcuts v1 disponibles en GNOME local; adaptadores implementados, sin sesión de inyección autorizada/validada. KDE pendiente.
- **CI:** workflow Windows/Linux, renderer alternativo y MSRV definido; YAML parseado y acciones fijadas por commit. Ejecución hospedada pendiente.
- **Pendiente:** comparación glow/wgpu y benchmarks release; preferencias; CI hospedado; QA nativo/DPI/NVDA/Orca; packaging y aceptación por plataforma. No retirar C++ todavía.
- **Siguiente:** cerrar verificación del primer corte ejecutable, guardar commit y continuar preferencias, CI y mediciones independientes de los escritorios ausentes.

Ver [evidencia](evidence/implementation.md), [plan](../tasks/plan.md) y [tareas](../tasks/todo.md). Los hitos de plataforma permanecen abiertos aunque exista código compilable.
