//! Wire-level lifecycle checks against a private mock D-Bus portal.
#![forbid(unsafe_code)]
mod portal_mock;
use futures_lite::future::block_on;
use nanoclick_core::{ClickConfig, Engine, MouseButton, RunState, Snapshot, StopReason};
use nanoclick_platform::wayland::WaylandInput;
use portal_mock::{Mode, NAME, lock};
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn await_state(engine: &Engine, ready: impl Fn(&Snapshot) -> bool) -> Result<Snapshot> {
    let timeout = Instant::now() + Duration::from_secs(4);
    loop {
        let snapshot = engine.snapshot();
        if ready(&snapshot) {
            return Ok(snapshot);
        }
        if Instant::now() > timeout {
            return Err(format!("Portal lifecycle timed out: {snapshot:?}").into());
        }
        thread::sleep(Duration::from_millis(1));
    }
}
fn main() -> Result<()> {
    if std::env::var("NANOCLICK_ISOLATED_DBUS").as_deref() != Ok("1") {
        return Err("Use scripts/verify-portal.sh; the private-bus guard is required.".into());
    }
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return Err("The mock must run without a compositor connection.".into());
    }
    block_on(async {
        let connection = zbus::Connection::session().await?;
        let bus = zbus::fdo::DBusProxy::new(&connection).await?;
        if bus
            .list_activatable_names()
            .await?
            .iter()
            .any(|name| name.as_str() != "org.freedesktop.DBus")
        {
            return Err("The test bus must have no activatable host services.".into());
        }
        Ok::<(), Box<dyn Error>>(())
    })?;
    for scenario in [
        "normal-stop",
        "denied",
        "cancel-preparation",
        "release-failure",
        "closed-during-select",
        "revoked",
        "owner-lost",
        "owner-replaced",
    ] {
        let mode = match scenario {
            "denied" => Mode::Deny,
            "closed-during-select" => Mode::CloseDuringSelect,
            "cancel-preparation" => Mode::HoldStart,
            "release-failure" => Mode::FailRelease,
            _ => Mode::Allow,
        };
        let (service, state) = block_on(portal_mock::serve(mode))?;
        let engine = Engine::new(WaylandInput::open, || {})?;
        engine.start(ClickConfig::new(999_999, MouseButton::Left, None)?)?;
        let mut replacement = None;
        if matches!(mode, Mode::HoldStart) {
            await_state(&engine, |snapshot| {
                snapshot.state == RunState::Preparing && lock(&state).starts == 1
            })?;
            engine.stop();
        }
        if matches!(mode, Mode::Allow) {
            let running = await_state(&engine, |snapshot| {
                snapshot.metrics.sent == 1 || snapshot.state == RunState::Faulted
            })?;
            if running.state == RunState::Faulted {
                return Err(format!("Preparation failed: {:?}", running.error).into());
            }
            match scenario {
                "normal-stop" => engine.stop(),
                "revoked" => {
                    let session = lock(&state).session.clone().ok_or("Missing mock session")?;
                    block_on(portal_mock::closed(&service, &session))?;
                }
                "owner-lost" => {
                    block_on(service.release_name(NAME))?;
                }
                "owner-replaced" => {
                    let connection = block_on(zbus::Connection::session())?;
                    block_on(connection.request_name_with_flags(
                        NAME,
                        zbus::fdo::RequestNameFlags::ReplaceExisting
                            | zbus::fdo::RequestNameFlags::DoNotQueue,
                    ))?;
                    replacement = Some(connection);
                }
                _ => return Err("Unknown lifecycle scenario".into()),
            }
        }
        let expected = if matches!(scenario, "normal-stop" | "cancel-preparation") {
            RunState::Idle
        } else {
            RunState::Faulted
        };
        let terminal = await_state(&engine, |snapshot| snapshot.state == expected)?;
        let observed = lock(&state);
        if expected == RunState::Idle {
            if terminal.reason != Some(StopReason::Stopped) || observed.closes != 1 {
                return Err("Normal Stop did not close the session exactly once".into());
            }
        } else if terminal.reason != Some(StopReason::Failed) || terminal.error.is_none() {
            return Err("Portal failure was not retained".into());
        }
        if scenario == "revoked" && observed.closes != 1 {
            return Err("Revoked session cleanup was not attempted exactly once".into());
        }
        if matches!(scenario, "owner-lost" | "owner-replaced")
            && !terminal
                .error
                .as_deref()
                .is_some_and(|message| message.contains("Session:"))
        {
            return Err("Session cleanup failure after owner loss was hidden".into());
        }
        if matches!(mode, Mode::Allow) {
            if observed.buttons != [(0x110, 1), (0x110, 0)] {
                return Err("Mock did not receive one complete pair".into());
            }
        } else if matches!(mode, Mode::FailRelease) {
            if observed.buttons != [(0x110, 1), (0x110, 0), (0x110, 0)]
                || observed.closes != 1
                || terminal.metrics.sent != 0
                || terminal.metrics.failures != 1
            {
                return Err(
                    "Failed release was counted as sent, not retried, or leaked its session".into(),
                );
            }
        } else if !observed.buttons.is_empty() || observed.closes != 1 {
            return Err("Denied/revoked preparation injected input or leaked its session".into());
        }
        if scenario == "denied" && observed.starts != 0 {
            return Err("Denied selection continued to Start".into());
        }
        drop(observed);
        drop(engine);
        if let Some(connection) = replacement {
            block_on(connection.release_name(NAME))?;
        }
        let _ = block_on(service.release_name(NAME))?;
        println!("PASS portal lifecycle: {scenario}");
    }
    Ok(())
}
