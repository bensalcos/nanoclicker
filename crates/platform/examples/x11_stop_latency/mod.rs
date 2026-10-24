use nanoclick_core::{ClickConfig, Engine, MouseButton, RunState, Snapshot, StopReason};
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use x11rb::{connection::Connection, protocol::Event, rust_connection::RustConnection};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Default)]
struct Capture {
    pressed: bool,
    pairs: u64,
}
impl Capture {
    fn drain(&mut self, connection: &RustConnection) -> Result<()> {
        while let Some(event) = connection.poll_for_event()? {
            let (down, detail) = match event {
                Event::ButtonPress(event) => (true, event.detail),
                Event::ButtonRelease(event) => (false, event.detail),
                _ => continue,
            };
            if detail != 1 || down == self.pressed {
                return Err("Stop capture received a wrong button or unordered pair.".into());
            }
            self.pressed = down;
            if !down {
                self.pairs += 1;
            }
        }
        Ok(())
    }
}

fn observe(engine: &Engine, id: u64) -> Result<Snapshot> {
    let snapshot = engine.snapshot();
    if snapshot.run_id != id || snapshot.state == RunState::Faulted {
        return Err(format!("Stop session changed or failed: {:?}", snapshot.error).into());
    }
    Ok(snapshot)
}

pub fn measure(connection: &RustConnection, engine: &Engine) -> Result<()> {
    println!("scope,interval_ms,samples,p50_us,p95_us,p99_us,max_us,captured_pairs");
    // Reproducible xorshift delays, rather than a wall-clock-dependent seed.
    let mut random = 0x6e61_6e6f_636c_6963u64;
    for interval in [1, 10, 100, 999_999] {
        let mut samples = Vec::with_capacity(1000);
        let mut total_pairs = 0u64;
        for _ in 0..1000 {
            let mut capture = Capture::default();
            let id = engine.start(ClickConfig::new(interval, MouseButton::Left, None)?)?;
            let timeout = Instant::now() + Duration::from_secs(3);
            loop {
                capture.drain(connection)?;
                if observe(engine, id)?.metrics.sent > 0 && capture.pairs > 0 {
                    break;
                }
                if Instant::now() > timeout {
                    return Err("First X11 pair timed out.".into());
                }
                thread::sleep(Duration::from_micros(100));
            }
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            thread::sleep(Duration::from_micros(random % 2001));
            let began = Instant::now();
            engine.stop();
            let timeout = began + Duration::from_secs(3);
            let terminal = loop {
                let snapshot = observe(engine, id)?;
                if snapshot.state == RunState::Idle {
                    samples.push(began.elapsed().as_secs_f64() * 1e6);
                    break snapshot;
                }
                capture.drain(connection)?;
                if Instant::now() > timeout {
                    return Err("X11 Stop acknowledgement timed out.".into());
                }
                thread::sleep(Duration::from_micros(100));
            };
            if terminal.reason != Some(StopReason::Stopped) || terminal.metrics.sent == 0 {
                return Err("Missing successful terminal Stop snapshot.".into());
            }
            loop {
                capture.drain(connection)?;
                if capture.pairs > terminal.metrics.sent {
                    return Err("Capture received more pairs than the terminal snapshot.".into());
                }
                if capture.pairs == terminal.metrics.sent && !capture.pressed {
                    break;
                }
                if Instant::now() > timeout {
                    return Err("Terminal pair capture timed out.".into());
                }
                thread::sleep(Duration::from_micros(100));
            }
            total_pairs += capture.pairs;
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "x11_native_input,{interval},1000,{:.3},{:.3},{:.3},{:.3},{total_pairs}",
            samples[499], samples[949], samples[989], samples[999]
        );
    }
    Ok(())
}
