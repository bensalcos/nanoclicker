//! Thread-bound WM_HOTKEY listener; native event shutdown and no key polling.
use global_hotkey::hotkey::{HotKey, Modifiers};
use nanoclick_core::BackendError;
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr::{null, null_mut},
    sync::{Arc, mpsc},
    thread::JoinHandle,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::{CreateEventW, INFINITE, SetEvent},
    UI::{
        Input::KeyboardAndMouse::{
            MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey,
            UnregisterHotKey,
        },
        WindowsAndMessaging::{
            MSG, MsgWaitForMultipleObjects, PM_NOREMOVE, PM_REMOVE, PeekMessageW, QS_ALLINPUT,
            WM_HOTKEY, WM_QUIT, WM_USER,
        },
    },
};

pub struct WindowsShortcut {
    shutdown: Arc<OwnedHandle>,
    worker: Option<JoinHandle<()>>,
}
impl WindowsShortcut {
    pub fn register(
        key: HotKey,
        activated: impl Fn() + Send + 'static,
        failed: impl Fn(String) + Send + 'static,
    ) -> Result<Self, BackendError> {
        let code = crate::windows_key::virtual_key(key)?;
        let mut modifiers = MOD_NOREPEAT;
        for (flag, mask) in [
            (Modifiers::CONTROL, MOD_CONTROL),
            (Modifiers::SHIFT, MOD_SHIFT),
            (Modifiers::ALT, MOD_ALT),
            (Modifiers::META, MOD_WIN),
        ] {
            if key.mods.contains(flag) {
                modifiers |= mask;
            }
        }
        // SAFETY: default security, unnamed manual-reset event, initially clear.
        let handle = unsafe { CreateEventW(null(), 1, 0, null()) };
        if handle.is_null() {
            return Err(BackendError::new(format!(
                "Could not create shortcut shutdown event: {}",
                std::io::Error::last_os_error()
            )));
        }
        // SAFETY: CreateEventW returned a new owned kernel handle. OwnedHandle
        // closes it exactly once, after the worker's Arc also releases ownership.
        let shutdown = Arc::new(unsafe { OwnedHandle::from_raw_handle(handle) });
        let interrupted = shutdown.clone();
        let (ready, response) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("nanoclick-windows-shortcut".into())
            .spawn(move || {
                let mut message = MSG::default();
                // SAFETY: initialized writable MSG; Peek creates this private queue.
                unsafe {
                    PeekMessageW(&mut message, null_mut(), WM_USER, WM_USER, PM_NOREMOVE);
                }
                // SAFETY: NULL binds to this thread; validated key/modifiers, ID 1
                // belongs exclusively to this worker for the lifetime of its queue.
                if unsafe { RegisterHotKey(null_mut(), 1, modifiers, code) } == 0 {
                    let _ = ready.send(Err(BackendError::new(format!(
                        "Windows shortcut unavailable: {}",
                        std::io::Error::last_os_error()
                    ))));
                    return;
                }
                if ready.send(Ok(())).is_ok() {
                    'messages: loop {
                        let handle = interrupted.as_raw_handle();
                        // SAFETY: one valid event handle kept alive by this Arc;
                        // messages and the event wake the same wait without polling.
                        let result = unsafe {
                            MsgWaitForMultipleObjects(1, &handle, 0, INFINITE, QS_ALLINPUT)
                        };
                        if result == WAIT_OBJECT_0 {
                            break;
                        }
                        if result == WAIT_FAILED {
                            failed(format!(
                                "Windows shortcut wait failed: {}",
                                std::io::Error::last_os_error()
                            ));
                            break;
                        }
                        // SAFETY: message remains writable; this thread owns its queue.
                        while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) }
                            != 0
                        {
                            if message.message == WM_QUIT {
                                break 'messages;
                            }
                            if message.message == WM_HOTKEY && message.wParam == 1 {
                                activated();
                            }
                        }
                    }
                }
                // SAFETY: same thread and ID as the successful registration.
                if unsafe { UnregisterHotKey(null_mut(), 1) } == 0 {
                    failed(format!(
                        "Could not release the Windows shortcut: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            })
            .map_err(|e| BackendError::new(format!("Could not create shortcut worker: {e}")))?;
        match response.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(Self {
                shutdown,
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(error) => {
                // SAFETY: this Arc owns a valid manual-reset event, including
                // the race where the worker has just completed registration.
                unsafe {
                    SetEvent(shutdown.as_raw_handle());
                }
                drop(response);
                let _ = worker.join();
                Err(BackendError::new(format!(
                    "Shortcut registration did not finish: {error}"
                )))
            }
        }
    }
}
impl Drop for WindowsShortcut {
    fn drop(&mut self) {
        // SAFETY: event lifetime is shared with the worker. An event avoids
        // posting to a possibly recycled thread ID after an unexpected exit.
        unsafe {
            SetEvent(self.shutdown.as_raw_handle());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
