//! Event-driven X11 shortcut listener with a shutdown pipe, without polling.
use global_hotkey::hotkey::{HotKey, Modifiers};
use nanoclick_core::BackendError;
use rustix::event::{PollFd, PollFlags, poll};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    sync::mpsc,
    thread::JoinHandle,
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event, xkb,
        xproto::{ConnectionExt, GrabMode, ModMask},
    },
};

pub struct X11Shortcut {
    shutdown: UnixStream,
    worker: Option<JoinHandle<()>>,
}
fn error(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!("X11 shortcut: {error}"))
}
impl X11Shortcut {
    /// Registration is completed before returning, preserving the previous grab
    /// when a replacement conflicts. The caller owns at most one listener.
    pub fn register(
        hotkey: HotKey,
        activated: impl Fn() + Send + 'static,
        failed: impl Fn(String) + Send + 'static,
    ) -> Result<Self, BackendError> {
        let (shutdown, mut interrupted) = UnixStream::pair().map_err(error)?;
        let (ready, response) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("nanoclick-x11-shortcut".into())
            .spawn(move || {
                let setup = (|| {
                    let (conn, screen) = x11rb::connect(None).map_err(error)?;
                    let root = conn.setup().roots[screen].root;
                    let symbol = keysym(hotkey)?;
                    let first = conn.setup().min_keycode;
                    let count = conn.setup().max_keycode - first + 1;
                    let map = conn
                        .get_keyboard_mapping(first, count)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?;
                    let code = map
                        .keysyms
                        .chunks(usize::from(map.keysyms_per_keycode))
                        .position(|symbols| symbols.contains(&symbol))
                        .and_then(|offset| u8::try_from(usize::from(first) + offset).ok())
                        .ok_or_else(|| {
                            BackendError::new(
                                "The requested key is not present in the current keyboard layout.",
                            )
                        })?;
                    xkb::ConnectionExt::xkb_use_extension(&conn, 1, 0)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?;
                    let repeat = xkb::ConnectionExt::xkb_per_client_flags(
                        &conn,
                        xkb::ID::USE_CORE_KBD.into(),
                        xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
                        xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    )
                    .map_err(error)?
                    .reply()
                    .map_err(error)?;
                    if !repeat
                        .value
                        .contains(xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT)
                    {
                        return Err(BackendError::new(
                            "The X11 server cannot suppress shortcut key repeat.",
                        ));
                    }
                    let mut modifiers = ModMask::default();
                    for (flag, mask) in [
                        (Modifiers::CONTROL, ModMask::CONTROL),
                        (Modifiers::SHIFT, ModMask::SHIFT),
                        (Modifiers::ALT, ModMask::M1),
                        (Modifiers::META, ModMask::M4),
                    ] {
                        if hotkey.mods.contains(flag) {
                            modifiers |= mask;
                        }
                    }
                    // Ignore CapsLock and NumLock. Partial grabs are released when
                    // this private connection closes on any registration failure.
                    for ignored in [
                        ModMask::default(),
                        ModMask::LOCK,
                        ModMask::M2,
                        ModMask::LOCK | ModMask::M2,
                    ] {
                        conn.grab_key(
                            false,
                            root,
                            modifiers | ignored,
                            code,
                            GrabMode::ASYNC,
                            GrabMode::ASYNC,
                        )
                        .map_err(error)?
                        .check()
                        .map_err(error)?;
                    }
                    conn.flush().map_err(error)?;
                    Ok((conn, code))
                })();
                let (conn, keycode) = match setup {
                    Ok(ready_state) => {
                        let _ = ready.send(Ok(()));
                        ready_state
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                let result = (|| -> Result<(), BackendError> {
                    let mut held = false;
                    loop {
                        while let Some(event) = conn.poll_for_event().map_err(error)? {
                            match event {
                                Event::KeyPress(event) if event.detail == keycode && !held => {
                                    held = true;
                                    activated();
                                }
                                Event::KeyRelease(event) if event.detail == keycode => held = false,
                                Event::MappingNotify(_) => {
                                    return Err(BackendError::new(
                                        "Keyboard mapping changed. Apply the shortcut again.",
                                    ));
                                }
                                _ => {}
                            }
                        }
                        let mut descriptors = [
                            PollFd::new(conn.stream(), PollFlags::IN),
                            PollFd::new(&interrupted, PollFlags::IN),
                        ];
                        if let Err(errno) = poll(&mut descriptors, None) {
                            if errno == rustix::io::Errno::INTR {
                                continue;
                            }
                            return Err(error(errno));
                        }
                        if descriptors[1]
                            .revents()
                            .intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR)
                        {
                            let mut byte = [0];
                            let _ = interrupted.read(&mut byte);
                            return Ok(());
                        }
                        if descriptors[0]
                            .revents()
                            .intersects(PollFlags::HUP | PollFlags::ERR)
                        {
                            return Err(BackendError::new("X11 shortcut connection closed."));
                        }
                    }
                })();
                if let Err(error) = result {
                    failed(error.to_string());
                }
            })
            .map_err(error)?;
        match response.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(Self {
                shutdown,
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(error) => {
                let _ = shutdown.shutdown(std::net::Shutdown::Both);
                let _ = worker.join();
                Err(BackendError::new(format!(
                    "Shortcut registration did not finish: {error}"
                )))
            }
        }
    }
}
impl Drop for X11Shortcut {
    fn drop(&mut self) {
        let _ = self.shutdown.write_all(&[1]);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn keysym(hotkey: HotKey) -> Result<u32, BackendError> {
    let name = format!("{:?}", hotkey.key);
    if let Some(letter) = name.strip_prefix("Key")
        && letter.len() == 1
    {
        return Ok(u32::from(letter.as_bytes()[0].to_ascii_lowercase()));
    }
    if let Some(digit) = name.strip_prefix("Digit")
        && digit.len() == 1
    {
        return Ok(u32::from(digit.as_bytes()[0]));
    }
    if let Some(number) = name.strip_prefix('F').and_then(|n| n.parse::<u32>().ok())
        && (1..=24).contains(&number)
    {
        return Ok(0xffbd + number);
    }
    match name.as_str() {
        "Space" => Ok(0x20),
        "Enter" => Ok(0xff0d),
        "Tab" => Ok(0xff09),
        "Escape" => Ok(0xff1b),
        "Home" => Ok(0xff50),
        "End" => Ok(0xff57),
        "PageUp" => Ok(0xff55),
        "PageDown" => Ok(0xff56),
        "Insert" => Ok(0xff63),
        "Delete" => Ok(0xffff),
        "ArrowLeft" => Ok(0xff51),
        "ArrowRight" => Ok(0xff53),
        "ArrowUp" => Ok(0xff52),
        "ArrowDown" => Ok(0xff54),
        _ => Err(BackendError::new(
            "Use a letter, digit, F1–F24, Space, Enter, Tab, Escape, navigation, Insert or Delete key.",
        )),
    }
}
