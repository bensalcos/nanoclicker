//! Controlled by verify-x11-stall.py with its own Xvfb and authority FIFO.
#![forbid(unsafe_code)]
use nanoclick_core::{BackendError, ClickConfig, Engine, MouseButton, RunState, StopReason};
use nanoclick_platform::{Desktop, NativeInput};
use std::{
    io::{self, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
fn instruction(expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    if line.trim() != expected {
        return Err("Unexpected isolated-server driver instruction".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("NANOCLICK_ISOLATED_X11").as_deref() != Ok("1")
        || std::env::var("NANOCLICK_X11_STALL_DRIVER").as_deref() != Ok("1")
    {
        return Err("Use scripts/verify-x11-stall.py; a test-owned server is required".into());
    }
    let scenario = std::env::args().nth(1).ok_or("Pass a fixture scenario")?;
    if matches!(
        scenario.as_str(),
        "shortcut-registration" | "authority-shortcut"
    ) {
        println!("READY");
        io::stdout().flush()?;
        instruction("paused")?;
        println!("ARMED");
        io::stdout().flush()?;
        let began = Instant::now();
        let result =
            nanoclick_platform::x11_shortcut::X11Shortcut::register("F6".parse()?, || {}, |_| {});
        if result.is_ok() {
            return Err("Shortcut registered against a blocked setup or paused server".into());
        }
        println!(
            "PASS {}: returned in {:.3} ms",
            scenario,
            began.elapsed().as_secs_f64() * 1000.0
        );
        return Ok(());
    }
    let preparation = match scenario.as_str() {
        "preparation" | "authority" | "authority-timeout" => true,
        "input" | "input-no-stop" | "disconnect" => false,
        _ => {
            return Err("Unknown isolated-server fixture scenario".into());
        }
    };
    let (ready, received) = mpsc::sync_channel(1);
    let (permit, permitted) = mpsc::sync_channel(1);
    let engine = Engine::new(
        move |cancel| {
            let prepared = if preparation {
                None
            } else {
                Some(NativeInput::open(Desktop::X11, cancel)?)
            };
            ready
                .send(())
                .map_err(|e| BackendError::new(e.to_string()))?;
            permitted
                .recv()
                .map_err(|e| BackendError::new(e.to_string()))?;
            match prepared {
                Some(sink) => Ok(sink),
                None => NativeInput::open(Desktop::X11, cancel),
            }
        },
        || {},
    )?;
    engine.start(ClickConfig::new(10, MouseButton::Left, None)?)?;
    received.recv_timeout(Duration::from_secs(4))?;
    println!("READY");
    io::stdout().flush()?;
    instruction("paused")?;
    permit.send(())?;
    println!("ARMED");
    io::stdout().flush()?;
    // The driver confirms a protocol poll or the setup helper's FIFO open wait.
    instruction(
        if matches!(scenario.as_str(), "input-no-stop" | "authority-timeout") {
            "observe"
        } else {
            "stop"
        },
    )?;
    let began = Instant::now();
    if !matches!(scenario.as_str(), "input-no-stop" | "authority-timeout") {
        engine.stop();
    }
    loop {
        let snapshot = engine.snapshot();
        if !snapshot.state.is_active() {
            if preparation {
                if snapshot.metrics.attempts != 0
                    || (scenario == "authority-timeout"
                        && (snapshot.state != RunState::Faulted
                            || snapshot.reason != Some(StopReason::Failed)
                            || snapshot.error.is_none()))
                    || (scenario != "authority-timeout"
                        && snapshot.reason != Some(StopReason::Stopped))
                {
                    return Err(
                        format!("Preparation cancellation was not clean: {snapshot:?}").into(),
                    );
                }
            } else if snapshot.state != RunState::Faulted
                || snapshot.reason != Some(StopReason::Failed)
                || snapshot.error.is_none()
                || snapshot.metrics.attempts != 1
                || snapshot.metrics.sent != 0
                || snapshot.metrics.failures != 1
            {
                return Err(
                    format!("Unverified input was not reported as failure: {snapshot:?}").into(),
                );
            }
            println!(
                "PASS {case}: terminal in {elapsed:.3} ms; error={error:?}",
                case = scenario,
                elapsed = began.elapsed().as_secs_f64() * 1000.0,
                error = snapshot.error
            );
            return Ok(());
        }
        if began.elapsed()
            > Duration::from_secs(if scenario == "authority-timeout" {
                5
            } else {
                2
            })
        {
            println!("FAIL terminal timeout");
            io::stdout().flush()?;
            return Err(
                "The X11 operation did not terminate while the private resource remained blocked"
                    .into(),
            );
        }
        thread::sleep(Duration::from_millis(1));
    }
}
