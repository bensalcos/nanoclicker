# Registro de implementación

## Incremento inicial

- Toolchain fijado: Rust 1.97.1; edition 2024, MSRV declarado 1.95 (aún sin ejecutar con 1.95).
- Workspace de tres crates. Core sin unsafe ni tipos de UI/SO; configuración validada, scheduler monotónico y worker persistente.
- Primera verificación: 10 tests de core pasaron; Clippy sin warnings antes de incorporar los adaptadores. Una prueba adicional de revocación se añadió primero y falló por API ausente, antes de implementar la señal.
- Espera inicial: Condvar de std, interrumpida bajo el mismo mutex que el predicado. Es una base funcional, no evidencia de precisión de 1 ms en Windows. Comparar con waitable timers en P0/T16 antes de aceptar presupuestos.
- eframe 0.37.0 exige Rust 1.95. glow es el candidato inicial; no se ha elegido definitivamente sin comparar wgpu.
- ashpd 0.13.13: el feature remote_desktop importa tipos de screencast pero no habilita ese feature automáticamente; habilitado screencast por compilación, sin pedir captura de pantalla. Cliente async-io acotado en el backend; sin Tokio en el motor.
- Adaptadores escritos: SendInput por par, XTEST y portal RemoteDesktop con cleanup. Compilación Windows y prueba nativa todavía pendientes.
- UI inicial inglesa implementada con lógica/control separada y redibujado por notificaciones. La pantalla provisional identifica el atajo Wayland como pendiente; no se considera T12 terminado.

Fuentes de APIs: fuentes/documentación versionadas de eframe/egui 0.37.0, ashpd 0.13.13, global-hotkey 0.9.0 y x11rb 0.13.2 en el registro de Cargo; enlaces primarios generales en docs/architecture/RUST.md. Los resultados finales de tests y probes se añaden tras ejecutar cada comprobación.

## Evidencia posterior

- `cargo check --workspace --target x86_64-pc-windows-msvc --locked -j 2`: terminó correctamente; se corrigió después un warning de campo ctx solo usado en Linux. No es build enlazado ni prueba nativa Windows.
- `scripts/verify-x11.sh`: PASS para 10 pares completos Left/Middle/Right capturados en orden en Xvfb; PASS atajo fuera de foco, supresión de repetición tras sostener F6 900 ms y conservación del registro original al rechazar conflicto.
- Reemplazado el listener global-hotkey de Linux (polling de 50 ms observado en su fuente) por listener X11 propio con poll de socket/pipe y apagado por señal; sin polling periódico. global-hotkey continúa como parser y como implementación Windows (MOD_NOREPEAT).
- Primer screenshot Xvfb revisado: el bloque de atajos requería scroll a 520×600. Ajustado espaciado; captura final pendiente. Ninguna captura Xvfb sustituye QA nativo/DPI/lector.
- Núcleo tras recuperación de panic y señal de revocación: 13 tests pasan, incluidos 200 ciclos Start/Stop y recuperación de un backend que entra en panic. Una prueba reprodujo primero la UI quedando en Running al morir el worker; ahora se publica Faulted y el worker permite reintento.

## Verificación del primer corte integrado (2026-10-24)

- `cargo test --workspace --locked -j 2`: 16 pruebas pasaron (14 core, 2 controller). Incluye regresión del primer contador visible durante intervalo largo.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS Linux.
- `cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --locked -- -D warnings`: PASS. El ejemplo de captura X11 se activa únicamente con feature x11-tests en la matriz Linux; no es una prueba Windows omitida.
- Capturas finales Xvfb de 520×600 en `screenshots/linux-x11-{dark,light}.png`: revisadas visualmente; tres bloques completos, contraste de texto mejorado, acción principal visible. No acredita DPI, teclado ni lectores nativos.
- El código conserva rough edges identificados: preferencias aún sin persistir, artefactos/CI hospedado y benchmarks no realizados, consentimiento/denegación/revocación real Wayland y Windows/KDE pendientes. No se declara paridad aceptada ni release listo.

## Preparación manual después del permiso

La cuenta regresiva inicial vivía en controller y se consumía antes de que el worker solicitara permisos Wayland. Se trasladó al motor como una espera opcional después de preparar el sink. El snapshot publica ready_at durante esa espera; los atajos usan demora cero. Stop/cierre/revocación interrumpen la espera y el sink finaliza antes del acuse.

Tres pruebas adicionales cubren deadline posterior a preparación, limpieza al cancelar y rechazo de demora desmesurada. Suite total: 27 tests pasan. Clippy Linux pasa; prueba gráfica del ejecutable X11 volvió a pasar con la nueva secuencia. Esta prueba de protocolo no equivale a consentir un portal real Wayland.
