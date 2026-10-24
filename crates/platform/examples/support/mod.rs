use std::{
    error::Error,
    ffi::OsStr,
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    protocol::{
        xproto::{
            AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask,
            ImageFormat, InputFocus, MapState, StackMode,
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
pub fn scale_factor() -> Result<f64> {
    let factor = std::env::var("WINIT_X11_SCALE_FACTOR")
        .unwrap_or_else(|_| "1".into())
        .parse::<f64>()?;
    if ![1.0, 1.5, 2.0].contains(&factor) {
        return Err("Harness scale must be 1, 1.5 or 2".into());
    }
    Ok(factor)
}
pub fn action_position(conn: &RustConnection, window: u32) -> Result<(i16, i16)> {
    let geometry = conn.get_geometry(window)?.reply()?;
    Ok((
        i16::try_from(geometry.width / 2)?,
        i16::try_from(
            geometry
                .height
                .saturating_sub((50.0 * scale_factor()?).round() as u16),
        )?,
    ))
}
pub enum ButtonColor {
    Blue,
    Red,
    Other,
}
pub fn button_color(conn: &RustConnection, window: u32) -> Option<ButtonColor> {
    let (x, y) = action_position(conn, window).ok()?;
    let image = conn
        .get_image(ImageFormat::Z_PIXMAP, window, x, y, 1, 1, u32::MAX)
        .ok()?
        .reply()
        .ok()?;
    let pixel = image.data.get(..3)?;
    // Xvfb's x86_64 ZPixmap is BGRX. Ready is blue; active states are red.
    Some(
        if pixel[0] > 140 && u16::from(pixel[0]) > u16::from(pixel[2]) * 2 {
            ButtonColor::Blue
        } else if pixel[2] > 140 && u16::from(pixel[2]) > u16::from(pixel[0]) * 2 {
            ButtonColor::Red
        } else {
            ButtonColor::Other
        },
    )
}
pub fn ready_button(conn: &RustConnection, window: u32) -> bool {
    matches!(button_color(conn, window), Some(ButtonColor::Blue))
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
fn candidate_windows(conn: &RustConnection, root: u32) -> Result<Vec<u32>> {
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
pub fn launch(conn: &RustConnection, root: u32, binary: &OsStr) -> Result<(App, u32, Duration)> {
    let began = Instant::now();
    let mut app = App(Command::new(binary).args(["--theme", "dark"]).spawn()?);
    let mut window = None;
    wait(
        || {
            if let Some(status) = app.0.try_wait()? {
                return Err(format!("NanoClick exited early: {status}").into());
            }
            for candidate in candidate_windows(conn, root)? {
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
    )
    .map_err(|error| format!("Application window discovery/readiness failed: {error}"))?;
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
    wait(
        || match app.0.try_wait()? {
            None => Ok(false),
            Some(status) if status.success() => Ok(true),
            Some(status) => Err(format!("NanoClick failed during shutdown: {status}").into()),
        },
        Duration::from_secs(3),
    )
}
