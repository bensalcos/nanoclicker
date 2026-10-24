//! End-to-end GUI acceptance on a private X11 test display only.
use std::{
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{
            ConfigureWindowAux, ConnectionExt, CreateWindowAux, EventMask, InputFocus, WindowClass,
        },
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};
mod support;
mod ui_close;
mod ui_preferences;
mod ui_stress;
use support::{Result, action_position, close, launch, pair, ready_button, scale_factor, wait};
fn active_button(conn: &RustConnection, window: u32) -> bool {
    matches!(
        support::button_color(conn, window),
        Some(support::ButtonColor::Red)
    )
}
fn drain(conn: &RustConnection, events: &mut Vec<bool>) -> Result<()> {
    while let Some(event) = conn.poll_for_event()? {
        match event {
            Event::ButtonPress(event) => {
                if event.detail != 1 {
                    return Err("GUI capture received the wrong button".into());
                }
                events.push(true);
            }
            Event::ButtonRelease(event) => {
                if event.detail != 1 {
                    return Err("GUI capture received the wrong button".into());
                }
                events.push(false);
            }
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
    let stress_cycles = std::env::var("NANOCLICK_UI_STRESS_CYCLES")
        .ok()
        .map(|value| value.parse::<u32>())
        .transpose()?;
    if stress_cycles.is_some_and(|cycles| !(1..=1000).contains(&cycles)) {
        return Err("UI stress cycles must be 1..1000.".into());
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
        1400,
        100,
        250,
        250,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new()
            .override_redirect(1u32)
            .background_pixel(0x202020)
            .event_mask(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE),
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
    support::focus_target(&conn, target)?;
    pair(&conn, key)?;
    let mut events = Vec::new();
    wait(
        || {
            drain(&conn, &mut events)?;
            Ok(events.len() >= 4)
        },
        Duration::from_secs(3),
    )
    .map_err(|error| {
        format!(
            "Outside-focus Start failed: {error}; received {} target events, ready={}, active={}",
            events.len(),
            ready_button(&conn, window),
            active_button(&conn, window)
        )
    })?;
    capture(window, "NANOCLICK_RUNNING_CAPTURE")?;
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

    if let Some(cycles) = stress_cycles {
        ui_stress::run(&conn, window, key, app.0.id(), &binary, cycles)?;
    }

    // The main action must remain usable at the documented minimum size,
    // while secondary content may scroll. No input is active during resize.
    let factor = scale_factor()?;
    let width = (420.0 * factor).round() as u32;
    let height = (520.0 * factor).round() as u32;
    conn.configure_window(
        window,
        &ConfigureWindowAux::new().width(width).height(height),
    )?
    .check()?;
    wait(
        || {
            let size = conn.get_geometry(window)?.reply()?;
            Ok(u32::from(size.width) == width
                && u32::from(size.height) == height
                && ready_button(&conn, window))
        },
        Duration::from_secs(3),
    )?;
    capture(window, "NANOCLICK_LAYOUT_CAPTURE")?;
    if std::env::var_os("NANOCLICK_SCROLL_CAPTURE").is_some() {
        conn.warp_pointer(
            x11rb::NONE,
            window,
            0,
            0,
            0,
            0,
            (200.0 * factor).round() as i16,
            (300.0 * factor).round() as i16,
        )?
        .check()?;
        for _ in 0..5 {
            conn.xtest_fake_input(4, 5, 0, x11rb::NONE, 0, 0, 0)?
                .check()?;
            conn.xtest_fake_input(5, 5, 0, x11rb::NONE, 0, 0, 0)?
                .check()?;
        }
        capture(window, "NANOCLICK_SCROLL_CAPTURE")?;
        if !ready_button(&conn, window) {
            return Err("Scrolling displaced Start".into());
        }
    }
    println!("PASS GUI: Start remains visible at 420x520 logical units, scale {factor}");

    // Exercise the actual Start button, then move to the target during its
    // cancelable three-second preparation instead of bypassing the controller.
    events.clear();
    conn.set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
        .check()?;
    let (action_x, action_y) = action_position(&conn, window)?;
    support::move_pointer(&conn, window, action_x, action_y)?;
    conn.xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    let started = Instant::now();
    let gpu_fixture = std::env::var_os("NANOCLICK_GPU_SESSION").is_some();
    let mut cursor_reassertions = 0;
    wait(|| Ok(active_button(&conn, window)), Duration::from_secs(3)).map_err(|error| {
        let _ = capture(window, "NANOCLICK_FAILURE_CAPTURE");
        format!("Manual Start was not acknowledged: {error}")
    })?;
    support::focus_target(&conn, target)?;
    capture(window, "NANOCLICK_PREPARING_CAPTURE")?;
    wait(
        || {
            drain(&conn, &mut events)?;
            // A headless compositor's fake seat can reset emulated coordinates
            // on focus changes. Keep only that fixture's cursor on its target;
            // the normal Xvfb/manual-pointer scenario remains unchanged.
            if gpu_fixture {
                let pointer = conn.query_pointer(target)?.reply()?;
                if pointer.win_x != 100 || pointer.win_y != 100 {
                    support::move_pointer(&conn, target, 100, 100)?;
                    cursor_reassertions += 1;
                }
            }
            Ok(events.len() >= 2)
        },
        Duration::from_secs(6),
    )
    .map_err(|error| {
        let _ = capture(window, "NANOCLICK_FAILURE_CAPTURE");
        let pointer = conn.query_pointer(target).ok().and_then(|cookie| cookie.reply().ok()).map(|reply| (reply.root_x, reply.root_y, reply.win_x, reply.win_y, u16::from(reply.mask), reply.child));
        let target_geometry = conn.get_geometry(target).ok().and_then(|cookie| cookie.reply().ok()).map(|reply| (reply.x, reply.y, reply.width, reply.height));
        let root_geometry = conn.get_geometry(root).ok().and_then(|cookie| cookie.reply().ok()).map(|reply| (reply.width, reply.height));
        format!("Manual Start did not reach its target: {error}; captured {} events; pointer={pointer:?}; target={target_geometry:?}; root={root_geometry:?}", events.len())
    })?;
    if gpu_fixture {
        println!("gpu_fixture_cursor_reassertions,{cursor_reassertions}");
    }
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
    ui_close::run(&conn, root, target, key, &binary)?;
    ui_preferences::run(&conn, root, target, key, &binary)?;
    Ok(())
}

fn capture(window: u32, variable: &str) -> Result<()> {
    if let Some(path) = std::env::var_os(variable) {
        // Optional test-only capture; production has no screenshot flag.
        thread::sleep(Duration::from_millis(100));
        let status = std::process::Command::new("import")
            .args(["-silent", "-window", &window.to_string()])
            .arg(path)
            .status()?;
        if !status.success() {
            return Err("Minimum-size screenshot failed".into());
        }
    }
    Ok(())
}
