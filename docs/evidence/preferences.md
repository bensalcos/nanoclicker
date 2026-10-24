# Preferencias versionadas

Esquema v1: intervalo, botón, repetición finita/infinita, tema y atajo solicitado. Guardado explícito mediante Save; lectura al arrancar, sin persistir sesiones activas, contadores ni permisos vigentes.

Ubicación seleccionada mediante directories 6.0.0 (directorios estándar del SO). Escritura temporal en el mismo directorio, sync_all del contenido, publicación con tempfile 3.27.0 `persist` (reemplazo atómico según su contrato) y sincronización del directorio en Unix. Un error de publicación no elimina el destino. No se promete persistencia ante todos los fallos de hardware o filesystem.

La lectura está limitada a 64 KiB. Se rechazan versiones desconocidas, campos adicionales y valores inválidos. Si la lectura falla se usan defaults para esta sesión y se protege el original contra reemplazo; el usuario puede moverlo aparte y reiniciar. No hay sobrescritura automática.

Evidencia local: seis tests pasan en Linux para roundtrip, reemplazo válido, corrupción/versión futura, ausencia de estado activo, fallo de ruta, fallo real de persist sobre destino incompatible y tamaño/valores inválidos. Suite total: 22 tests. Clippy del workspace para Windows también pasó, pero el reemplazo de archivos aún no tiene prueba nativa Windows.

Los scripts de captura y aceptación gráfica usan XDG_CONFIG_HOME temporal, evitando leer preferencias personales en pruebas/capturas.
