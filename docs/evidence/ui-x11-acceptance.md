# Aceptación gráfica aislada X11

Fecha: 2026-10-24. Comando: `scripts/verify-ui-x11.sh`. Binario de la aplicación real, build debug, tema oscuro, ventana 520×600, configuración temporal. Xvfb privado 1024×768×24, sin interacción con el escritorio del usuario.

Resultados observados:

- PASS: F6 inicia y detiene con el foco en una ventana receptora externa a NanoClick.
- PASS: el destino recibe pares down/up completos; después de observar la vuelta de la UI a Ready no recibe eventos adicionales durante 250 ms.
- PASS: click real en Start, mover cursor al destino y esperar; no se omite la preparación de tres segundos, después se reciben pares en el destino.
- PASS: cierre mediante WM_DELETE_WINDOW termina el proceso dentro del timeout.

El harness espera la ventana y el estado visual de la acción principal antes de actuar. Las coordenadas/colores son fixtures de la ventana por defecto en Xvfb x86_64; no se presentan como prueba general de todos los DPI ni layouts. La cuenta finita/tres botones se cubre aparte con core y capturador de plataforma.

Evidencia más amplia que unit tests de controller, pero no acredita un compositor X11 real, Wayland, Windows, readers, DPI ni todos los estados de error. El workflow preserva los logs cuando se ejecute hospedado; todavía no hay run de GitHub.
