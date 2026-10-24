//! Opt-in aggregate terminal measurements; no per-click output or saved run state.
use nanoclick_core::{Snapshot, StopReason};
use nanoclick_platform::Desktop;
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::Path,
};

pub struct Diagnostics<W = BufWriter<File>> {
    output: W,
    last_run: u64,
}
impl Diagnostics {
    pub fn create(path: &Path) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        Ok(Self {
            output: BufWriter::new(options.open(path)?),
            last_run: 0,
        })
    }
}
#[derive(Serialize)]
struct TerminalRecord {
    schema_version: u32,
    app_version: &'static str,
    backend: &'static str,
    renderer: &'static str,
    run_id: u64,
    reason: &'static str,
    attempts: u64,
    sent: u64,
    failures: u64,
    skipped: u64,
    lateness_p50_ns: u128,
    lateness_p95_ns: u128,
    lateness_p99_ns: u128,
    max_lateness_ns: u128,
}
impl<W: Write> Diagnostics<W> {
    /// Writes only terminal snapshots actually observed by the UI. This is not
    /// a durable audit of every run, especially across abrupt process exits.
    pub fn observe(&mut self, snapshot: &Snapshot, desktop: Desktop) -> io::Result<()> {
        if snapshot.state.is_active() || snapshot.run_id <= self.last_run {
            return Ok(());
        }
        let Some(reason) = snapshot.reason else {
            return Ok(());
        };
        let metrics = snapshot.metrics;
        let record = TerminalRecord {
            schema_version: 1,
            app_version: env!("CARGO_PKG_VERSION"),
            backend: match desktop {
                Desktop::Windows => "windows",
                Desktop::X11 => "x11",
                Desktop::Wayland => "wayland",
            },
            renderer: if cfg!(feature = "glow") {
                "glow"
            } else {
                "wgpu"
            },
            run_id: snapshot.run_id,
            reason: match reason {
                StopReason::Stopped => "stopped",
                StopReason::Completed => "completed",
                StopReason::Failed => "failed",
            },
            attempts: metrics.attempts,
            sent: metrics.sent,
            failures: metrics.failures,
            skipped: metrics.skipped,
            lateness_p50_ns: metrics.lateness_p50.as_nanos(),
            lateness_p95_ns: metrics.lateness_p95.as_nanos(),
            lateness_p99_ns: metrics.lateness_p99.as_nanos(),
            max_lateness_ns: metrics.max_lateness.as_nanos(),
        };
        serde_json::to_writer(&mut self.output, &record).map_err(io::Error::other)?;
        self.output.write_all(b"\n")?;
        self.output.flush()?;
        self.last_run = snapshot.run_id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nanoclick_core::{Metrics, RunState, StopReason};
    use std::time::Duration;
    #[test]
    fn records_each_observed_terminal_run_once_without_error_or_settings_text() -> io::Result<()> {
        let mut recorder = Diagnostics {
            output: Vec::new(),
            last_run: 0,
        };
        recorder.observe(&Snapshot::default(), Desktop::X11)?;
        let mut snapshot = Snapshot {
            run_id: 1,
            state: RunState::Running,
            ..Default::default()
        };
        recorder.observe(&snapshot, Desktop::X11)?;
        assert!(recorder.output.is_empty());
        snapshot.state = RunState::Idle;
        snapshot.reason = Some(StopReason::Stopped);
        snapshot.metrics = Metrics {
            attempts: 12,
            sent: 12,
            lateness_p50: Duration::from_micros(100),
            lateness_p95: Duration::from_micros(300),
            lateness_p99: Duration::from_micros(400),
            max_lateness: Duration::from_micros(425),
            ..Default::default()
        };
        snapshot.error = Some("must never export raw error text".into());
        recorder.observe(&snapshot, Desktop::X11)?;
        recorder.observe(&snapshot, Desktop::X11)?;
        let text = String::from_utf8(recorder.output).map_err(io::Error::other)?;
        assert_eq!(text.lines().count(), 1);
        let record: serde_json::Value = serde_json::from_str(&text)?;
        assert_eq!(record["schema_version"], 1);
        assert_eq!(record["backend"], "x11");
        assert_eq!(record["run_id"], 1);
        assert_eq!(record["sent"], 12);
        assert_eq!(record["lateness_p99_ns"], 400_000);
        assert_eq!(record["max_lateness_ns"], 425_000);
        assert!(!text.contains("must never") && record.get("settings").is_none());
        Ok(())
    }
    #[test]
    fn creating_output_never_overwrites_an_existing_path() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("measurements.jsonl");
        std::fs::write(&path, b"preserved")?;
        assert!(
            matches!(Diagnostics::create(&path), Err(error) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        assert_eq!(std::fs::read(&path)?, b"preserved");
        let path = directory.path().join("new.jsonl");
        let _recorder = Diagnostics::create(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            assert_eq!(std::fs::metadata(path)?.permissions().mode() & 0o777, 0o600);
        }
        Ok(())
    }
    #[test]
    fn reports_writer_failure_without_claiming_the_run_was_recorded() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("fixture write failure"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut recorder = Diagnostics {
            output: Broken,
            last_run: 0,
        };
        let snapshot = Snapshot {
            run_id: 3,
            state: RunState::Faulted,
            reason: Some(StopReason::Failed),
            ..Default::default()
        };
        assert!(recorder.observe(&snapshot, Desktop::Wayland).is_err());
        assert_eq!(recorder.last_run, 0);
    }
}
