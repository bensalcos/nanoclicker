//! Run only inside the private Xvfb started by scripts/verify-x11.sh.
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
        if let Ok(error) = failure.try_recv() {
            return Err(error.into());
        }
        println!(
            "PASS X11 shortcut: outside-focus activation, held-key suppression and conflict preserves binding"
        );
    }
    capture.destroy_window(window)?.check()?;
    Ok(())
}
