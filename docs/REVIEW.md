# Revisión del proyecto existente

Revisión estática: 2026-10-23. Base: `3fd5cd8`. No se ejecutó el binario Windows ni se midió rendimiento. El árbol estaba limpio al comenzar. No hay tests ni automatización de CI versionados.

## Inventario y paridad

| Componente | Implementación actual | Comportamiento que preservar |
|---|---|---|
| Arranque | `Autoclicker.cpp`, Win32 y common controls | Una ventana de escritorio |
| Motor | `AutoclickerEngine.{h,cpp}`, hilo persistente | Primer clic inmediato, intervalo en ms, cursor actual |
| Entrada | `SendInput`, down/up separados | Izquierdo, derecho y central |
| Parada | Flag atómico y conteo `int` | Manual o después de N clics completos |
| UI | `MainWindow.{h,cpp}` | Configuración bloqueada durante ejecución |
| Atajo | `RegisterHotKey`, F6 por defecto | Alternar inicio/parada fuera de foco |
| Idioma | Cadenas ES/EN dentro de la ventana | La reescritura será solo en inglés por decisión del usuario |
| Recursos | `.rc`, dos iconos, menú | Identidad NanoClick |
| Build | `.vcxproj`, `.slnx` | Actualmente solo Windows, toolset v145, C++20 |

Valores iniciales: intervalo 100 ms, conteo 10, modo infinito, botón izquierdo, español y F6. No existe configuración persistente que importar. No hay posición fija, macros, doble clic, bandeja ni actualizador implementados.

## Hallazgos

1. **Concurrencia:** los campos de configuración se escriben en `Start()` sin el mutex usado por el worker; ciclos rápidos Stop/Start pueden solaparse con lecturas de la ejecución anterior. El protocolo de condición tampoco protege con el mismo mutex todas las transiciones del predicado: existe riesgo de perder una notificación. Rediseñar ownership y sincronización; no trasladar este esquema a Rust.
2. **Cancelación:** `Stop()` no despierta `Sleep(clickRate)`. El cierre hace `join()` y puede bloquear hasta el intervalo restante, que la UI permite llevar a 999999 ms.
3. **Fin automático:** el worker pone `isRunning=false` antes del callback; `StopAutoclicker()` retorna si ya está detenido y no llama `UpdateControlStates()`. Los controles pueden quedar deshabilitados.
4. **Inyección:** se ignoran los resultados de ambas llamadas `SendInput`. Se cuentan intentos aunque fallen; un down aceptado y un up fallido necesitan recuperación explícita.
5. **Temporización:** `Sleep(intervalo)` añade el coste de inyección al período y acumula deriva. `timeBeginPeriod(1)` se mantiene toda la vida del proceso y su resultado se ignora. Resolución de reloj no equivale a precisión garantizada.
6. **Atajos:** registro inicial y cambios ignoran errores; se elimina el atajo anterior antes de comprobar el nuevo. Se consulta el control cada 500 ms. Falta supresión de repetición de tecla.
7. **Validación:** `_wtoi` no distingue entradas inválidas; intervalo cero se convierte silenciosamente a uno; un conteo cero termina después del primer intento. La nueva versión rechazará explícitamente valores inválidos.
8. **UI y recursos:** distribución por coordenadas y altura mínima fija; falta evidencia de DPI, teclado y lector de pantalla. El brush creado para la clase no tiene liberación visible. El menú About está declarado en recursos pero no tiene acción implementada en la ventana.
9. **Documentación/build:** README menciona `.sln` aunque existe `.slnx`, C++17 aunque el proyecto usa C++20 y recomienda VS2022 aunque configura v145. Las afirmaciones de cero lag y precisión no tienen mediciones.
10. **Licencia:** MIT con `[year] [fullname]` sin completar; conservar avisos y resolver titularidad antes de distribuir.

## Consecuencia para la migración

Reescritura pequeña y modular, sin FFI hacia el motor C++. Mantener la versión existente como referencia hasta validar paridad. La sustitución de lenguaje por sí sola no prueba una mejora frente a C++: el objetivo medible es reducir deriva, latencia de parada y consumo, además de ampliar compatibilidad.
