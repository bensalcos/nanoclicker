//! Run only inside the private Xvfb started by scripts/verify-x11.sh.
mod x11_stop_latency;
use nanoclick_core::{ClickConfig, Engine, MouseButton, RunState, StopReason};
use nanoclick_platform::{Desktop, NativeInput};
use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt, CreateWindowAux, EventMask, InputFocus, WindowClass},
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("NANOCLICK_ISOLATED_X11").as_deref() != Ok("1") {
        return Err(
            "Use scripts/verify-x11.sh; desktop injection is disabled outside isolation.".into(),
        );
    }
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let stop_latency = arguments.as_slice() == ["--stop-latency"];
    let timing = match arguments.as_slice() {
        [mode] if mode == "--stop-latency" => None,
        [] => None,
        [seconds, workers] => {
            let seconds = seconds.parse::<u64>()?;
            let workers = workers.parse::<usize>()?;
            if !(1..=3600).contains(&seconds) || workers > 2 {
                return Err("Timing requires 1..3600 seconds and 0..2 load threads.".into());
            }
            Some((seconds, workers))
        }
        _ => return Err("Usage: x11_acceptance [timing_seconds load_threads]".into()),
    };
    let (capture, screen_number) = x11rb::connect(None)?;
    let screen = &capture.setup().roots[screen_number];
    let window = capture.generate_id()?;
    capture
        .create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            300,
            200,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE),
        )?
        .check()?;
    capture.map_window(window)?.check()?;
    capture
        .set_input_focus(InputFocus::PARENT, window, x11rb::CURRENT_TIME)?
        .check()?;
    capture
        .warp_pointer(x11rb::NONE, window, 0, 0, 0, 0, 100, 100)?
        .check()?;
    capture.flush()?;
    let engine = Engine::new(|token| NativeInput::open(Desktop::X11, token), || {})?;
    for (button, detail) in [
        (MouseButton::Left, 1),
        (MouseButton::Middle, 2),
        (MouseButton::Right, 3),
    ] {
        let id = engine.start(ClickConfig::new(10, button, Some(10))?)?;
        let timeout = Instant::now() + Duration::from_secs(5);
        let mut events = Vec::new();
        loop {
            while let Some(event) = capture.poll_for_event()? {
                match event {
                    Event::ButtonPress(event) => events.push((true, event.detail)),
                    Event::ButtonRelease(event) => events.push((false, event.detail)),
                    _ => {}
                }
            }
            let snapshot = engine.snapshot();
            if snapshot.run_id == id
                && snapshot.reason == Some(StopReason::Completed)
                && events.len() == 20
            {
                break;
            }
            if snapshot.state == RunState::Faulted {
                return Err(format!("Backend fault: {:?}", snapshot.error).into());
            }
            if Instant::now() > timeout {
                return Err(format!("Incomplete {button:?}: {events:?}").into());
            }
            thread::sleep(Duration::from_millis(1));
        }
        let expected: Vec<_> = (0..10)
            .flat_map(|_| [(true, detail), (false, detail)])
            .collect();
        if events != expected {
            return Err(format!("Wrong event order for {button:?}: {events:?}").into());
        }
        println!("PASS {button:?}: 10 complete pairs received in order");
    }
    {
        use nanoclick_platform::x11_shortcut::X11Shortcut;
        use std::sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        };
        use x11rb::protocol::xtest::ConnectionExt as _;
        let activations = Arc::new(AtomicU64::new(0));
        let observed = activations.clone();
        let (failed, failure) = std::sync::mpsc::sync_channel(1);
        let _shortcut = X11Shortcut::register(
            "F6".parse()?,
            move || {
                observed.fetch_add(1, Ordering::Relaxed);
            },
            move |error| {
                let _ = failed.send(error);
            },
        )?;
        if X11Shortcut::register("F6".parse()?, || {}, |_| {}).is_ok() {
            return Err("A conflicting shortcut was incorrectly accepted.".into());
        }
        let first = capture.setup().min_keycode;
        let map = capture
            .get_keyboard_mapping(first, capture.setup().max_keycode - first + 1)?
            .reply()?;
        let offset = map
            .keysyms
            .chunks(usize::from(map.keysyms_per_keycode))
            .position(|symbols| symbols.contains(&0xffc3))
            .ok_or("F6 not mapped")?;
        let key = u8::try_from(usize::from(first) + offset)?;
        capture
            .xtest_fake_input(2, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        thread::sleep(Duration::from_millis(900));
        capture
            .xtest_fake_input(3, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        let timeout = Instant::now() + Duration::from_secs(2);
        while activations.load(Ordering::Relaxed) < 1 && Instant::now() < timeout {
            thread::sleep(Duration::from_millis(1));
        }
        if activations.load(Ordering::Relaxed) != 1 {
            return Err("Held shortcut toggled more than once.".into());
        }
        thread::sleep(Duration::from_millis(20));
        capture
            .xtest_fake_input(2, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        capture
            .xtest_fake_input(3, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        let timeout = Instant::now() + Duration::from_secs(2);
        while activations.load(Ordering::Relaxed) < 2 && Instant::now() < timeout {
            thread::sleep(Duration::from_millis(1));
        }
        if activations.load(Ordering::Relaxed) != 2 {
            return Err("The original shortcut was lost after a conflicting registration.".into());
        }
        let pointer_map = capture.get_pointer_mapping()?.reply()?.map;
        if pointer_map.len() < 2 {
            return Err("Pointer mapping fixture is too small".into());
        }
        let mut changed = pointer_map.clone();
        let last = changed.len() - 1;
        changed.swap(last - 1, last);
        for mapping in [&changed, &pointer_map] {
            if capture.set_pointer_mapping(mapping)?.reply()?.status
                != x11rb::protocol::xproto::MappingStatus::SUCCESS
            {
                return Err("Isolated pointer mapping could not be changed/restored".into());
            }
        }
        capture
            .xtest_fake_input(2, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        capture
            .xtest_fake_input(3, key, 0, x11rb::NONE, 0, 0, 0)?
            .check()?;
        let timeout = Instant::now() + Duration::from_secs(2);
        while activations.load(Ordering::Relaxed) < 3 && Instant::now() < timeout {
            if let Ok(error) = failure.try_recv() {
                return Err(
                    format!("Pointer mapping invalidated keyboard shortcut: {error}").into(),
                );
            }
            thread::sleep(Duration::from_millis(1));
        }
        if activations.load(Ordering::Relaxed) != 3 {
            return Err("Shortcut stopped working after pointer mapping".into());
        }
        if let Ok(error) = failure.try_recv() {
            return Err(error.into());
        }
        println!(
            "PASS X11 shortcut: outside-focus activation, held-key suppression, conflict and pointer-map changes preserve binding"
        );
        let width = usize::from(map.keysyms_per_keycode);
        let original = &map.keysyms[offset * width..(offset + 1) * width];
        let changed: Vec<_> = original
            .iter()
            .map(|symbol| if *symbol == 0xffc3 { 0xffc4 } else { *symbol })
            .collect();
        for symbols in [&changed[..], original] {
            capture
                .change_keyboard_mapping(1, key, map.keysyms_per_keycode, symbols)?
                .check()?;
        }
        let error = failure.recv_timeout(Duration::from_secs(2))?;
        if !error.contains("Keyboard mapping changed") {
            return Err(format!("Keyboard mapping invalidation was not reported: {error}").into());
        }
        println!("PASS X11 shortcut: keyboard mapping changes invalidate the binding explicitly");
    }
    if let Some((seconds, workers)) = timing {
        measure_deadlines(&capture, &engine, seconds, workers)?;
    }
    if stop_latency {
        x11_stop_latency::measure(&capture, &engine)?;
    }
    capture.destroy_window(window)?.check()?;
    Ok(())
}

// RAII also stops load threads if input, capture or timeout validation fails.
struct Load {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
}
impl Load {
    fn start(workers: usize) -> Self {
        let mut load = Self {
            stop: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            threads: Vec::with_capacity(workers),
        };
        for _ in 0..workers {
            let stop = load.stop.clone();
            load.threads.push(thread::spawn(move || {
                let mut value = 1u64;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    value = std::hint::black_box(
                        value.wrapping_mul(6364136223846793005).wrapping_add(1),
                    );
                }
            }));
        }
        load
    }
}
impl Drop for Load {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        for worker in self.threads.drain(..) {
            let _ = worker.join();
        }
    }
}

fn measure_deadlines(
    capture: &x11rb::rust_connection::RustConnection,
    engine: &Engine,
    seconds: u64,
    workers: usize,
) -> Result<(), Box<dyn Error>> {
    let _load = Load::start(workers);
    let began = Instant::now();
    let id = engine.start(ClickConfig::new(10, MouseButton::Left, None)?)?;
    let stop_at = began + Duration::from_secs(seconds);
    let timeout = stop_at + Duration::from_secs(3);
    let mut stopping = false;
    let mut pressed = false;
    let mut pairs = 0u64;
    let snapshot = loop {
        while let Some(event) = capture.poll_for_event()? {
            let (down, detail) = match event {
                Event::ButtonPress(event) => (true, event.detail),
                Event::ButtonRelease(event) => (false, event.detail),
                _ => continue,
            };
            if detail != 1 || down == pressed {
                return Err("Timing capture received a wrong button or unordered pair.".into());
            }
            pressed = down;
            if !down {
                pairs += 1;
            }
        }
        let snapshot = engine.snapshot();
        if snapshot.run_id != id || snapshot.state == RunState::Faulted {
            return Err(format!("Timing session changed or failed: {:?}", snapshot.error).into());
        }
        if stopping
            && snapshot.state == RunState::Idle
            && pairs == snapshot.metrics.sent
            && !pressed
        {
            break snapshot;
        }
        if Instant::now() > timeout {
            return Err("Timing session did not stop with matching captured pairs.".into());
        }
        if !stopping && Instant::now() >= stop_at {
            engine.stop();
            stopping = true;
        }
        thread::sleep(Duration::from_millis(1));
    };
    if snapshot.reason != Some(StopReason::Stopped) || pairs == 0 {
        return Err("Timing run did not preserve a successful Stop acknowledgement.".into());
    }
    let metrics = snapshot.metrics;
    println!(
        "scope,load_threads,elapsed_s,attempts,sent,captured_pairs,skipped,p50_upper_us,p95_upper_us,p99_upper_us,max_us"
    );
    println!(
        "x11_native_input,{workers},{:.6},{},{},{pairs},{},{:.3},{:.3},{:.3},{:.3}",
        began.elapsed().as_secs_f64(),
        metrics.attempts,
        metrics.sent,
        metrics.skipped,
        metrics.lateness_p50.as_secs_f64() * 1e6,
        metrics.lateness_p95.as_secs_f64() * 1e6,
        metrics.lateness_p99.as_secs_f64() * 1e6,
        metrics.max_lateness.as_secs_f64() * 1e6
    );
    Ok(())
}
