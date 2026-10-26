//! Wire-level lifecycle checks against a private mock D-Bus portal.
#![forbid(unsafe_code)]
mod portal_mock;
use futures_lite::future::block_on;
use nanoclick_core::{ClickConfig, Engine, MouseButton, RunState, Snapshot, StopReason};
use nanoclick_platform::{
    portal_shortcut::{PortalShortcut, ShortcutUpdate},
    wayland::{PortalContext, WaylandInput},
};
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
    {
        let (service, state) = block_on(portal_mock::serve(Mode::Allow))?;
        let context = PortalContext::default();
        let engine = Engine::new(move |cancel| context.open(cancel), || {})?;
        for _ in 0..2 {
            engine.start(ClickConfig::new(10, MouseButton::Left, Some(1))?)?;
            await_state(&engine, |snapshot| {
                snapshot.reason == Some(StopReason::Completed)
            })?;
        }
        if lock(&state).persist_modes != [2, 2]
            || lock(&state).restore_tokens != [None, Some("mock-token-1".into())]
            || lock(&state).closes != 2
        {
            return Err(
                "RemoteDesktop did not renew/consume restoration tokens across runs".into(),
            );
        }
        drop(engine);
        block_on(service.release_name(NAME))?;
        println!("PASS portal lifecycle: restored-second-run");
    }
    for (name, mode, count) in [
        ("stalled-press", Mode::HoldPress, 1),
        ("stalled-release", Mode::HoldRelease, 2),
    ] {
        let (service, state) = block_on(portal_mock::serve(mode))?;
        let engine = Engine::new(WaylandInput::open, || {})?;
        engine.start(ClickConfig::new(10, MouseButton::Left, None)?)?;
        await_state(&engine, |_| lock(&state).buttons.len() >= count)?;
        let began = Instant::now();
        engine.stop();
        let terminal = await_state(&engine, |snapshot| !snapshot.state.is_active())?;
        if began.elapsed() > Duration::from_millis(250)
            || terminal.metrics.sent != 0
            || terminal.state != RunState::Faulted
            || lock(&state).closes != 1
        {
            return Err(
                format!("Stalled operation did not cancel safely: {name}: {terminal:?}").into(),
            );
        }
        println!(
            "portal_case,{name},stop_ms={:.3}",
            began.elapsed().as_secs_f64() * 1000.0
        );
        drop(engine);
        block_on(service.release_name(NAME))?;
    }
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
    shortcut_lifecycle()?;
    Ok(())
}
fn shortcut_lifecycle() -> Result<()> {
    for scenario in [
        "shortcut-owner-lost",
        "shortcut-owner-replaced",
        "shortcut-closed",
        "shortcut-closed-during-bind",
        "shortcut-owner-lost-during-bind",
    ] {
        let during_bind = scenario.ends_with("during-bind");
        let mode = match scenario {
            "shortcut-closed-during-bind" => Mode::CloseDuringSelect,
            "shortcut-owner-lost-during-bind" => Mode::HoldStart,
            _ => Mode::Allow,
        };
        let (service, state) = block_on(portal_mock::serve(mode))?;
        let (updates, received) = std::sync::mpsc::channel();
        let shortcut = PortalShortcut::configure(
            move |update| {
                let _ = updates.send(update);
            },
            || {},
        )?;
        if !during_bind {
            match received.recv_timeout(Duration::from_secs(4))? {
                ShortcutUpdate::Registered(trigger) if trigger == "F6" => {}
                update => {
                    return Err(format!("Unexpected shortcut registration: {update:?}").into());
                }
            }
        } else {
            let deadline = Instant::now() + Duration::from_secs(4);
            while lock(&state).starts == 0 {
                if Instant::now() >= deadline {
                    return Err("Shortcut did not reach BindShortcuts".into());
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
        let replacement = if scenario == "shortcut-owner-replaced" {
            let replacement = block_on(zbus::Connection::session())?;
            block_on(replacement.request_name_with_flags(
                NAME,
                zbus::fdo::RequestNameFlags::ReplaceExisting
                    | zbus::fdo::RequestNameFlags::DoNotQueue,
            ))?;
            Some(replacement)
        } else {
            if matches!(
                scenario,
                "shortcut-owner-lost" | "shortcut-owner-lost-during-bind"
            ) {
                block_on(service.release_name(NAME))?;
            } else if !during_bind {
                let session = lock(&state)
                    .session
                    .clone()
                    .ok_or("Missing shortcut session")?;
                block_on(portal_mock::closed(&service, &session))?;
            }
            None
        };
        match received.recv_timeout(Duration::from_secs(4))? {
            ShortcutUpdate::Unavailable(_) => {}
            update => return Err(format!("Shortcut lifecycle not invalidated: {update:?}").into()),
        }
        drop(shortcut);
        if matches!(scenario, "shortcut-closed" | "shortcut-closed-during-bind")
            && lock(&state).closes != 1
        {
            return Err("Shortcut session was not closed exactly once".into());
        }
        if let Some(replacement) = replacement {
            block_on(replacement.release_name(NAME))?;
        }
        let _ = block_on(service.release_name(NAME))?;
        println!("PASS portal lifecycle: {scenario}");
    }
    Ok(())
}
