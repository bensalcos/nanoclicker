# Backlog Rust

Estado: implementación en curso; consultar docs/STATUS.md y docs/evidence/implementation.md. Tamaño S: 1–2 archivos; M: 3–5 archivos. Si una tarea excede ese alcance, dividirla antes de implementarla, conservando dependencias y aceptación. Rutas siguientes son previstas.

## P0 — Viabilidad

### T01 — Fijar protocolo de comparación [S]
- [x] Registrar máquinas, sistemas disponibles, matriz de versiones y capturador de prueba en `docs/evidence/environment.md`.
- [x] Definir fixtures de paridad y medir C++ en Windows si está disponible; si no, dejar baseline explícitamente pendiente.

**Dependencias:** ninguna. **Verificación:** protocolo reproducible y evidencia separada de resultados pendientes. **Archivos:** `docs/evidence/environment.md`, `docs/evidence/baseline.md`.

### T02 — Prototipar renderer y UI accesible [M]

Partial evidence: matched-source release glow/wgpu runs on software X11 recorded in `docs/evidence/renderer-comparison.md`. Both exceed memory and active CPU budgets. Linux default remains provisional; Windows/GPU/accessibility acceptance is pending.
- [ ] Comparar glow/wgpu con ventana mínima, teclado, DPI y árbol accesible; registrar tamaño, arranque y reposo.
- [ ] Elegir renderer y fijar tokens/componentes visuales con capturas de ambos temas; si ninguno cumple, evaluar alternativa de arquitectura.

**Dependencias:** T01. **Verificación:** ejecución Windows/Linux y revisión de resultados contra UI.md/VALIDATION.md. **Archivos:** `experiments/ui/`, `docs/evidence/ui-spike.md`.

### T03 — Probar entrada y espera Windows/X11 [M]

Implementation: Windows high-resolution timer and per-run cancellation event compile; Linux condition-variable and X11 tests pass. Native Windows timing remains pending. See `docs/evidence/native-waiter.md`.
- [ ] Inyectar pares en capturador aislado, detectar errores y liberar botón ante fallo parcial; probar registro/conflicto de atajo.
- [ ] Seleccionar espera nativa cancelable y comprobar Stop durante intervalo largo sin busy-wait.

**Dependencias:** T01. **Verificación:** eventos capturados, pruebas de error y tiempos registrados por SO. **Archivos:** `experiments/input/`, `docs/evidence/input-spike.md`.

### T04 — Probar portales Wayland [M]
- [ ] En GNOME/KDE, probar sesión POINTER y atajo fuera de foco sobre cliente Wayland nativo; documentar trigger efectivo.
- [ ] Probar denegación/revocación y medir coste de NotifyPointerButton; seleccionar cliente D-Bus y modelo de ejecución.

**Dependencias:** T01. **Verificación:** tabla de capacidades/versiones, eventos y mediciones; entornos ausentes permanecen pendientes. **Archivos:** `experiments/wayland/`, `docs/evidence/wayland-spike.md`.

**Hito P0:** [ ] Decisiones de UI/plataforma sustentadas; ninguna limitación crítica oculta.

## P1 — Primer flujo utilizable

### T05 — Crear workspace y dominio validado [M]
- [x] Añadir manifests de tres crates, toolchain/MSRV, lockfile y features compatibles por target.
- [x] Modelar intervalo, botón, conteo y estados con errores tipados; rechazar límites inválidos.

**Nota de ejecución:** el dominio y el workspace avanzan mientras T02/T03/T04 reúnen evidencia de escritorio; esto no cierra P0 ni congela el renderer.

**Dependencias:** T02 y T01. **Verificación:** fmt, clippy y tests de límites; builds por target. **Archivos:** manifests/toolchain, `crates/core/src/config.rs`, `state.rs` y entradas mínimas.

### T06 — Implementar sesión con backend falso [M]
- [x] Scheduler monotónico, primer clic inmediato, N exacto, omisión de slots y métricas acotadas.
- [x] Worker con señal Stop/Shutdown prioritaria, snapshot terminal y run_id; parada y cierre interrumpen espera.

