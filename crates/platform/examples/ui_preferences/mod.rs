//! Actual Save/relaunch flow; all files and input stay inside the private fixture.
use crate::support::{self, Result, close, launch, pair, ready_button, wait};
use std::{ffi::OsStr, fs, path::PathBuf, thread, time::Duration};
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
    let mapping = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let offset = mapping
        .keysyms
        .chunks(usize::from(mapping.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&symbol))
        .ok_or("Required key is not mapped")?;
    Ok(u8::try_from(usize::from(first) + offset)?)
}
fn save(conn: &RustConnection, window: u32) -> Result<()> {
    conn.set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
        .check()?;
    let factor = support::scale_factor()?;
    let width = conn.get_geometry(window)?.reply()?.width;
    support::move_pointer(
        conn,
        window,
        (f64::from(width) - 45.0 * factor).round() as i16,
        (40.0 * factor).round() as i16,
    )?;
    conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    Ok(())
}
fn edit_interval(conn: &RustConnection, window: u32, digits: &str) -> Result<()> {
    // The Save status inserts a line above Timing. Wait for that frame before
    // targeting the interval, then let its click establish text focus.
    thread::sleep(Duration::from_millis(150));
    let factor = support::scale_factor()?;
    support::move_pointer(
        conn,
        window,
        (145.0 * factor).round() as i16,
        (157.0 * factor).round() as i16,
    )?;
    conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    thread::sleep(Duration::from_millis(100));
    let control = key(conn, 0xffe3)?;
    conn.xtest_fake_input(2, control, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    pair(conn, key(conn, u32::from(b'a'))?)?;
    conn.xtest_fake_input(3, control, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    for digit in digits.bytes() {
        pair(conn, key(conn, u32::from(digit))?)?;
    }
    // Give the actual input/render loop a bounded chance to process the edit.
    thread::sleep(Duration::from_millis(100));
    if !ready_button(conn, window) {
        return Err(
            "Editing preferences unexpectedly started input or invalidated the plan".into(),
        );
    }
    Ok(())
}
fn path() -> Result<PathBuf> {
    let base = PathBuf::from(
        std::env::var_os("XDG_CONFIG_HOME").ok_or("Private configuration is required")?,
    );
    if !base.is_absolute() {
        return Err("Private configuration must be absolute".into());
    }
    Ok(base.join("nanoclick/preferences.json"))
}
fn read(path: &std::path::Path) -> Result<serde_json::Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn observed_interval(
    conn: &RustConnection,
    target: u32,
    window: u32,
    toggle: u8,
    expected: std::ops::RangeInclusive<u32>,
) -> Result<u32> {
    support::focus_target(conn, target)?;
    let mut times = Vec::new();
    let mut events = Vec::new();
    thread::sleep(Duration::from_millis(500));
    while let Some(event) = conn.poll_for_event()? {
        if matches!(event, Event::ButtonPress(_) | Event::ButtonRelease(_)) {
            return Err("Relaunch resumed input without an explicit Start".into());
        }
    }
    pair(conn, toggle)?;
    wait(
        || {
            while let Some(event) = conn.poll_for_event()? {
                match event {
                    Event::ButtonPress(event) => {
                        times.push(event.time);
                        events.push(true);
                    }
                    Event::ButtonRelease(_) => events.push(false),
                    _ => {}
                }
            }
            Ok(times.len() >= 12)
        },
        Duration::from_secs(3),
    )?;
    pair(conn, toggle)?;
    wait(|| Ok(ready_button(conn, window)), Duration::from_secs(3))?;
    while let Some(event) = conn.poll_for_event()? {
        match event {
            Event::ButtonPress(_) => events.push(true),
            Event::ButtonRelease(_) => events.push(false),
            _ => {}
        }
    }
    crate::assert_pairs(&events)?;
    let mut gaps: Vec<_> = times
        .windows(2)
        .map(|pair| pair[1].wrapping_sub(pair[0]))
        .collect();
    gaps.sort_unstable();
    let median = gaps[gaps.len() / 2];
    if !expected.contains(&median) {
        return Err(format!("GUI interval did not match {expected:?}: median {median} ms").into());
    }
    Ok(median)
}

pub fn run(
    conn: &RustConnection,
    root: u32,
    target: u32,
    toggle: u8,
    binary: &OsStr,
) -> Result<()> {
    let path = path()?;
    if path.exists() {
        return Err("Save fixture requires an unused private preferences path".into());
    }
    let (mut app, window, _) = launch(conn, root, binary)?;
    save(conn, window)?;
    wait(|| Ok(path.is_file()), Duration::from_secs(3))
        .map_err(|error| format!("Initial Save failed: {error}"))?;
    crate::capture(window, "NANOCLICK_PREFERENCES_CAPTURE")?;
    edit_interval(conn, window, "37")?;
    save(conn, window)?;
    wait(
        || Ok(path.is_file() && read(&path)?["interval_ms"] == 37),
        Duration::from_secs(3),
    )
    .map_err(|error| {
        format!(
            "Edited interval was not saved: {error}; file={:?}",
            read(&path)
        )
    })?;
    let saved = fs::read(&path)?;
    let data = read(&path)?;
    if data["version"] != 1
        || data["button"] != "left"
        || !data["count"].is_null()
        || data["shortcut"] != "F6"
        || data["theme"] != "dark"
        || data
            .as_object()
            .ok_or("Preferences must be an object")?
            .len()
            != 6
    {
        return Err(format!("Unexpected saved preferences: {data}").into());
    }
    edit_interval(conn, window, "91")?;
    observed_interval(conn, target, window, toggle, 70..=115)?;
    close(conn, window, &mut app)?;
    if fs::read(&path)? != saved {
        return Err("Closing auto-saved an unsaved edit".into());
    }
    let (mut app, window, _) = launch(conn, root, binary)?;
    let median = observed_interval(conn, target, window, toggle, 25..=65)?;
    // Remove only this fixture's known file so Save must actually publish the
    // restored draft; an already matching file cannot satisfy the predicate.
    fs::remove_file(&path)?;
    save(conn, window)?;
    wait(
        || Ok(path.is_file() && read(&path)?["interval_ms"] == 37),
        Duration::from_secs(3),
    )?;
    close(conn, window, &mut app)?;
    if fs::read(&path)? != saved {
        return Err("Relaunch changed saved preferences".into());
    }
    println!(
        "PASS GUI preferences: explicit Save, unsaved edit discarded, saved interval restored, no automatic Start; captured median interval {median} ms"
    );
    Ok(())
}
