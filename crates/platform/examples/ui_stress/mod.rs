//! Bounded lifecycle/resource exercise of the real application process.
use crate::{
    active_button,
    support::{Result, pair, ready_button, wait},
};
use std::{ffi::OsStr, fs, path::Path, thread, time::Duration};
use x11rb::{connection::Connection, protocol::Event, rust_connection::RustConnection};

struct Resources {
    rss_kib: u64,
    descriptors: usize,
    threads: usize,
}
fn count_entries(path: String) -> std::io::Result<usize> {
    fs::read_dir(path)?.try_fold(0usize, |count, entry| entry.map(|_| count + 1))
}
fn sample(pid: u32) -> Result<Resources> {
    let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
    let rss_kib = status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("App RSS unavailable")?
        .parse()?;
    Ok(Resources {
        rss_kib,
        descriptors: count_entries(format!("/proc/{pid}/fd"))?,
        threads: count_entries(format!("/proc/{pid}/task"))?,
    })
}
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
                return Err("UI stress received a wrong button or unordered pair".into());
            }
            self.pressed = down;
            if !down {
                self.pairs += 1;
            }
        }
        Ok(())
    }
}
pub fn run(
    connection: &RustConnection,
    window: u32,
    key: u8,
    pid: u32,
    binary: &OsStr,
    cycles: u32,
) -> Result<()> {
    if fs::read_link(format!("/proc/{pid}/exe"))? != Path::new(binary).canonicalize()? {
        return Err(
            "Resource stress requires the direct application executable, without a wrapper".into(),
        );
    }
    let warmup = 10;
    let mut baseline = None;
    let mut total_pairs = 0u64;
    println!("scope,completed_cycles,rss_kib,descriptors,threads,captured_pairs");
    for cycle in 1..=warmup + cycles {
        let mut capture = Capture::default();
        pair(connection, key)?;
        wait(
            || {
                capture.drain(connection)?;
                Ok(capture.pairs > 0 && active_button(connection, window))
            },
            Duration::from_secs(3),
        )?;
        pair(connection, key)?;
        wait(
            || {
                capture.drain(connection)?;
                Ok(ready_button(connection, window) && !capture.pressed)
            },
            Duration::from_secs(3),
        )?;
        let stopped_pairs = capture.pairs;
        thread::sleep(Duration::from_millis(25));
        capture.drain(connection)?;
        if capture.pairs != stopped_pairs || capture.pressed {
            return Err("UI stress received input after Stop acknowledgement".into());
        }
        total_pairs += capture.pairs;
        if cycle == warmup
            || (cycle > warmup && ((cycle - warmup).is_multiple_of(20) || cycle == warmup + cycles))
        {
            let resources = sample(pid)?;
            println!(
                "ui_x11_stress,{},{},{},{},{total_pairs}",
                cycle.saturating_sub(warmup),
                resources.rss_kib,
                resources.descriptors,
                resources.threads
            );
            if let Some(descriptors) = baseline {
                if resources.descriptors != descriptors {
                    return Err("Application descriptor count changed after warm-up; investigate before accepting lifecycle stability".into());
                }
            } else {
                baseline = Some(resources.descriptors);
            }
        }
    }
    println!(
        "PASS GUI stress: {warmup} warm-up and {cycles} measured Start/Stop cycles; ordered pairs, no post-stop input and stable descriptors"
    );
    Ok(())
}
