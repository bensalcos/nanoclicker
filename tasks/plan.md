# Plan de reescritura de NanoClick en Rust

## Objetivo

Conservar funciones del autoclicker C++ y entregar UI moderna en Rust para Windows y Linux, con parada rápida, temporización estable y consumo medido. Esta primera entrega define el proyecto; la implementación comienza por pruebas técnicas pequeñas, antes de comprometer el motor completo.

Documentos normativos: [revisión](../docs/REVIEW.md), [arquitectura](../docs/architecture/RUST.md), [UI](../docs/UI.md), [validación](../docs/VALIDATION.md). Tareas y aceptación: [todo.md](todo.md). Estado actual: [STATUS.md](../docs/STATUS.md).

## Fases e hitos

| Fase | Tareas | Resultado / condición de salida |
|---|---|---|
| P0 — Reducir incertidumbre | T01–T04 | Renderer seleccionado y evidencia real de Windows, X11 y portales; limitaciones enumeradas |
| P1 — Primera función completa | T05–T08 | Iniciar/detener botón izquierdo desde UI en Windows/X11, espera cancelable y CI |
| P2 — Paridad y Linux moderno | T09–T12 | N exacto, tres botones, atajos, UI en inglés y Wayland integrado |
| P3 — Experiencia y recuperación | T13–T15 | Preferencias, accesibilidad, fallos y lifecycle aceptados |
| P4 — Optimización y entrega | T16–T18 | Mediciones aceptadas, artefactos reproducibles y documentación de usuario |

Cada hito requiere evidencia de sus criterios, no solo tareas marcadas. No declarar Linux completo si solo funciona X11. Si un portal está ausente, registrar bloqueo por entorno y continuar tareas independientes; no sustituir automáticamente por acceso privilegiado.

## Orden y dependencias

```text
T01 ─ T02/T03/T04 (pruebas independientes)
 └─ T05 ─ T06 ─ T07 ─ T08 ─ T09 ─ T10 ─ T11
             T04 + T08 + T10 ─ T12
             T11 + T12 ─ T13 ─ T14 ─ T15 ─ T16 ─ T17 ─ T18
```

T05 puede avanzar cuando la UI de T02 esté decidida; T07 requiere T03. Los spikes de P0 son descartables y no congelan APIs por conveniencia. No mantener dos motores conectados mediante FFI. Implementar cortes verticales verificables; conservar C++ compilable hasta aceptación final. Después retirar fuentes/build antiguos en un cambio separado con referencia al commit histórico.

## Riesgos y mitigaciones

| Riesgo | Tratamiento |
|---|---|
| Portales incompletos o lentos | Probar primero GNOME/KDE, capacidades explícitas y métricas por backend |
| Renderer consume demasiado o falla en GPU | Prototipo glow/wgpu, elegir uno y conservar criterio de revisión |
| UI bonita pero inaccesible | Navegación y AccessKit desde prototipo; NVDA/Orca antes de aceptación |
| Optimización empeora energía/parada | Espera cancelable sin spin; medir CPU, jitter y Stop juntos |
| Rust introduce complejidad innecesaria | Tres crates, std para worker, executor solo si plataforma lo requiere |
| Sin escritorio Windows disponible | Tests de core/CI avanzan; soporte nativo permanece pendiente de evidencia |
| Aplicación destino rechaza eventos | Reportar semántica de backend, sin prometer recepción ni eludir restricciones |

## Decisiones pendientes de evidencia

- Conjunto de versiones, MSRV y dependencias por target en T01/T05.
- glow frente a wgpu en T02; Slint solo si fracasa el criterio de UI.
- Mecanismo exacto de espera cancelable e integración portal en T03/T04.
- Distro/versiones KDE y máquinas de referencia en T01.
- Presupuestos: medirlos en T16; cualquier modificación requiere decisión documentada con resultados.

Ninguna de estas decisiones exige una pregunta previa para comenzar las pruebas técnicas. Datos de firma/publicación y titularidad de licencia se necesitan únicamente para distribuir.
