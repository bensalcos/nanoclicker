//! Measures the actual GUI executable on a private X11 test display, Linux only.
mod support;
use std::{
    collections::BTreeMap,
    fs,
    process::Command,
    thread,
    time::{Duration, Instant},
};
use support::{Result, close, launch, pair, ready_button, wait};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt, CreateWindowAux, EventMask, WindowClass},
    },
};

fn ticks(pid: u32) -> Result<u64> {
    let content = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let fields: Vec<_> = content
        .rsplit_once(") ")
        .ok_or("Invalid process stat")?
        .1
        .split_whitespace()
        .collect();
    if fields.len() < 13 {
        return Err("Incomplete process stat".into());
    }
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}
fn rss_mib(pid: u32) -> Result<f64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
    let line = status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .ok_or("VmRSS unavailable")?;
    Ok(line
        .split_whitespace()
        .nth(1)
        .ok_or("Invalid VmRSS")?
        .parse::<f64>()?
        / 1024.0)
}
fn mapping_rss(pid: u32) -> Result<()> {
    let content = fs::read_to_string(format!("/proc/{pid}/smaps"))?;
    let mut mapping = String::new();
    let mut totals = BTreeMap::<String, u64>::new();
    for line in content.lines() {
        let mut fields = line.split_whitespace();
        let Some(first) = fields.next() else {
            continue;
        };
        if first.contains('-') && first.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            let path = fields.skip(4).collect::<Vec<_>>().join(" ");
            mapping = if path.is_empty() {
                "[anonymous]".into()
            } else {
                std::path::Path::new(&path)
                    .file_name()
                    .ok_or("Invalid mapping path")?
                    .to_string_lossy()
                    .into_owned()
            };
        } else if first == "Rss:" {
            *totals.entry(mapping.clone()).or_default() +=
                fields.next().ok_or("Missing mapping RSS")?.parse::<u64>()?;
        }
    }
    let mut totals: Vec<_> = totals.into_iter().collect();
    totals.sort_by_key(|(_, rss)| std::cmp::Reverse(*rss));
    for (name, rss) in totals.into_iter().take(12) {
        println!("idle_mapping_rss_kib,{name},{rss}");
    }
    Ok(())
}
fn thread_ticks(pid: u32) -> Result<BTreeMap<String, (String, u64)>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(format!("/proc/{pid}/task"))? {
        let entry = entry?;
        let content = fs::read_to_string(entry.path().join("stat"))?;
        let (name, fields) = content.rsplit_once(") ").ok_or("Invalid thread stat")?;
        let fields: Vec<_> = fields.split_whitespace().collect();
        let name = name
            .split_once('(')
            .ok_or("Missing thread name")?
            .1
            .to_owned();
        result.insert(
            entry.file_name().to_string_lossy().into_owned(),
            (
                name,
                fields[11].parse::<u64>()? + fields[12].parse::<u64>()?,
            ),
        );
    }
    Ok(result)
}
fn main() -> Result<()> {
    if std::env::var("NANOCLICK_ISOLATED_X11").as_deref() != Ok("1") {
        return Err(
            "Use scripts/measure-ui-x11.sh or measure-ui-gpu.sh; requires an isolated display."
                .into(),
        );
    }
    let binary = std::env::args_os()
        .nth(1)
        .ok_or("Pass the release executable path.")?;
    let quick = std::env::args().any(|argument| argument == "--quick-profile");
    let duration = Duration::from_secs(if quick { 10 } else { 60 });
    let clock_hz = String::from_utf8(Command::new("getconf").arg("CLK_TCK").output()?.stdout)?
        .trim()
        .parse::<f64>()?;
    let (conn, screen) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen].root;
    let mut startup = Vec::new();
    for index in 0..if quick { 1 } else { 31 } {
        let (mut app, window, elapsed) = launch(&conn, root, &binary)?;
        if index != 0 {
            startup.push(elapsed.as_secs_f64() * 1000.0);
            println!(
                "startup_sample_ms,{index},{:.3}",
                elapsed.as_secs_f64() * 1000.0
            );
        }
        close(&conn, window, &mut app)?;
    }
    startup.sort_by(f64::total_cmp);
    if !startup.is_empty() {
        println!("startup_p95_ms,{:.3}", startup[28]);
    }
    let diagnostics_path =
        std::env::var_os("NANOCLICK_DIAGNOSTICS_PATH").map(std::path::PathBuf::from);
    let (mut app, window, _) =
        support::launch_with_diagnostics(&conn, root, &binary, diagnostics_path.as_deref())?;
    let pid = app.0.id();
    println!("phase,stabilizing,{}s", duration.as_secs());
    thread::sleep(duration);
    let before = ticks(pid)?;
    let began = Instant::now();
    thread::sleep(duration);
    println!(
        "idle_cpu_percent_one_core,{:.4}",
        (ticks(pid)? - before) as f64 / clock_hz / began.elapsed().as_secs_f64() * 100.0
    );
    println!("idle_rss_mib,{:.3}", rss_mib(pid)?);
    mapping_rss(pid)?;
    let target = conn.generate_id()?;
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        target,
        root,
        700,
        100,
        250,
        250,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new()
            .override_redirect(1u32)
            .background_pixel(0x202020)
            .event_mask(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE),
    )?
    .check()?;
    conn.map_window(target)?.check()?;
    support::focus_target(&conn, target)?;
    let first = conn.setup().min_keycode;
    let map = conn
        .get_keyboard_mapping(first, conn.setup().max_keycode - first + 1)?
        .reply()?;
    let offset = map
        .keysyms
        .chunks(usize::from(map.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&0xffc3))
        .ok_or("F6 not mapped")?;
    let key = u8::try_from(usize::from(first) + offset)?;
    pair(&conn, key)?;
    let before = ticks(pid)?;
    let began = Instant::now();
    let thread_before = thread_ticks(pid)?;
    let mut presses = Vec::with_capacity(7000);
    let mut releases = 0;
    println!("phase,active,{}s", duration.as_secs());
    while began.elapsed() < duration {
        while let Some(event) = conn.poll_for_event()? {
            match event {
                Event::ButtonPress(event) => presses.push(event.time),
                Event::ButtonRelease(_) => releases += 1,
                _ => {}
            }
        }
        if presses.len() > 10_000 {
            return Err("Unexpected input burst during benchmark".into());
        }
        thread::sleep(Duration::from_millis(5));
    }
    println!(
        "active_cpu_percent_one_core,{:.4}",
        (ticks(pid)? - before) as f64 / clock_hz / began.elapsed().as_secs_f64() * 100.0
    );
    println!("active_rss_mib,{:.3}", rss_mib(pid)?);
    let mut threads: Vec<_> = thread_ticks(pid)?
        .into_iter()
        .map(|(id, (name, after))| {
            let before = thread_before.get(&id).map_or(0, |value| value.1);
            (after.saturating_sub(before), name)
        })
        .collect();
    threads.sort_by_key(|value| std::cmp::Reverse(value.0));
    for (delta, name) in threads.into_iter().take(8) {
        println!("thread_cpu_ticks,{name},{delta}");
    }

    pair(&conn, key)?;
    wait(|| Ok(ready_button(&conn, window)), Duration::from_secs(3))?;
    while let Some(event) = conn.poll_for_event()? {
        match event {
            Event::ButtonPress(event) => presses.push(event.time),
            Event::ButtonRelease(_) => releases += 1,
            _ => {}
        }
    }
    if presses.len() != releases || presses.len() < duration.as_secs() as usize * 80 {
        return Err(format!(
            "Incomplete input sample: {} down, {releases} up",
            presses.len()
        )
        .into());
    }
    let mut jitter: Vec<_> = presses
        .windows(2)
        .map(|p| p[1].wrapping_sub(p[0]).abs_diff(10))
        .collect();
    jitter.sort_unstable();
    println!("received_pairs,{}", presses.len());
    println!(
        "interval_abs_error_p99_ms,{}",
        jitter[(jitter.len() * 99).div_ceil(100) - 1]
    );
    println!("interval_abs_error_max_ms,{}", jitter[jitter.len() - 1]);
    if let Some(path) = diagnostics_path {
        let text = fs::read_to_string(path)?;
        let records: Vec<serde_json::Value> = text
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        if records.len() != 1 {
            return Err("Expected one observed terminal diagnostics record".into());
        }
        let record = &records[0];
        if record["schema_version"] != 1
            || record["backend"] != "x11"
            || record["reason"] != "stopped"
            || record["sent"].as_u64() != Some(presses.len() as u64)
            || record["failures"].as_u64() != Some(0)
        {
            return Err(format!("Diagnostics do not match captured input: {record}").into());
        }
        let mut previous = 0;
        for field in [
            "lateness_p50_ns",
            "lateness_p95_ns",
            "lateness_p99_ns",
            "max_lateness_ns",
        ] {
            let value = record[field]
                .as_u64()
                .ok_or("Missing diagnostic lateness")?;
            if value < previous {
                return Err("Diagnostic quantiles are unordered".into());
            }
            previous = value;
            println!("engine_{field},{value}");
        }
        println!("engine_skipped,{}", record["skipped"]);
    }
    close(&conn, window, &mut app)?;
    Ok(())
}
