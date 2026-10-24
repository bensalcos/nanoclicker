# Entorno de desarrollo observado

Fecha: 2026-10-23. Sesión disponible: Fedora Linux 44 Workstation, kernel 7.2.6-200.fc44.x86_64, GNOME Wayland. CPU AMD Ryzen 5 5625U, 6 núcleos/12 hilos, RAM 14.4 GiB. La máquina tiene carga de otras aplicaciones: no es todavía una máquina de benchmark controlado.

Rust disponible: rustc/cargo 1.97.1; target instalado inicialmente x86_64-unknown-linux-gnu. Xvfb disponible para pruebas X11 aisladas. libX11, libXtst y libwayland-client disponibles.

Consulta D-Bus real (solo lectura): RemoteDesktop version=2, AvailableDeviceTypes=7 (incluye POINTER); GlobalShortcuts version=1. Esto demuestra presencia de las interfaces, no autorización, inyección ni eficacia del atajo. No se ha pedido control del escritorio del usuario.

Windows, Ubuntu 24.04, KDE, NVDA y Orca: sin evidencia nativa todavía. Registrar versiones exactas de compositor/portal y GPU al realizar mediciones.

## Capturador y aislamiento

El capturador X11 deberá ejecutarse dentro de Xvfb privado con ventana receptora enfocada y recoger ButtonPress/ButtonRelease; nunca usar DISPLAY=:0 para tests automáticos de inyección. El capturador Windows requiere escritorio/VM dedicado. Wayland requiere cliente nativo y consentimiento del escritorio; Xvfb/XWayland no sustituyen esa evidencia.

Preservar por ejecución: commit, toolchain, build release, backend, renderer, versiones, parámetros, eventos, timestamps monotónicos y resultado. La fecha y el estado de sesión no prueban rendimiento.
