# UI de NanoClick Rust

Especificación inicial; requiere prototipo visual y QA nativo antes de considerarse aceptada.

La nueva aplicación será únicamente en inglés: controles, estados, errores, tooltips y nombres accesibles. No tendrá selector de idioma ni módulo de traducciones. Esta decisión reemplaza el comportamiento bilingüe de la versión C++.

## Flujo principal

Una ventana compacta, tamaño inicial 520×600 unidades lógicas, mínimo 420×520 con scroll si el contenido no cabe. Cabecera con marca NanoClick, selector de tema sistema/claro/oscuro y botón Save para guardar preferencias de forma explícita. Tres bloques: ritmo y botón, condición de parada, atajo. Pie con estado, contador de sesión y acción principal persistente. Respetar decoraciones del SO para mover, redimensionar y cerrar.

```text
NanoClick                              Theme
Ready                         Windows / X11 / Wayland

Timing
Interval [100          ] ms     ≈ 10 clicks/s
Button    [Left ▾]

Repeat
(●) Until stopped   ( ) Count [10        ]

Global shortcut
[F6                       Change]  Registered

Session: 0 pairs sent
[                 Start                   ]
```

En Running, la acción principal cambia a **Stop**, se bloquea edición del plan activo y permanece posible cambiar el tema. En Preparing, mostrar solicitud de permiso y botón Cancel. En Stopping, mantener estado visible hasta acuse del motor. Al completar N pares, restaurar controles inmediatamente y mostrar “Completed”. Faulted incluye causa comprensible y recuperación, conservando valores.

El botón Start no debe dejar que el primer clic inmediato reactive la propia UI por estar el cursor encima: tras pulsar, preparar primero la entrada y completar cualquier permiso del escritorio; después mostrar instrucción de mover el cursor y una cuenta regresiva cancelable de 3 segundos para inicio manual. El inicio por atajo conserva primer clic inmediato. La demora manual es una mejora deliberada respecto a C++; medir latencia de motor desde fin de cuenta regresiva, separada de la experiencia de arranque.

No añadir navegación lateral, gráficos de CPS en vivo, animaciones continuas, tray, macros o perfiles múltiples en V1. No minimizar al arrancar. El atajo mostrado debe ser el realmente registrado, con mensajes de conflicto y alternativa manual cuando falte soporte.

## Sistema visual

- Espaciado base 8, márgenes 24, separación de bloques 20; radios 10 y bordes discretos.
- Tipografía legible de 14–16 unidades lógicas, título 24, cifras con alineación estable. Fuente incluida solo si su licencia permite redistribución.
- Tema oscuro: fondo grafito, superficies elevadas suaves y acento azul; tema claro equivalente. Reservar rojo para Detener/error, sin depender únicamente del color.
- Labels permanentes, unidades visibles, error junto al campo, sin corregir silenciosamente entradas inválidas. Cantidad e intervalo enteros; tooltip del período objetivo y límites.
- Estado textual y visual: listo, preparando, activo, deteniendo, completado, error o capacidad no disponible. Evitar presentar precisión teórica como CPS observado.

## Accesibilidad y QA

Tab/Shift+Tab en orden de lectura; foco visible; Space/Enter solo en el control adecuado; Escape cancela captura de atajo o preparación y detiene una ejecución activa cuando la ventana tiene foco. No hacer Start por Enter mientras se edita un campo. Contraste de texto ≥4.5:1, controles identificables y nombres accesibles en inglés. Desactivar movimiento innecesario.

Probar 100%, 150% y 200% DPI; ventana mínima; mensajes largos; teclado completo; NVDA en Windows y Orca en Linux. AccessKit habilitado no demuestra por sí solo que estas pruebas pasan. Guardar capturas y resultados por SO/tema; aprobar la apariencia en escritorio real, no solo compilación ni headless.

Preferencias persistidas al pulsar Save: tema, configuración y atajo solicitado. El resultado del guardado se muestra en la ventana. Archivo versionado con escritura temporal y reemplazo seguro específico del SO; recuperación de archivo corrupto sin sobrescribirlo automáticamente. Usar directorios de configuración estándar. Nunca persistir estado Running, permisos como si fueran vigentes ni contadores que reactiven clics al abrir.

## Implemented layout and accessibility update

The initial 520×600 size and 420×520 minimum remain. Secondary padding is 14×6 with a 30-unit minimum target; the primary action remains 46 units high. Redundant section spacing was removed so help fits at default size. A nonfloating scrollbar appears when needed at smaller sizes.

The body is created before the footer button for reading-order keyboard traversal; footer space is reserved from wrapped status/progress measurements. Fields/selectors have explicit English accessible names and values. Session status is a polite atomic live region; the counter is not live. Palette tokens have automated text/control/focus contrast checks. These implementation checks do not close native DPI/NVDA/Orca acceptance; see evidence/ui-accessibility.md and evidence/ui-scaling.md.
