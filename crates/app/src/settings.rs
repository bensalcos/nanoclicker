use crate::controller::Draft;
use eframe::egui;
use nanoclick_core::{ClickConfig, MouseButton, StopCondition};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SavedTheme {
    #[default]
    System,
    Light,
    Dark,
}
impl From<SavedTheme> for egui::ThemePreference {
    fn from(value: SavedTheme) -> Self {
        match value {
            SavedTheme::System => Self::System,
            SavedTheme::Light => Self::Light,
            SavedTheme::Dark => Self::Dark,
        }
    }
}
impl From<egui::ThemePreference> for SavedTheme {
    fn from(value: egui::ThemePreference) -> Self {
        match value {
            egui::ThemePreference::System => Self::System,
            egui::ThemePreference::Light => Self::Light,
            egui::ThemePreference::Dark => Self::Dark,
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SavedButton {
    #[default]
    Left,
    Right,
    Middle,
}
impl From<SavedButton> for MouseButton {
    fn from(value: SavedButton) -> Self {
        match value {
            SavedButton::Left => Self::Left,
            SavedButton::Right => Self::Right,
            SavedButton::Middle => Self::Middle,
        }
    }
}
impl From<MouseButton> for SavedButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
        }
    }
}

/// Only preferences are serializable. Run state and granted permissions are absent.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub version: u32,
    pub interval_ms: u64,
    pub button: SavedButton,
    pub count: Option<u64>,
    pub theme: SavedTheme,
    pub shortcut: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            interval_ms: 100,
            button: SavedButton::Left,
            count: None,
            theme: SavedTheme::System,
            shortcut: "F6".into(),
        }
    }
}
impl Preferences {
    fn validate(&self) -> Result<(), SettingsError> {
        if self.version != 1 {
            return Err(SettingsError::Invalid(format!(
                "Preferences version {} is not supported. The file was preserved.",
                self.version
            )));
        }
        ClickConfig::new(self.interval_ms, self.button.into(), self.count)
            .map_err(|e| SettingsError::Invalid(e.to_string()))?;
        if self.shortcut.len() > 128 {
            return Err(SettingsError::Invalid(
                "The saved shortcut is too long.".into(),
            ));
        }
        self.shortcut
            .parse::<global_hotkey::hotkey::HotKey>()
            .map_err(|e| SettingsError::Invalid(e.to_string()))?;
        Ok(())
    }
    pub fn draft(&self) -> Draft {
        Draft {
            interval: self.interval_ms.to_string(),
            count: self.count.unwrap_or(10).to_string(),
            finite: self.count.is_some(),
            button: self.button.into(),
        }
    }
    pub fn from_draft(
        draft: &Draft,
        theme: egui::ThemePreference,
        shortcut: &str,
    ) -> Result<Self, SettingsError> {
        let plan = draft.validate().map_err(SettingsError::Invalid)?;
        let count = match plan.stop_condition() {
            StopCondition::UntilStopped => None,
            StopCondition::Count(count) => Some(count.get()),
        };
        let preferences = Self {
            version: 1,
            interval_ms: plan.interval().as_millis() as u64,
            button: plan.button().into(),
            count,
            theme: theme.into(),
            shortcut: shortcut.into(),
        };
        preferences.validate()?;
        Ok(preferences)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Preferences could not be read or saved: {0}")]
    Io(#[from] std::io::Error),
    #[error("Preferences are invalid; the original file was preserved: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
    #[error(
        "The original preferences file is protected because it could not be loaded. Move it aside and restart NanoClick before saving new preferences."
    )]
    Protected,
}