**Dependencias:** T05. **Verificación:** reloj falso, pruebas de carreras, saturación y overflow; sin sleeps para probar aritmética. **Archivos:** `core/src/{engine,scheduler,ports,metrics}.rs`, tests.

### T07 — Integrar backend Windows [M]
- [ ] Wrapper SendInput con verificación de retorno, recuperación de botón y espera cancelable seleccionada.
- [ ] Flujo real Start/Stop izquierdo desde harness; cierre sin worker activo ni handle perdido.

**Dependencias:** T03, T06. **Verificación:** capturador Windows y pruebas de error; no usar el escritorio del usuario como test automático. **Archivos:** `platform/src/windows/`, integración Windows.

### T08 — Entregar Start/Stop desde UI en Linux X11 [M]
- [ ] Backend XTEST y composición UI/controller usando contratos existentes; conectar también Windows.
- [ ] UI permite inicio manual con cuenta regresiva cancelable y parada con acuse, sin bloquear hilo gráfico.

**Dependencias:** T02, T03, T07. **Verificación:** clics fuera de la UI, Stop y cierre en ambos sistemas. **Archivos:** `platform/src/x11/`, `app/src/{main,controller,ui}.rs`.

**Hito P1:** [ ] Flujo izquierdo Windows/X11 medido y utilizable; núcleo independiente del renderer.

## P2 — Paridad

### T09 — Automatizar checks por plataforma [S]

Accepted CI evidence: run 40092934864 at acc817c passed all four jobs; X11 evidence artifact downloaded and inspected. Native GUI/input/reader gates remain separate. See `docs/evidence/ci.md`.
- [x] CI Windows/Linux con fmt, clippy, tests, build release y features válidas.
- [x] Artefactos de fallo e integración X11 aislada; separar estos checks de QA nativo.

**Dependencias:** T08. **Verificación:** runs hospedados exitosos y logs; un YAML local no es CI aceptado. **Archivos:** `.github/workflows/`, scripts si son necesarios.

### T10 — Completar botones y repetición [M]
- [ ] Tres botones, infinito y N validados desde UI; snapshot muestra conteo y razón de parada.
- [ ] Fin automático desbloquea controles; edición activa bloqueada, reinicio rápido no reutiliza estado anterior.

**Dependencias:** T09. **Verificación:** pruebas core y capturador, N=1/N grande, secuencias rápidas y fallo down/up. **Archivos:** `app/src/ui.rs`, `controller.rs`, adaptadores/tests existentes.

### T11 — Integrar atajos Windows/X11 [M]
- [ ] F6 por defecto y cambio sin perder binding válido ante conflicto; una pulsación produce una transición.
- [ ] La UI en inglés muestra el atajo efectivo y permite Stop fuera de foco.

**Dependencias:** T10. **Verificación:** conflictos, modificadores/layout, tecla sostenida y mensajes en estados terminales. **Archivos:** adaptadores hotkey, UI/controller.

### T12 — Integrar Wayland y capacidades [M]
- [ ] Adaptador de portal con preparación cancelable, sesiones y señales de revocación; UI distingue inyección y atajo.
- [ ] Trigger efectivo del escritorio y modo manual explícito cuando no hay atajo; portal ausente impide iniciar si falta inyección.

**Dependencias:** T04, T08, T10. **Verificación:** flujos nativos GNOME/KDE, rechazo y revocación; prueba XWayland separada. **Archivos:** `platform/src/wayland/`, `capabilities.rs`, controller y vista de capacidades.

**Hito P2:** [ ] Matriz de paridad completa, con soporte por entorno acreditado y limitaciones visibles.

## P3 — Calidad de uso

### T13 — Persistir preferencias de forma recuperable [M]

Implementation: versioned explicit Save and seven recovery tests pass on Linux and hosted Windows. Native desktop Save/relaunch QA remains pending.
- [x] Tema, plan y atajo solicitado en esquema versionado, sin estado activo ni reanudación automática.
- [ ] Escritura/reemplazo seguro en cada SO, recuperación de archivo inválido y directorios estándar.

**Dependencias:** T11, T12. **Verificación:** roundtrip, corrupción, versión futura y fallo de escritura preservando último archivo válido. **Archivos:** `app/src/settings.rs`, integración/controller y tests.

