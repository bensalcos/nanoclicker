# Calidad, rendimiento y aceptación

Todos los números siguientes son **presupuestos iniciales pendientes de medición**, no resultados ni garantías de tiempo real. No reducirlos silenciosamente: documentar datos, causa y decisión si requieren revisión.

## Presupuestos

| Medida | Objetivo inicial | Condiciones |
|---|---|---|
| Arranque cálido hasta ventana utilizable | p95 ≤500 ms | 30 aperturas release, sin diálogo de permiso |
| Memoria en reposo | RSS/private working set ≤100 MiB | 60 s estabilizado, registrar SO y renderer |
| CPU en reposo | ≤1% de un núcleo | Media 60 s, sin animaciones ni interacción |
| CPU activo a 100 clics/s | ≤5% de un núcleo | Media 60 s, proceso completo; registrar backend |
| Stop hasta acuse del motor | p99 ≤20 ms, máximo observado ≤100 ms | 1000 paradas aleatorias, intervalos 1/10/100/999999 ms |
| Retraso de deadline a 10 ms de período | p99 ≤2 ms | 60 s sin carga y 60 s con carga documentada |
| Binario portable comprimido | ≤25 MiB | Por plataforma, listar librerías requeridas aparte |

Aplicar la misma tabla a cada backend. Un incumplimiento Wayland no queda exento por usar D-Bus: requiere optimización o decisión explícita de capacidad/rango admitido. El intervalo mínimo configurable de 1 ms no promete 1000 clics/s sostenidos; mostrar período solicitado y resultado medido por separado.

Máquinas de referencia: x86_64, al menos 4 núcleos y 8 GiB; registrar CPU exacta, RAM, GPU/driver, SO/kernel, escritorio/compositor/portal, energía, carga y commit. Para cada medida separar cold/warm, depuración/release y motor/UI/destino. Medir el proceso completo para CPU/RAM, no solo un microbenchmark.

## Estrategia de pruebas

1. **Core determinista:** reloj/espera/inyección falsos; validación, N exacto, primer clic, slots omitidos, overflow, inicio repetido, parada en preparación y ejecución, Shutdown y errores parciales. Probar conteos sin esperar tiempo real.
2. **Concurrencia:** carreras Start/Stop/Start y cierre, wakeup perdido, canal lleno, snapshot terminal y eventos de run_id anterior; tests de estrés acotados. Usar Loom solo si aporta cobertura al protocolo propio de sincronización.
3. **Adaptadores:** destinos de prueba aislados capturan botón, orden down/up y conteo. No inyectar sobre escritorios de CI compartidos. Xvfb aporta evidencia X11 aislada, no soporte Wayland ni QA gráfico nativo.
4. **Fallos:** atajo ocupado, pérdida de servidor X/D-Bus, portal ausente, permiso denegado/revocado, fallo de liberación y configuración corrupta. El estado Running no puede sobrevivir a un backend inválido.
5. **UI:** textos y mensajes en inglés, fin de conteo habilita controles, cuenta regresiva cancelable, DPI, teclado y lectores de pantalla según UI.md.
6. **Rendimiento:** benchmarks release del scheduler y mediciones end-to-end; histogramas acotados, p50/p95/p99/máximo y slots omitidos. Diferenciar timestamp de llamada de inyección y recepción en capturador; ninguna prueba demuestra comportamiento de todas las aplicaciones.

Cuando exista workspace, checks esperados por target admitido:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
cargo bench --locked
```

Añadir matriz de features válidas (renderer elegido y backend por target); `--all-features` solo si no combina opciones mutuamente excluyentes. Mantener check de MSRV, auditoría de dependencias/licencias y símbolos de diagnóstico. Miri para core si las dependencias lo permiten; no considerarlo prueba de FFI del SO.

## Matriz de entrega

| Plataforma | Build/test CI | Prueba nativa requerida |
|---|---|---|
| Windows 11 x64 MSVC | fmt, clippy, tests, release | SendInput, hotkey fuera de foco, DPI, NVDA, cierre |
| Ubuntu 24.04 x64 X11 | tests, release, integración aislada | XTEST, conflictos, layouts, Orca |
| Ubuntu GNOME Wayland | build Linux y tests de contrato | Portales, cliente Wayland nativo, permisos y revocación |
| KDE Plasma Wayland | build Linux y tests de contrato | Mismos flujos; registrar distro/versiones en P0 |

Artefactos iniciales: ZIP Windows con ejecutable y licencias; tar.gz Linux con binario, `.desktop`, icono y lista de requisitos. AppImage/MSI/Flatpak quedan posteriores a verificar necesidad y compatibilidad; no llamar “portable sin dependencias” a un binario Linux sin comprobar linkage y baseline glibc.

Release solo después de paridad, matriz nativa y presupuestos aceptados, inventario de licencias, checksums y guía reproducible de compilación. Revisar procedencia del icono y completar aviso MIT con datos confirmados. Firma de código y publicación requieren credenciales/alcance explícito; preparar artefactos localmente sin bloquear la implementación por ello.
