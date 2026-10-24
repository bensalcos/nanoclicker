//! Exercise orderly window close while input is preparing or active.
use crate::{
    active_button, assert_pairs, drain,
    support::{Result, action_position, close, launch, pair, wait},
};
use std::{ffi::OsStr, thread, time::Duration};
use x11rb::{
    protocol::{
        xproto::{ConnectionExt, InputFocus},
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};

pub fn run(
    connection: &RustConnection,
    root: u32,
    target: u32,
    key: u8,
    binary: &OsStr,
) -> Result<()> {
    for scenario in ["preparing", "running", "stop-request-race"] {
        let (mut app, window, _) = launch(connection, root, binary)?;
        let mut events = Vec::new();
        if scenario == "preparing" {
            connection
                .set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
                .check()?;
            let (x, y) = action_position(connection, window)?;
            connection
                .warp_pointer(x11rb::NONE, window, 0, 0, 0, 0, x, y)?
                .check()?;
            connection
                .xtest_fake_input(4, 1, 0, x11rb::NONE, 0, 0, 0)?
                .check()?;
            connection
                .xtest_fake_input(5, 1, 0, x11rb::NONE, 0, 0, 0)?
                .check()?;
        }
        if scenario == "preparing" {
            wait(
                || Ok(active_button(connection, window)),
                Duration::from_secs(3),
            )?;
        }
        crate::support::focus_target(connection, target)?;
        if scenario != "preparing" {
            pair(connection, key)?;
        }
        wait(
            || {
                if app.0.try_wait()?.is_some() {
                    return Err("Application exited before active-close test".into());
                }
                drain(connection, &mut events)?;
                Ok(active_button(connection, window)
                    && (scenario == "preparing" || events.len() >= 2))
            },
            Duration::from_secs(3),
        )?;
        if scenario == "stop-request-race" {
            pair(connection, key)?;
        }
        close(connection, window, &mut app)?;
        drain(connection, &mut events)?;
        if scenario == "preparing" {
            if !events.is_empty() {
                return Err("Close during preparation sent input before cancellation".into());
            }
        } else {
            assert_pairs(&events)?;
        }
        let count = events.len();
        thread::sleep(Duration::from_millis(250));
        drain(connection, &mut events)?;
        if events.len() != count {
            return Err("Input continued after application exit".into());
        }
        println!("PASS GUI close: {scenario}; clean exit, no held pair and no post-exit input");
    }
    Ok(())
}