### T14 — Aceptar UI, DPI y accesibilidad [M]

Partial evidence: names/values verified in the actual AccessKit tree; Tab/Shift+Tab order, Enter behavior, token contrast and primary-action bounds have tests. Xvfb full flows pass at 100/150/200%, including minimum-size scrolling; default help clipping is fixed. Native DPI and NVDA/Orca remain pending. See `docs/evidence/ui-scaling.md` and `ui-accessibility.md`.
- [ ] Pulir tokens, foco, errores, escalado y temas según UI.md con capturas reales.
- [ ] Completar teclado y pruebas NVDA/Orca, sin recorte a 420×520 y 200% DPI.

**Dependencias:** T13. **Verificación:** matriz visual y checklist con resultados por entorno. **Archivos:** `app/src/{ui,theme}.rs`, `docs/evidence/ui-qa.md`.

### T15 — Endurecer lifecycle y fallos [M]
- [ ] Cubrir cierre en Preparing/Running/Stopping, backend desconectado, permisos revocados y liberación fallida.
- [ ] Garantizar estado terminal observable y recursos cerrados; repetir ciclos y verificar estabilidad de memoria.

**Dependencias:** T14. **Verificación:** fallos inyectados y estrés nativo acotado; logs agregados sin evento por clic. **Archivos:** motor/controller/adaptadores afectados, tests y evidencia.

**Hito P3:** [ ] Flujos normales y fallos aceptados con accesibilidad verificada.

## P4 — Rendimiento y entrega

### T16 — Medir y optimizar cuellos de botella [M]

Partial evidence: actual release GUI measured on Xvfb; 2 Hz progress lowers software-rendering CPU from 19.67% to 5.23%. Active CPU and RSS budgets remain unmet. Native X11 input completed 60-second no-added-load and two-load-thread timing runs with matching captured pairs; this excludes the GUI and other backends. An additional 4,000 native X11 Stop cycles retain matching captured pairs. See `docs/evidence/ui-performance.md`, `docs/evidence/x11-deadline-lateness.md` and `docs/evidence/x11-stop-latency.md`.
- [ ] Medir tabla VALIDATION.md por renderer/backend, comparar C++ donde exista baseline y perfilar desviaciones.
- [ ] Optimizar únicamente hallazgos reproducibles; comprobar que ninguna mejora empeora Stop, CPU o precisión sin decisión registrada.

**Dependencias:** T15. **Verificación:** release, capturador, 30 arranques, pruebas de carga y 1000 paradas; datos por commit. **Archivos:** `benches/`, harness, `docs/evidence/performance.md` y módulo perfilado.

### T17 — Empaquetar y comprobar instalación limpia [M]

Partial evidence: Linux preview with notices, source archives, checksums and separate symbols passed isolated Ubuntu 24.04 execution. Windows CI now produces a ZIP/PDB preview with zero unresolved notices; the downloaded archives pass independent structural/hash verification. Eight packaging tests pass; Windows runtime and native desktop acceptance remain pending. See `docs/evidence/packaging.md`.
- [ ] ZIP Windows y tar.gz Linux con iconos, licencias, checksums, requisitos y símbolos separados.
- [ ] Ejecutar en instalaciones limpias de referencia; revisar features, dependencias/licencias y baseline glibc.

**Dependencias:** T16. **Verificación:** ejecutar artefacto final, no solo `cargo run`; registrar auditoría y recursos usados. **Archivos:** scripts de packaging, workflow de artefactos, documentación.

### T18 — Cerrar migración y actualizar documentación [M]
- [ ] README Rust con build, uso, permisos y límites reales; changelog y matriz final de evidencia.
- [ ] Retirar C++/Visual Studio en cambio separado después de aceptación; conservar referencia histórica y revisar titularidad MIT/iconos antes de distribución.

**Dependencias:** T17 y todos los hitos. **Verificación:** reproducción desde checkout limpio, links y guía de rollback al commit C++; publicación solo con alcance y credenciales disponibles. **Archivos:** README/changelog/docs y retirada legacy posterior.

**Hito P4:** [ ] Paridad, presupuestos y matriz nativa aceptados; artefactos listos para distribuir.
