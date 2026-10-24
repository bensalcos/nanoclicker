# Baseline de paridad y rendimiento

La referencia funcional es el C++ del commit 3fd5cd8. Baseline Windows de rendimiento pendiente: este entorno es Linux y no se ejecutó el binario C++.

Fixtures: intervalo 1/10/100/999999 ms; botones Left/Right/Middle; infinito y conteos 1/10/999999; entradas cero/fuera de rango; inicio repetido; Stop durante espera larga; cierre activo; error parcial down/up; atajo ocupado/sostenido; fin automático seguido de reinicio. Los fixtures grandes se verifican con tiempo falso en core y una muestra acotada en integración nativa.

Diferencias intencionadas: UI solo en inglés; validación explícita; inicio manual con 3 s de preparación cancelable; omitir slots vencidos; verificar envío y recuperación ante fallo. Conservar primer clic inmediato desde atajo y conteo de pares completos.

No afirmar mejora respecto a C++ sin baseline comparable. Medir el motor, UI y destino por separado según docs/VALIDATION.md.
