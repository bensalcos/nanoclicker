//! Generic private-window helpers shared by the product and passive-target checks.
use std::{
    error::Error,
    process::Child,
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{
            ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, InputFocus, StackMode,
        },
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub struct App(pub Child);
impl Drop for App {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub fn wait(mut ready: impl FnMut() -> Result<bool>, duration: Duration) -> Result<()> {
    let deadline = Instant::now() + duration;
    loop {
        if ready()? {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err("GUI acceptance timed out.".into());
        }
        thread::sleep(Duration::from_millis(5));
    }
}
pub fn pair(conn: &RustConnection, key: u8) -> Result<()> {
    conn.xtest_fake_input(2, key, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(3, key, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    Ok(())
}

pub fn move_pointer(conn: &RustConnection, window: u32, x: i16, y: i16) -> Result<()> {
    let root = conn.get_geometry(window)?.reply()?.root;
    let point = conn.translate_coordinates(window, root, x, y)?.reply()?;
    // Send motion through the same synthetic device as button events, so XI2
    // clients receive the updated position before a following click.
    conn.xtest_fake_input(6, 0, x11rb::CURRENT_TIME, root, point.dst_x, point.dst_y, 0)?
        .check()?;
    Ok(())
}
pub fn focus_target(conn: &RustConnection, target: u32) -> Result<()> {
    conn.configure_window(
        target,
        &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
    )?
    .check()?;
    move_pointer(conn, target, 100, 100)?;
    conn.set_input_focus(InputFocus::PARENT, target, x11rb::CURRENT_TIME)?
        .check()?;
    Ok(())
}
pub fn candidate_windows(conn: &RustConnection, root: u32) -> Result<Vec<u32>> {
    let mut pending = vec![(root, 0)];
    let mut windows = Vec::new();
    while let Some((window, depth)) = pending.pop() {
        if windows.len() >= 256 {
            return Err("Window discovery exceeded its bounded test tree".into());
        }
        windows.push(window);
        if depth < 4 {
            let reply = match conn.query_tree(window)?.reply() {
                Ok(reply) => reply,
                Err(x11rb::errors::ReplyError::X11Error(error))
                    if error.error_kind == x11rb::protocol::ErrorKind::Window =>
                {
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            pending.extend(reply.children.into_iter().map(|child| (child, depth + 1)));
        }
    }
    Ok(windows)
}
pub fn close(conn: &RustConnection, window: u32, app: &mut App) -> Result<()> {
    let protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;
    let delete = conn.intern_atom(false, b"WM_DELETE_WINDOW")?.reply()?.atom;
    conn.send_event(
        false,
        window,
        EventMask::NO_EVENT,
        ClientMessageEvent::new(
            32,
            window,
            protocols,
            [delete, x11rb::CURRENT_TIME, 0, 0, 0],
        ),
    )?
    .check()?;
    let first = conn.setup().min_keycode;
    let mapping = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let key = |symbol| -> Result<u8> {
        let offset = mapping
            .keysyms
            .chunks(usize::from(mapping.keysyms_per_keycode))
            .position(|symbols| symbols.contains(&symbol))
            .ok_or("Close fixture key is not mapped")?;
        Ok(u8::try_from(usize::from(first) + offset)?)
    };
    let (alt, discard) = (key(0xffe9)?, key(u32::from(b'd'))?);
    let mut last_attempt = Instant::now();
    wait(
        || match app.0.try_wait()? {
            None => {
                // This isolated fixture explicitly chooses Discard in the
                // unsaved-changes modal; it never changes product preferences.
                if last_attempt.elapsed() >= Duration::from_millis(100) {
                    last_attempt = Instant::now();
                    if conn
                        .set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
                        .check()
                        .is_ok()
                    {
                        conn.xtest_fake_input(2, alt, 0, x11rb::NONE, 0, 0, 0)?
                            .check()?;
                        pair(conn, discard)?;
                        conn.xtest_fake_input(3, alt, 0, x11rb::NONE, 0, 0, 0)?
                            .check()?;
                    }
                }
                Ok(false)
            }
            Some(status) if status.success() => Ok(true),
            Some(status) => Err(format!("NanoClick failed during shutdown: {status}").into()),
        },
        Duration::from_secs(3),
    )
}
