//! Native-waiter/core timing baseline. Sends no input and opens no UI or portal.
use nanoclick_core::{BackendError, ClickConfig, Engine, InputSink, MouseButton, RunState};
use std::{
    error::Error,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

struct NoInput;
impl InputSink for NoInput {
    fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let seconds = std::env::args()
        .nth(1)
        .map(|value| value.parse::<u64>())
        .transpose()?
        .unwrap_or(60);
    if !(1..=3600).contains(&seconds) {
        return Err("Measurement duration must be 1..3600 seconds.".into());
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    let engine = Engine::with_waiter(
        |_| Ok(NoInput),
        nanoclick_platform::waiter()?,
        move || {
            let _ = sender.try_send(());
        },
    )?;
    let began = Instant::now();
    engine.start(ClickConfig::new(10, MouseButton::Left, None)?)?;
    thread::sleep(Duration::from_secs(seconds));
    engine.stop();
    let timeout = Instant::now() + Duration::from_secs(3);
    let snapshot = loop {
        let snapshot = engine.snapshot();
        if snapshot.state == RunState::Faulted {
            return Err(format!("Timing run failed: {:?}", snapshot.error).into());
        }
        if snapshot.state == RunState::Idle {
            break snapshot;
        }
        receiver.recv_timeout(timeout.saturating_duration_since(Instant::now()))?;
    };
    let metrics = snapshot.metrics;
    if metrics.sent == 0 {
        return Err("No scheduler samples were collected.".into());
    }
    println!("scope,elapsed_s,attempts,sent,skipped,p50_upper_us,p95_upper_us,p99_upper_us,max_us");
    println!(
        "native_waiter_no_input,{:.6},{},{},{},{:.3},{:.3},{:.3},{:.3}",
        began.elapsed().as_secs_f64(),
        metrics.attempts,
        metrics.sent,
        metrics.skipped,
        metrics.lateness_p50.as_secs_f64() * 1e6,
        metrics.lateness_p95.as_secs_f64() * 1e6,
        metrics.lateness_p99.as_secs_f64() * 1e6,
        metrics.max_lateness.as_secs_f64() * 1e6,
    );
    Ok(())
}
