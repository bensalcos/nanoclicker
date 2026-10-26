use crate::controller::Draft;
use eframe::egui;
use nanoclick_core::{ClickConfig, MouseButton};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedRepeatMode {
    UntilStopped,
    Count,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreferencesV1 {
    version: u32,
    interval_ms: u64,
    button: SavedButton,
    count: Option<u64>,
    theme: SavedTheme,
    shortcut: String,
}
impl From<PreferencesV1> for Preferences {
    fn from(old: PreferencesV1) -> Self {
        debug_assert_eq!(old.version, 1);
        Self {
            version: 2,
            interval_ms: old.interval_ms,
            button: old.button,
            count: old.count.unwrap_or(10),
            repeat_mode: if old.count.is_some() {
                SavedRepeatMode::Count
            } else {
                SavedRepeatMode::UntilStopped
            },
            theme: old.theme,
            shortcut: old.shortcut,
        }
    }
}

/// Only preferences are serializable. Run state and granted permissions are absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub version: u32,
    pub interval_ms: u64,
    pub button: SavedButton,
    pub count: u64,
    pub repeat_mode: SavedRepeatMode,
    pub theme: SavedTheme,
    pub shortcut: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 2,
            interval_ms: 100,
            button: SavedButton::Left,
            count: 10,
            repeat_mode: SavedRepeatMode::UntilStopped,
            theme: SavedTheme::System,
            shortcut: "F6".into(),
        }
    }
}
impl Preferences {
    fn validate(&self) -> Result<(), SettingsError> {
        if self.version != 2 {
            return Err(SettingsError::UnsupportedVersion(self.version));
        }
        // Preserve and validate the remembered finite count in both modes.
        ClickConfig::new(self.interval_ms, self.button.into(), Some(self.count))
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
            count: self.count.to_string(),
            finite: self.repeat_mode == SavedRepeatMode::Count,
            button: self.button.into(),
        }
    }
    pub fn from_draft(
        draft: &Draft,
        theme: egui::ThemePreference,
        shortcut: &str,
    ) -> Result<Self, SettingsError> {
        let plan = draft.validate().map_err(SettingsError::Invalid)?;
        let count = draft.count.parse::<u64>().map_err(|_| {
            SettingsError::Invalid("Enter a count from 1 to 999999 clicks before saving.".into())
        })?;
        let preferences = Self {
            version: 2,
            interval_ms: plan.interval().as_millis() as u64,
            button: plan.button().into(),
            count,
            repeat_mode: if draft.finite {
                SavedRepeatMode::Count
            } else {
                SavedRepeatMode::UntilStopped
            },
            theme: theme.into(),
            shortcut: shortcut.into(),
        };
        preferences.validate()?;
        Ok(preferences)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Preferences version {0} is not supported. The file was preserved.")]
    UnsupportedVersion(u32),
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
    recovery_available: bool,
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
            recovery_available: false,
        }
    }
    pub fn can_replace(&self) -> bool {
        self.can_replace
    }
    pub fn can_recover(&self) -> bool {
        self.recovery_available
    }
    pub fn load(&mut self) -> Result<Preferences, SettingsError> {
        self.can_replace = false;
        self.recovery_available = false;
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // Windows can report PATH_NOT_FOUND when an existing parent is
                // a regular file. That is a blocked location, not a first run.
                for parent in self.path.ancestors().skip(1) {
                    match fs::metadata(parent) {
                        Ok(metadata) if metadata.is_dir() => break,
                        Ok(_) => return Err(SettingsError::Invalid(
                            "A parent of the preferences path is not a directory. The original path was preserved.".into(),
                        )),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                }
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
        #[derive(Deserialize)]
        struct Version {
            version: u32,
        }
        let version: Version = match serde_json::from_slice(&bytes) {
            Ok(version) => version,
            Err(error) => {
                self.recovery_available = true;
                return Err(error.into());
            }
        };
        if !matches!(version.version, 1 | 2) {
            return Err(SettingsError::UnsupportedVersion(version.version));
        }
        let decoded = match version.version {
            1 => serde_json::from_slice::<PreferencesV1>(&bytes).map(Preferences::from),
            _ => serde_json::from_slice::<Preferences>(&bytes),
        };
        let preferences = match decoded {
            Ok(preferences) => preferences,
            Err(error) => {
                self.recovery_available = true;
                return Err(error.into());
            }
        };
        if let Err(error) = preferences.validate() {
            self.recovery_available = true;
            return Err(error);
        }
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

    /// Preserve a rejected preferences file before explicitly replacing it.
    /// The backup and replacement are kept in the same directory so the final
    /// save remains an atomic rename on the destination filesystem.
    pub fn recover_and_save(
        &mut self,
        preferences: &Preferences,
    ) -> Result<PathBuf, SettingsError> {
        preferences.validate()?;
        if self.can_replace {
            return Err(SettingsError::Invalid(
                "The preferences file does not need recovery.".into(),
            ));
        }
        if !self.recovery_available {
            return Err(SettingsError::Protected);
        }

        let parent = self.path.parent().ok_or_else(|| {
            SettingsError::Invalid("No preferences directory is available.".into())
        })?;
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| SettingsError::Invalid("The preferences filename is invalid.".into()))?;

        // Match the load limit and bound disk and memory work for a file that
        // cannot be parsed. Larger files remain untouched for manual recovery.
        let source = File::open(&self.path)?;
        let mut bytes = Vec::new();
        source.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            self.recovery_available = false;
            return Err(SettingsError::Invalid(
                "The preferences file exceeds 64 KiB and was left untouched. Move it aside manually before saving new preferences.".into(),
            ));
        }

        let mut backup = tempfile::NamedTempFile::new_in(parent)?;
        backup.write_all(&bytes)?;
        backup.as_file().sync_all()?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut backup_path = None;
        for attempt in 0_u32..100 {
            let candidate = parent.join(format!("{file_name}.invalid-{timestamp}-{attempt}.bak"));
            match backup.persist_noclobber(&candidate) {
                Ok(_) => {
                    backup_path = Some(candidate);
                    break;
                }
                Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                    backup = error.file;
                }
                Err(error) => return Err(error.error.into()),
            }
        }
        let backup_path = backup_path.ok_or_else(|| {
            SettingsError::Invalid("Could not create a unique preferences backup.".into())
        })?;

        self.can_replace = true;
        self.recovery_available = false;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;

        self.save(preferences)?;
        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn until_stopped_preserves_the_remembered_count() -> Result<(), Box<dyn std::error::Error>> {
        let draft = Draft {
            count: "500".into(),
            finite: false,
            ..Default::default()
        };
        let preferences = Preferences::from_draft(&draft, egui::ThemePreference::Dark, "F6")?;
        let restored: Preferences = serde_json::from_str(&serde_json::to_string(&preferences)?)?;
        assert_eq!(restored.draft().count, "500");
        assert!(!restored.draft().finite);
        Ok(())
    }
    #[test]
    fn version_one_migrates_in_memory_without_rewriting_the_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("preferences.json");
        for (count, expected, finite) in [("null", "10", false), ("500", "500", true)] {
            let text = format!(
                r#"{{"version":1,"interval_ms":37,"button":"middle","count":{count},"theme":"dark","shortcut":"F6"}}"#
            );
            fs::write(&path, &text)?;
            let mut file = PreferencesFile::new(path.clone());
            let migrated = file.load()?;
            assert_eq!(migrated.version, 2);
            assert_eq!(migrated.draft().count, expected);
            assert_eq!(migrated.draft().finite, finite);
            assert_eq!(fs::read_to_string(&path)?, text);
            file.save(&migrated)?;
            assert_eq!(file.load()?, migrated);
        }
        Ok(())
    }
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
        for bytes in [
            b"broken".as_slice(),
            br#"{"version":99,"future_field":true}"#.as_slice(),
        ] {
            std::fs::write(&path, bytes)?;
            let mut file = PreferencesFile::new(path.clone());
            assert!(file.load().is_err());
            assert_eq!(file.can_recover(), bytes == b"broken");
            assert!(file.save(&Preferences::default()).is_err());
            assert_eq!(std::fs::read(&path)?, bytes);
        }
        Ok(())
    }
    #[test]
    fn future_schema_is_protected_before_parsing_current_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("preferences.json");
        let bytes = br#"{"version":99,"interval_ms":"a future representation","extra":true}"#;
        std::fs::write(&path, bytes)?;
        let mut file = PreferencesFile::new(path.clone());
        assert!(matches!(
            file.load(),
            Err(SettingsError::UnsupportedVersion(99))
        ));
        assert!(!file.can_recover());
        assert!(matches!(
            file.recover_and_save(&Preferences::default()),
            Err(SettingsError::Protected)
        ));
        assert_eq!(std::fs::read(path)?, bytes);
        Ok(())
    }
    #[test]
    fn unknown_fields_in_current_schema_still_allow_explicit_recovery()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("preferences.json");
        let text = serde_json::to_string(&Preferences::default())?;
        std::fs::write(&path, text.replacen('{', "{\"unexpected\":true,", 1))?;
        let mut file = PreferencesFile::new(path);
        assert!(matches!(file.load(), Err(SettingsError::Json(_))));
        assert!(file.can_recover());
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
        assert!(!file.can_recover());
        assert!(matches!(
            file.save(&Preferences::default()),
            Err(SettingsError::Protected)
        ));
        assert!(matches!(
            file.recover_and_save(&Preferences::default()),
            Err(SettingsError::Protected)
        ));
        assert_eq!(std::fs::read(path)?, b"keep me");
        Ok(())
    }
    #[test]
    fn missing_nested_directories_allow_first_save() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir
            .path()
            .join("missing")
            .join("nested")
            .join("preferences.json");
        let mut file = PreferencesFile::new(path.clone());
        let mut preferences = file.load()?;
        assert!(!path.exists());
        preferences.interval_ms = 25;
        file.save(&preferences)?;
        assert_eq!(file.load()?.interval_ms, 25);
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
            count: 0,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }
}
