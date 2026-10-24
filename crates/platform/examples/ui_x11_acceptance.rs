//! End-to-end GUI acceptance on an isolated Xvfb display only.
use std::{
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt, CreateWindowAux, EventMask, InputFocus, WindowClass},
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};
mod support;
use support::{Result, close, launch, pair, ready_button, wait};
fn drain(conn: &RustConnection, events: &mut Vec<bool>) -> Result<()> {
    while let Some(event) = conn.poll_for_event()? {
        match event {
            Event::ButtonPress(event) if event.detail == 1 => events.push(true),
            Event::ButtonRelease(event) if event.detail == 1 => events.push(false),
            _ => {}
        }
    }
    Ok(())
}
fn assert_pairs(events: &[bool]) -> Result<()> {
    if events.is_empty()
        || !events.len().is_multiple_of(2)
        || events.chunks(2).any(|pair| pair != [true, false])
    {
        return Err(format!("Incomplete or unordered GUI-generated input: {events:?}").into());
    }
    Ok(())
}
fn main() -> Result<()> {
    if std::env::var("NANOCLICK_ISOLATED_X11").as_deref() != Ok("1") {
        return Err("Use scripts/verify-ui-x11.sh; requires an isolated display.".into());
    }
    let binary = std::env::args_os()
        .nth(1)
        .ok_or("Pass the NanoClick binary path.")?;
    let (conn, screen) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen].root;
    let target = conn.generate_id()?;
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        target,
        root,
        700,
        100,
        250,
        250,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new().event_mask(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE),
    )?
    .check()?;
    conn.map_window(target)?.check()?;
    let (mut app, window, _) = launch(&conn, root, &binary)?;
    let first = conn.setup().min_keycode;
    let map = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let offset = map
        .keysyms
        .chunks(usize::from(map.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&0xffc3))
        .ok_or("F6 not mapped.")?;
    let key = u8::try_from(usize::from(first) + offset)?;
    conn.warp_pointer(x11rb::NONE, target, 0, 0, 0, 0, 100, 100)?
        .check()?;
    conn.set_input_focus(InputFocus::PARENT, target, x11rb::CURRENT_TIME)?
        .check()?;
    pair(&conn, key)?;
    let mut events = Vec::new();
    wait(
        || {
            drain(&conn, &mut events)?;
            Ok(events.len() >= 4)
        },
        Duration::from_secs(3),
    )?;
    pair(&conn, key)?;
    wait(|| Ok(ready_button(&conn, window)), Duration::from_secs(3))?;
    drain(&conn, &mut events)?;
    assert_pairs(&events)?;
    let stopped_count = events.len();
    thread::sleep(Duration::from_millis(250));
    drain(&conn, &mut events)?;
    if events.len() != stopped_count {
        return Err("The GUI continued clicking after Stop acknowledgement.".into());
    }
    println!(
        "PASS GUI: global shortcut starts/stops outside focus; complete input pairs and no post-stop input"
    );

    // Exercise the actual Start button, then move to the target during its
    // cancelable three-second preparation instead of bypassing the controller.
    events.clear();
    conn.set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
        .check()?;
    conn.warp_pointer(x11rb::NONE, window, 0, 0, 0, 0, 260, 550)?
        .check()?;
    conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    let started = Instant::now();
    conn.warp_pointer(x11rb::NONE, target, 0, 0, 0, 0, 100, 100)?
        .check()?;
    conn.set_input_focus(InputFocus::PARENT, target, x11rb::CURRENT_TIME)?
        .check()?;
    wait(
        || {
            drain(&conn, &mut events)?;
            Ok(events.len() >= 2)
        },
        Duration::from_secs(6),
    )?;
    if started.elapsed() < Duration::from_millis(2800) {
        return Err("Manual Start bypassed its preparation countdown.".into());
    }
    pair(&conn, key)?;
    wait(|| Ok(ready_button(&conn, window)), Duration::from_secs(3))?;
    drain(&conn, &mut events)?;
    assert_pairs(&events)?;
    println!(
        "PASS GUI: manual Start delays input for preparation and then sends pairs to the target"
    );
    close(&conn, window, &mut app)?;
    println!("PASS GUI: clean window shutdown");
    Ok(())
}
