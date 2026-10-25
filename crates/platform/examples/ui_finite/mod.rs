//! Finite plans edited through the actual GUI, captured on the private display.
use crate::support::{self, Result, close, launch_with_diagnostics, pair, ready_button, wait};
use std::{ffi::OsStr, fs, path::Path, thread, time::Duration};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt, InputFocus},
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};
fn key(conn: &RustConnection, symbol: u32) -> Result<u8> {
    let first = conn.setup().min_keycode;
    let map = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let offset = map
        .keysyms
        .chunks(usize::from(map.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&symbol))
        .ok_or("Required finite-plan key is not mapped")?;
    Ok(u8::try_from(usize::from(first) + offset)?)
}
fn click(conn: &RustConnection, window: u32, x: f64, y: f64) -> Result<()> {
    let factor = support::scale_factor()?;
    support::move_pointer(
        conn,
        window,
        (x * factor).round() as i16,
        (y * factor).round() as i16,
    )?;
    conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}
fn edit(conn: &RustConnection, window: u32, x: f64, y: f64, digits: &str) -> Result<()> {
    conn.set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
        .check()?;
    click(conn, window, x, y)?;
    let control = key(conn, 0xffe3)?;
    conn.xtest_fake_input(2, control, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    pair(conn, key(conn, u32::from(b'a'))?)?;
    conn.xtest_fake_input(3, control, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    for digit in digits.bytes() {
        pair(conn, key(conn, u32::from(digit))?)?;
    }
    thread::sleep(Duration::from_millis(100));
    Ok(())
}
fn records(path: &Path) -> Result<Vec<serde_json::Value>> {
    let text = fs::read_to_string(path)?;
    // A concurrent terminal flush can be observed before its final newline.
    if !text.ends_with('\n') {
        return Ok(Vec::new());
    }
    Ok(text
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?)
}
fn capture(conn: &RustConnection, events: &mut Vec<(u8, bool)>) -> Result<()> {
    while let Some(event) = conn.poll_for_event()? {
        match event {
            Event::ButtonPress(event) => events.push((event.detail, true)),
            Event::ButtonRelease(event) => events.push((event.detail, false)),
            _ => {}
        }
    }
    Ok(())
}
pub fn run(
    conn: &RustConnection,
    root: u32,
    target: u32,
    toggle: u8,
    binary: &OsStr,
) -> Result<()> {
    let directory =
        std::env::var_os("XDG_CONFIG_HOME").ok_or("Private configuration is required")?;
    for (index, (name, detail)) in [("left", 1), ("right", 3), ("middle", 2)]
        .into_iter()
        .enumerate()
    {
        let path = Path::new(&directory).join(format!("finite-{name}.jsonl"));
        let (mut app, window, _) = launch_with_diagnostics(conn, root, binary, Some(&path))?;
        edit(conn, window, 145.0, 134.0, "25")?;
        if index != 0 {
            click(conn, window, 180.0, 170.0)?;
            crate::capture(window, "NANOCLICK_BUTTON_MENU_CAPTURE")?;
            click(conn, window, 180.0, 206.0 + index as f64 * 36.0)?;
        }
        click(conn, window, 40.0, 283.0)?;
        for (run, count) in [1usize, 7].into_iter().enumerate() {
            edit(conn, window, 145.0, 283.0, &count.to_string())?;
            if !ready_button(conn, window) {
                return Err("Finite plan was not ready after editing".into());
            }
            support::focus_target(conn, target)?;
            let mut events = Vec::new();
            capture(conn, &mut events)?;
            if !events.is_empty() {
                return Err("Input occurred while editing a finite plan".into());
            }
            pair(conn, toggle)?;
            wait(
                || {
                    capture(conn, &mut events)?;
                    Ok(events.len() >= count * 2
                        && ready_button(conn, window)
                        && records(&path)?.len() == run + 1)
                },
                Duration::from_secs(4),
            )
            .map_err(|error| {
                format!("Finite {name} N={count} did not complete: {error}; events={events:?}")
            })?;
            thread::sleep(Duration::from_millis(250));
            capture(conn, &mut events)?;
            if events.len() != count * 2
                || events
                    .chunks(2)
                    .any(|pair| pair != [(detail, true), (detail, false)])
            {
                return Err(format!("Finite {name} N={count} capture mismatch: {events:?}").into());
            }
            let all = records(&path)?;
            let terminal = &all[run];
            if terminal["run_id"] != (run + 1) as u64
                || terminal["reason"] != "completed"
                || terminal["attempts"] != count as u64
                || terminal["sent"] != count as u64
                || terminal["failures"] != 0
            {
                return Err(format!("Finite GUI terminal mismatch: {terminal}").into());
            }
            println!("finite_terminal,{name},{terminal}");
            println!(
                "PASS GUI finite: {name}, N={count}, run_id={}, exact complete pairs, Completed, controls editable, no late input",
                run + 1
            );
        }
        close(conn, window, &mut app)?;
    }
    Ok(())
}
