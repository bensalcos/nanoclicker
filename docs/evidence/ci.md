# CI configurado, aceptación hospedada pendiente

Workflow: `.github/workflows/ci.yml`. Matriz Windows 2025 / Ubuntu 24.04 para fmt, Clippy, tests y build release; integración X11 aislada y logs; jobs separados para renderer wgpu y MSRV 1.95.

Los jobs no envían clics a un escritorio compartido. El de Windows ejecuta pruebas de core/controller y compila el adaptador, pero todavía no valida SendInput nativo con un destino ni lector de pantalla. Xvfb no acredita Wayland.

Acciones fijadas por commits obtenidos de los tags públicos el 2026-10-24:

- [actions/checkout v7.0.1](https://github.com/actions/checkout/tree/3d3c42e5aac5ba805825da76410c181273ba90b1)
- [actions/upload-artifact v7](https://github.com/actions/upload-artifact/tree/cf430e030ddbb5b0abf93d22962f4752f3646cd9)

Se consultaron los README oficiales vigentes, en lugar de copiar versiones antiguas de ejemplos de skills. Permisos contents:read y sin persistir credenciales del checkout. No se ha hecho push ni disparado un workflow remoto; no existe evidencia de CI hospedado exitoso. El presupuesto de runners se usa solo cuando el workflow se ejecute en GitHub.
