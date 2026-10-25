//! Real window-event capture smoke, on an owned private X11 display only.
#![forbid(unsafe_code)]
#[path = "support/window.rs"]
mod support;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};
use support::{App, Result, close, focus_target, pair, wait};
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{AtomEnum, ConnectionExt, MapState},
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};
fn records(path: &Path) -> Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    if !text.ends_with('\n') {
        return Ok(Vec::new());
    }
    Ok(text
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?)
}
fn press(conn: &RustConnection, detail: u8) -> Result<()> {
    conn.xtest_fake_input(4, detail, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    conn.xtest_fake_input(5, detail, 0, x11rb::NONE, 0, 0, 0)?
        .check()?;
    Ok(())
}
fn validate(
    record: &serde_json::Value,
    id: u64,
    selected: usize,
    count: u64,
    reason: &str,
) -> Result<()> {
    let report = &record["report"];
    if record["record_type"] != "session"
        || record["schema_version"] != 1
        || record["backend"] != "x11"
        || record["finish_reason"] != reason
        || report["session_id"] != id
        || report["balanced"] != true
        || report["unexpected_buttons"] != false
        || report["overflowed"] != false
        || report["overlap_presses"] != 0
    {
        return Err(format!("Capture session metadata mismatch: {record}").into());
    }
    let buttons = report["buttons"]
        .as_array()
        .ok_or("Capture buttons must be an array")?;
    if buttons.len() != 5 {
        return Err("Capture must distinguish all five pointer buttons".into());
    }
    for (index, button) in buttons.iter().enumerate() {
        let expected = if index == selected { count } else { 0 };
        if button["presses"] != expected
            || button["releases"] != expected
            || button["pairs"] != expected
            || button["anomalies"] != 0
            || button["held"] != false
        {
            return Err(format!("Capture counter mismatch: {button}; expected {expected}").into());
        }
    }
    Ok(())
}
fn main() -> Result<()> {
    if std::env::var("NANOCLICK_ISOLATED_X11").as_deref() != Ok("1") {
        return Err(
            "Use the owned-display script; never use the user desktop for injection".into(),
        );
    }
    let mut args = std::env::args_os().skip(1);
    let binary = args.next().ok_or("Pass the input_capture executable")?;
    let path = PathBuf::from(args.next().ok_or("Pass a new JSONL path")?);
    if args.next().is_some() || path.exists() {
        return Err("Pass an executable and an unused JSONL path".into());
    }
    let (conn, screen) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen].root;
    let mut app = App(Command::new(binary).arg(&path).spawn()?);
    let pid_atom = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;
    let mut window = None;
    wait(
        || {
            if app.0.try_wait()?.is_some() {
                return Err("Input capture exited before window discovery".into());
            }
            for candidate in support::candidate_windows(&conn, root)? {
                let name = conn
                    .get_property(false, candidate, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 64)?
                    .reply()?
                    .value;
                let pid = conn
                    .get_property(false, candidate, pid_atom, AtomEnum::CARDINAL, 0, 1)?
                    .reply()?;
                if name == b"NanoClick input capture"
                    && pid.value32().and_then(|mut values| values.next()) == Some(app.0.id())
                    && conn.get_window_attributes(candidate)?.reply()?.map_state
                        == MapState::VIEWABLE
                    && records(&path)?.len() == 1
                {
                    window = Some(candidate);
                    return Ok(true);
                }
            }
            Ok(false)
        },
        Duration::from_secs(15),
    )?;
    let window = window.ok_or("Capture window not found")?;
    let header = records(&path)?.remove(0);
    if header["record_type"] != "metadata"
        || header["schema_version"] != 1
        || header["backend"] != "x11"
        || header["button_order"]
            != serde_json::json!(["left", "right", "middle", "extra1", "extra2"])
    {
        return Err(format!("Capture backend/header was not attested: {header}").into());
    }
    let first = conn.setup().min_keycode;
    let map = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let offset = map
        .keysyms
        .chunks(usize::from(map.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&0x20))
        .ok_or("Space not mapped")?;
    let space = u8::try_from(usize::from(first) + offset)?;
    focus_target(&conn, window)?;
    if let Some(capture) = std::env::var_os("NANOCLICK_CAPTURE_WINDOW_IMAGE") {
        thread::sleep(Duration::from_millis(150));
        if !Command::new("import")
            .args(["-silent", "-window", &window.to_string()])
            .arg(capture)
            .status()?
            .success()
        {
            return Err("Capture window image failed".into());
        }
    }
    // A click before arming must not leak into a later session.
    press(&conn, 1)?;
    for (index, (detail, count)) in [(1, 3u64), (3, 5), (2, 7)].into_iter().enumerate() {
        pair(&conn, space)?;
        for _ in 0..count {
            press(&conn, detail)?;
        }
        pair(&conn, space)?;
        wait(
            || Ok(records(&path)?.len() == index + 2),
            Duration::from_secs(3),
        )?;
        let record = records(&path)?.remove(index + 1);
        validate(&record, (index + 1) as u64, index, count, "paused")?;
        println!(
            "PASS capture window: session {}, native button {detail}, {count} complete pairs",
            index + 1
        );
    }
    pair(&conn, space)?;
    press(&conn, 1)?;
    close(&conn, window, &mut app)?;
    let final_records = records(&path)?;
    if final_records.len() != 5 {
        return Err("Closing did not retain exactly four capture sessions".into());
    }
    validate(&final_records[4], 4, 0, 1, "closed")?;
    println!("PASS capture window: active close exports once, clean exit; backend attested X11");
    Ok(())
}