pub struct PreferencesFile {
    path: PathBuf,
    can_replace: bool,
}
impl PreferencesFile {
    pub fn discover() -> Result<Self, SettingsError> {
        let dirs = directories::ProjectDirs::from("io.github", "bensalcos", "NanoClick")
            .ok_or_else(|| {
                SettingsError::Invalid("No configuration directory is available.".into())
            })?;
        Ok(Self::new(dirs.config_dir().join("preferences.json")))
    }
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            can_replace: false,
        }
    }
    pub fn load(&mut self) -> Result<Preferences, SettingsError> {
        self.can_replace = false;
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.can_replace = true;
                return Ok(Preferences::default());
            }
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::new();
        file.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            return Err(SettingsError::Invalid(
                "The preferences file exceeds 64 KiB. It was preserved.".into(),
            ));
        }
        let preferences: Preferences = serde_json::from_slice(&bytes)?;
        preferences.validate()?;
        self.can_replace = true;
        Ok(preferences)
    }
    pub fn save(&self, preferences: &Preferences) -> Result<(), SettingsError> {
        if !self.can_replace {
            return Err(SettingsError::Protected);
        }
        preferences.validate()?;
        let parent = self.path.parent().ok_or_else(|| {
            SettingsError::Invalid("No preferences directory is available.".into())
        })?;
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut temporary, preferences)?;
        temporary.write_all(b"\n")?;
        temporary.as_file().sync_all()?;
        // tempfile::persist atomically replaces the destination; errors never
        // cause us to delete the existing destination. Both paths share a directory.
        temporary.persist(&self.path).map_err(|error| error.error)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_replaces_only_valid_preferences() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let mut file = PreferencesFile::new(dir.path().join("preferences.json"));
        let mut preferences = file.load()?;
        file.save(&preferences)?;
        preferences.interval_ms = 25;
        file.save(&preferences)?;
        assert_eq!(file.load()?.interval_ms, 25);
        let before = std::fs::read(&file.path)?;
        preferences.interval_ms = 0;
        assert!(file.save(&preferences).is_err());
        assert_eq!(std::fs::read(&file.path)?, before);
        Ok(())
    }
    #[test]
    fn corrupt_and_future_files_are_preserved() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("preferences.json");
        for bytes in [b"broken".as_slice(), br#"{"version":99,"interval_ms":100,"button":"left","count":null,"theme":"system","shortcut":"F6"}"#.as_slice()] {
            std::fs::write(&path, bytes)?;
            let mut file = PreferencesFile::new(path.clone());
            assert!(file.load().is_err());
            assert!(file.save(&Preferences::default()).is_err());
            assert_eq!(std::fs::read(&path)?, bytes);
        }
        Ok(())
    }
    #[test]
    fn serialized_data_never_contains_active_session_state()
    -> Result<(), Box<dyn std::error::Error>> {
        let text = serde_json::to_string(&Preferences::default())?;
        assert!(!text.contains("running"));
        assert!(!text.contains("run_id"));
        let injected = text.replacen('{', "{\"running\":true,", 1);
        assert!(serde_json::from_str::<Preferences>(&injected).is_err());
        Ok(())
    }
    #[test]
    fn failed_publication_preserves_existing_path() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("blocked");
        std::fs::write(&path, b"keep me")?;
        let mut file = PreferencesFile::new(path.join("preferences.json"));
        assert!(file.load().is_err());
        assert!(file.save(&Preferences::default()).is_err());
        assert_eq!(std::fs::read(path)?, b"keep me");
        Ok(())
    }
    #[test]
    fn failed_atomic_replace_leaves_destination_untouched() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("preferences.json");
        let mut file = PreferencesFile::new(destination.clone());
        file.load()?;
        std::fs::create_dir(&destination)?;
        std::fs::write(destination.join("sentinel"), b"preserve")?;
        assert!(file.save(&Preferences::default()).is_err());
        assert_eq!(std::fs::read(destination.join("sentinel"))?, b"preserve");
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 1);
        Ok(())
    }
    #[test]
    fn oversized_and_invalid_values_are_rejected_without_overwriting()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("preferences.json");
        std::fs::write(&path, vec![b' '; 65_537])?;
        let mut file = PreferencesFile::new(path.clone());
        assert!(file.load().is_err());
        assert_eq!(std::fs::metadata(path)?.len(), 65_537);
        let invalid = Preferences {
            count: Some(0),
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }
}
