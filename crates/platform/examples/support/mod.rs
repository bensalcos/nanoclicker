use std::{
    error::Error,
    ffi::OsStr,
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    protocol::{
        xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, ImageFormat, MapState},
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
pub fn ready_button(conn: &RustConnection, window: u32) -> bool {
    let Ok(cookie) = conn.get_image(ImageFormat::Z_PIXMAP, window, 260, 550, 1, 1, u32::MAX) else {
        return false;
    };
    let Ok(image) = cookie.reply() else {
        return false;
    };
    // Xvfb's x86_64 ZPixmap is BGRX. The idle primary action is blue,
    // whereas Running/Preparing uses red and unavailable input is disabled.
    image.data.len() >= 3
        && image.data[0] > 140
        && u16::from(image.data[0]) > u16::from(image.data[2]) * 2
}
pub fn pair(conn: &RustConnection, key: u8) -> Result<()> {
    conn.xtest_fake_input(2, key, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(3, key, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    Ok(())
}

pub fn launch(conn: &RustConnection, root: u32, binary: &OsStr) -> Result<(App, u32, Duration)> {
    let began = Instant::now();
    let mut app = App(Command::new(binary).args(["--theme", "dark"]).spawn()?);
    let mut window = None;
    wait(
        || {
            if let Some(status) = app.0.try_wait()? {
                return Err(format!("NanoClick exited early: {status}").into());
            }
            for candidate in conn.query_tree(root)?.reply()?.children {
                let name = conn
                    .get_property(false, candidate, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 64)?
                    .reply()?
                    .value;
                if name == b"NanoClick"
                    && conn.get_window_attributes(candidate)?.reply()?.map_state
                        == MapState::VIEWABLE
                    && ready_button(conn, candidate)
                {
                    window = Some(candidate);
                    return Ok(true);
                }
            }
            Ok(false)
        },
        Duration::from_secs(15),
    )?;
    Ok((
        app,
        window.ok_or("NanoClick window was not found.")?,
        began.elapsed(),
    ))
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
    wait(|| Ok(app.0.try_wait()?.is_some()), Duration::from_secs(3))
}
