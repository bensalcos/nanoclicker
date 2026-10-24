# Atajo Windows por eventos

Revisión de global-hotkey 0.9.0: su ruta Windows crea un hilo por activación que consulta GetAsyncKeyState cada 50 ms hasta liberación. NanoClick ya no usa ese listener; conserva la biblioteca como parser de HotKey.

Implementado un worker propio en platform: RegisterHotKey asociado al hilo con MOD_NOREPEAT, cola de mensajes y MsgWaitForMultipleObjects. La espera atiende mensajes y un evento manual de apagado. OwnedHandle/Arc mantienen vivo el evento hasta terminar el worker; el cierre no depende de polling ni de enviar mensajes a un thread ID que podría haberse reciclado. Los bloques FFI incluyen precondiciones SAFETY.

Las actualizaciones de cada registro tienen su propio estado compartido: un callback tardío del registro sustituido no invalida el nuevo. Un conflicto conserva el registro anterior.

Fuentes primarias consultadas:

- [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey): cola de hilo con hWnd NULL y MOD_NOREPEAT.
- [MsgWaitForMultipleObjects](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjects): espera de handles y mensajes; no cerrar handles durante una espera pendiente.
- [Virtual-key codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes): mapeo explícito de letras, dígitos, F1–F24 y navegación.

Verificado: dos pruebas puras de mapeo pasan en Linux; suite total 24 tests; cargo check y Clippy del workspace Windows MSVC pasan. La prueba gráfica X11 pasó de nuevo tras refactorizar el despacho compartido. **No hay prueba nativa Windows del message loop, conflictos o teclas sostenidas**; T11/T15 permanecen pendientes de esa evidencia.
