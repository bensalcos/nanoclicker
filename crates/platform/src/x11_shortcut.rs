//! Event-driven X11 shortcut listener with a shutdown pipe, without polling.
use crate::x11_stream::{self, X11Connection};
use global_hotkey::hotkey::{HotKey, Modifiers};
use nanoclick_core::{BackendError, Cancellation};
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
        xproto::{ConnectionExt, GrabMode, Mapping, ModMask},
    },
};

pub struct X11Shortcut {
    shutdown: UnixStream,
    cancel: Cancellation,
    worker: Option<JoinHandle<()>>,
}
fn error(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!("X11 shortcut: {error}"))
}
#[derive(PartialEq, Eq)]
struct KeyboardMapping {
    keysyms_per_keycode: u8,
    keysyms: Vec<u32>,
    modifiers: Vec<u8>,
}
fn keyboard_mapping(conn: &X11Connection) -> Result<KeyboardMapping, BackendError> {
    let first = conn.setup().min_keycode;
    let count = conn.setup().max_keycode - first + 1;
    let symbols = conn
        .get_keyboard_mapping(first, count)
        .map_err(error)?
        .reply()
        .map_err(error)?;
    let modifiers = conn
        .get_modifier_mapping()
        .map_err(error)?
        .reply()
        .map_err(error)?;
    Ok(KeyboardMapping {
        keysyms_per_keycode: symbols.keysyms_per_keycode,
        keysyms: symbols.keysyms,
        modifiers: modifiers.keycodes,
    })
}
fn keycode_for_symbol(
    first: u8,
    mapping: &KeyboardMapping,
    symbol: u32,
) -> Result<(u8, bool), BackendError> {
    let width = usize::from(mapping.keysyms_per_keycode);
    if width == 0 {
        return Err(BackendError::new("The keyboard layout has no key symbols."));
    }
    // Prefer the unshifted level. Higher groups require XKB-aware group
    // selection and must not silently become an unrelated physical binding.
    for level in 0..2 {
        let mut matches = mapping
            .keysyms
            .chunks(width)
            .enumerate()
            .filter(|(_, symbols)| symbols.get(level) == Some(&symbol));
        if let Some((offset, _)) = matches.next() {
            if matches.next().is_some() {
                return Err(BackendError::new(
                    "The requested key has ambiguous keyboard mappings. Choose another shortcut.",
                ));
            }
            let code = u8::try_from(usize::from(first) + offset)
                .map_err(|_| BackendError::new("The keyboard keycode is out of range."))?;
            return Ok((code, level == 1));
        }
    }
    Err(BackendError::new(
        "The requested key is not present in the supported keyboard levels.",
    ))
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
        let cancel = Cancellation::default();
        let listener_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("nanoclick-x11-shortcut".into())
            .spawn(move || {
                let setup = (|| {
                    let (conn, screen) = x11_stream::connect(&listener_cancel).map_err(error)?;
                    let root = conn.setup().roots[screen].root;
                    let symbol = keysym(hotkey)?;
                    xkb::ConnectionExt::xkb_use_extension(&conn, 1, 0)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?;
                    // XKB-aware clients need explicit map/device subscriptions;
                    // core keyboard MappingNotify alone does not cover them.
                    let parts = xkb::MapPart::KEY_TYPES
                        | xkb::MapPart::KEY_SYMS
                        | xkb::MapPart::MODIFIER_MAP
                        | xkb::MapPart::EXPLICIT_COMPONENTS
                        | xkb::MapPart::KEY_ACTIONS
                        | xkb::MapPart::KEY_BEHAVIORS
                        | xkb::MapPart::VIRTUAL_MODS
                        | xkb::MapPart::VIRTUAL_MOD_MAP;
                    xkb::ConnectionExt::xkb_select_events(
                        &conn,
                        xkb::ID::USE_CORE_KBD.into(),
                        Default::default(),
                        xkb::EventType::MAP_NOTIFY | xkb::EventType::NEW_KEYBOARD_NOTIFY,
                        parts,
                        parts,
                        &Default::default(),
                    )
                    .map_err(error)?
                    .check()
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
                    let first = conn.setup().min_keycode;
                    let map = keyboard_mapping(&conn)?;
                    let (code, shifted) = keycode_for_symbol(first, &map, symbol)?;
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
                    if shifted {
                        modifiers |= ModMask::SHIFT;
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
                    Ok((conn, code, map))
                })();
                let (conn, keycode, original_mapping) = match setup {
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
                        conn.stream().begin_request();
                        while let Some(event) = conn.poll_for_event().map_err(error)? {
                            match event {
                                Event::KeyPress(event) if event.detail == keycode && !held => {
                                    held = true;
                                    activated();
                                }
                                Event::KeyRelease(event) if event.detail == keycode => held = false,
                                Event::XkbMapNotify(_) => {
                                    return Err(BackendError::new(
                                        "Keyboard mapping changed. Apply the shortcut again.",
                                    ));
                                }
                                Event::XkbNewKeyboardNotify(event) => {
                                    // XTEST may switch the master device without changing
                                    // its map. Preserve the binding only when both symbol
                                    // and modifier maps and keycode bounds are unchanged.
                                    if event.min_key_code != event.old_min_key_code
                                        || event.max_key_code != event.old_max_key_code
                                        || keyboard_mapping(&conn)? != original_mapping
                                    {
                                        return Err(BackendError::new(
                                            "Keyboard mapping changed. Apply the shortcut again.",
                                        ));
                                    }
                                }
                                Event::MappingNotify(event)
                                    if event.request != Mapping::POINTER =>
                                {
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
                if let Err(error) = result
                    && !listener_cancel.is_cancelled()
                {
                    failed(error.to_string());
                }
            })
            .map_err(error)?;
        match response.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(Self {
                shutdown,
                cancel,
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(error) => {
                cancel.fail(BackendError::new("Shortcut registration cancelled."));
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
        self.cancel
            .fail(BackendError::new("Shortcut listener stopped."));
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

#[cfg(test)]
mod tests {
    use super::*;
    fn mapping(symbols: &[u32]) -> KeyboardMapping {
        KeyboardMapping {
            keysyms_per_keycode: 2,
            keysyms: symbols.to_vec(),
            modifiers: vec![],
        }
    }
    #[test]
    fn azerty_digit_requires_shift() -> Result<(), BackendError> {
        assert_eq!(
            keycode_for_symbol(10, &mapping(&[b'&'.into(), b'1'.into()]), b'1'.into())?,
            (10, true)
        );
        Ok(())
    }
    #[test]
    fn prefers_unshifted_symbol_over_shifted_alias() -> Result<(), BackendError> {
        assert_eq!(
            keycode_for_symbol(
                10,
                &mapping(&[b'&'.into(), b'1'.into(), b'1'.into(), b'!'.into()]),
                b'1'.into()
            )?,
            (11, false)
        );
        Ok(())
    }
    #[test]
    fn rejects_ambiguous_base_level() {
        assert!(keycode_for_symbol(10, &mapping(&[0x31, 0x21, 0x31, 0x21]), 0x31).is_err());
    }
}
