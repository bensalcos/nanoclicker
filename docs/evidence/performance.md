# Mediciones iniciales de rendimiento

## Cancelación del núcleo (2026-10-24)

Comando: `cargo run --release --locked -p nanoclick-core --example cancellation_latency -j 2`.

Entorno: Fedora 44, Ryzen 5 5625U, Rust 1.97.1; ver environment.md. Build release estándar, sin LTO personalizado ni target-cpu=native. Máquina con otras aplicaciones activas; primera medición exploratoria, sin control de carga/energía. El linker de este entorno imprimió un warning sobre optimización de enlazador '1' deprecada; la compilación y ejecución terminaron correctamente, sin suprimir el aviso.

4000 paradas en total: 1000 por período (1, 10, 100 y 999999 ms), con variación determinista de 0–1999 µs entre primer par y Stop. El observador espera por notificaciones, sin busy-wait. Se mide desde antes de solicitar Stop hasta observar el snapshot terminal, incluyendo overhead del observador. Backend **sin entrada de escritorio**.

[Datos CSV](core-stop-latency.csv): p99 entre 255 y 296 µs; máximo observado 1281 µs. Esto demuestra que la espera cancelable del núcleo no espera a que venza un intervalo largo. No acredita latencia de teclado, portales, SendInput, XTEST, ni liberación de botón en una plataforma real.

El presupuesto de producto de p99 ≤20 ms y máximo ≤100 ms sigue pendiente de aceptación **por backend nativo**. Arranque, CPU, memoria, jitter de inyección, tamaño comprimido y comparación C++ aún pendientes. No presentar estos números de un sink sin entrada como rendimiento completo de NanoClick.
