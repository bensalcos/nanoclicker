//! Fixed-space window-event accounting; never injects or stores raw input.
use eframe::egui;
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};

pub const BUTTONS: [&str; 5] = ["left", "right", "middle", "extra1", "extra2"];
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct ButtonStats {
    pub presses: u64,
    pub releases: u64,
    pub pairs: u64,
    pub anomalies: u64,
    pub held: bool,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub session_id: u64,
    pub buttons: [ButtonStats; 5],
    pub overlap_presses: u64,
    pub overflowed: bool,
    pub balanced: bool,
    pub unexpected_buttons: bool,
}
#[derive(Default)]
pub struct Capture {
    id: u64,
    active: bool,
    buttons: [ButtonStats; 5],
    overlap_presses: u64,
    overflowed: bool,
}
impl Capture {
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn issue(&self) -> Option<&'static str> {
        if self.overflowed {
            Some("Counter overflow: this capture is invalid")
        } else if self.overlap_presses != 0 {
            Some("Overlapping button presses were observed")
        } else if self.buttons.iter().any(|button| button.anomalies != 0) {
            Some("Unmatched or repeated button events were observed")
        } else if self.buttons[3..]
            .iter()
            .any(|button| button.presses != 0 || button.releases != 0)
        {
            Some("Extra-button input was observed")
        } else {
            None
        }
    }
    pub fn counts(&self) -> &[ButtonStats; 5] {
        &self.buttons
    }
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.active {
            return Err("A capture session is already active");
        }
        if self.buttons.iter().any(|button| button.held) {
            return Err("Release all mouse buttons before starting a session");
        }
        self.id = self
            .id
            .checked_add(1)
            .ok_or("Capture session identity exhausted")?;
        self.buttons = [ButtonStats::default(); 5];
        self.overlap_presses = 0;
        self.overflowed = false;
        self.active = true;
        Ok(())
    }
    pub fn push(&mut self, button: egui::PointerButton, pressed: bool) {
        let overlap = pressed && self.buttons.iter().any(|button| button.held);
        let button = &mut self.buttons[button as usize];
        if self.active {
            if overlap {
                add(&mut self.overlap_presses, &mut self.overflowed);
            }
            if pressed {
                add(&mut button.presses, &mut self.overflowed);
                if button.held {
                    add(&mut button.anomalies, &mut self.overflowed);
                }
            } else {
                add(&mut button.releases, &mut self.overflowed);
                if button.held {
                    add(&mut button.pairs, &mut self.overflowed);
                } else {
                    add(&mut button.anomalies, &mut self.overflowed);
                }
            }
        }
        button.held = pressed;
    }
    pub fn stop(&mut self) -> Option<Report> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(Report {
            session_id: self.id,
            buttons: self.buttons,
            overlap_presses: self.overlap_presses,
            overflowed: self.overflowed,
            balanced: !self.overflowed
                && self.overlap_presses == 0
                && self.buttons.iter().all(|b| {
                    !b.held && b.anomalies == 0 && b.presses == b.releases && b.presses == b.pairs
                }),
            unexpected_buttons: self.buttons[3..]
                .iter()
                .any(|b| b.presses != 0 || b.releases != 0),
        })
    }
}
fn add(value: &mut u64, overflowed: &mut bool) {
    match value.checked_add(1) {
        Some(next) => *value = next,
        None => *overflowed = true,
    }
}
pub fn create(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}
pub fn write_record(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}
#[cfg(test)]
mod tests {
    use super::*;
    use egui::PointerButton::{Middle, Primary, Secondary};
    type Result = std::result::Result<(), Box<dyn std::error::Error>>;
    #[test]
    fn counts_only_armed_events_and_distinguishes_buttons() -> Result {
        let mut capture = Capture::default();
        capture.push(Primary, true);
        capture.push(Primary, false);
        capture.start()?;
        for button in [Primary, Secondary, Middle] {
            capture.push(button, true);
            capture.push(button, false);
        }
        let report = capture.stop().ok_or("Missing report")?;
        assert!(
            report.balanced
                && report.buttons[..3].iter().all(|b| b.pairs == 1)
                && report.buttons[3..].iter().all(|b| b.pairs == 0)
        );
        Ok(())
    }
    #[test]
    fn duplicate_press_and_unmatched_release_are_reported() -> Result {
        let mut capture = Capture::default();
        capture.start()?;
        capture.push(Primary, false);
        capture.push(Secondary, true);
        capture.push(Secondary, true);
        capture.push(Secondary, false);
        let report = capture.stop().ok_or("Missing report")?;
        assert!(
            !report.balanced
                && report.buttons[0].anomalies == 1
                && report.buttons[1].anomalies == 1
                && report.overlap_presses == 1
        );
        Ok(())
    }
    #[test]
    fn held_button_prevents_arming() {
        let mut capture = Capture::default();
        capture.push(Primary, true);
        assert!(capture.start().is_err());
    }
    #[test]
    fn stop_preserves_unreleased_state_and_next_session_starts_fresh() -> Result {
        let mut capture = Capture::default();
        capture.start()?;
        capture.push(Primary, true);
        let first = capture.stop().ok_or("Missing report")?;
        capture.push(Primary, false);
        capture.start()?;
        capture.push(Secondary, true);
        capture.push(Secondary, false);
        let second = capture.stop().ok_or("Missing report")?;
        assert!(
            !first.balanced
                && first.buttons[0].held
                && second.balanced
                && second.session_id == 2
                && second.buttons[0].presses == 0
                && second.buttons[1].pairs == 1
        );
        Ok(())
    }
    #[test]
    fn overlapping_different_buttons_are_not_balanced() -> Result {
        let mut capture = Capture::default();
        capture.start()?;
        capture.push(Primary, true);
        capture.push(Secondary, true);
        capture.push(Primary, false);
        capture.push(Secondary, false);
        let report = capture.stop().ok_or("Missing report")?;
        assert!(
            !report.balanced
                && report.overlap_presses == 1
                && report.buttons[0].pairs == 1
                && report.buttons[1].pairs == 1
        );
        Ok(())
    }
    #[test]
    fn extra_buttons_remain_distinct_from_supported_buttons() -> Result {
        let mut capture = Capture::default();
        capture.start()?;
        capture.push(egui::PointerButton::Extra1, true);
        capture.push(egui::PointerButton::Extra1, false);
        let report = capture.stop().ok_or("Missing report")?;
        assert!(
            report.unexpected_buttons
                && report.buttons[3].pairs == 1
                && report.buttons[..3].iter().all(|button| button.pairs == 0)
        );
        Ok(())
    }
    #[test]
    fn counter_overflow_is_visible() -> Result {
        let mut capture = Capture::default();
        capture.start()?;
        capture.buttons[0].presses = u64::MAX;
        capture.push(Primary, true);
        assert!(capture.stop().ok_or("Missing report")?.overflowed);
        Ok(())
    }
    #[test]
    fn output_never_overwrites_and_has_private_unix_permissions() -> Result {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("capture.jsonl");
        let mut file = create(&path)?;
        file.write_all(b"original")?;
        assert!(
            matches!(create(&path), Err(error) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        assert_eq!(std::fs::read(&path)?, b"original");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(path)?.permissions().mode() & 0o777, 0o600);
        }
        Ok(())
    }
    #[test]
    fn output_failure_is_returned() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("Fixture failure"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert!(write_record(&mut Broken, &serde_json::json!({"record_type":"test"})).is_err());
    }
}
