//! Measures observer-inclusive Stop acknowledgement with a no-op input sink.
//! No desktop input is sent. This does not measure a native backend's latency.
use nanoclick_core::{BackendError, ClickConfig, Engine, InputSink, MouseButton, RunState};
use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
struct NoInput;
impl InputSink for NoInput {
    fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
        Ok(())
    }
}
fn await_state(
    engine: &Engine,
    receiver: &Receiver<()>,
    ready: impl Fn(nanoclick_core::Snapshot) -> bool,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let state = engine.snapshot();
        if state.state == RunState::Faulted {
            return Err(format!("Worker failed: {:?}", state.error).into());
        }
        if ready(state) {
            return Ok(());
        }
        receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))?;
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let engine = Engine::new(
        |_| Ok(NoInput),
        move || {
            let _ = sender.try_send(());
        },
    )?;
    println!("interval_ms,samples,p50_us,p95_us,p99_us,max_us");
    for interval in [1, 10, 100, 999_999] {
        let mut samples = Vec::with_capacity(1000);
        for iteration in 0..1000 {
            engine.start(ClickConfig::new(interval, MouseButton::Left, None)?)?;
            await_state(&engine, &receiver, |state| state.metrics.sent > 0)?;
            // Deterministic variation makes Stop land at different points in
            // the wait without requiring an RNG or busy-waiting.
            thread::sleep(Duration::from_micros((iteration * 137) % 2000));
            let begin = Instant::now();
            engine.stop();
            await_state(&engine, &receiver, |state| state.state == RunState::Idle)?;
            samples.push(begin.elapsed().as_nanos() as f64 / 1000.0);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "{interval},1000,{:.3},{:.3},{:.3},{:.3}",
            samples[499], samples[949], samples[989], samples[999]
        );
    }
    Ok(())
}
