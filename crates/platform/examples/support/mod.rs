mod window;
use std::{
    ffi::OsStr,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
pub use window::*;
use x11rb::{
    protocol::xproto::{AtomEnum, ConnectionExt, ImageFormat, MapState},
    rust_connection::RustConnection,
};
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
pub fn launch(conn: &RustConnection, root: u32, binary: &OsStr) -> Result<(App, u32, Duration)> {
    launch_with_diagnostics(conn, root, binary, None)
}
pub fn launch_with_diagnostics(
    conn: &RustConnection,
    root: u32,
    binary: &OsStr,
    diagnostics: Option<&Path>,
) -> Result<(App, u32, Duration)> {
    let began = Instant::now();
    let mut command = Command::new(binary);
    command.args(["--theme", "dark"]);
    if let Some(path) = diagnostics {
        command.arg("--diagnostics").arg(path);
    }
    let mut app = App(command.spawn()?);
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
