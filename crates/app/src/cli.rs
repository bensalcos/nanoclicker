use eframe::egui::ThemePreference;
use std::{ffi::OsString, path::PathBuf};
pub const USAGE: &str =
    "Usage: nanoclick [--theme system|light|dark] [--diagnostics NEW_JSONL_PATH]";
#[derive(Default)]
pub struct Options {
    pub theme: Option<ThemePreference>,
    pub diagnostics: Option<PathBuf>,
    pub help: bool,
}
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Options, &'static str> {
    let mut options = Options::default();
    let mut args = arguments.into_iter();
    while let Some(argument) = args.next() {
        if argument == "--help" {
            options.help = true;
            return Ok(options);
        } else if argument == "--theme" {
            options.theme = Some(
                match args.next().as_deref().and_then(|value| value.to_str()) {
                    Some("system") => ThemePreference::System,
                    Some("light") => ThemePreference::Light,
                    Some("dark") => ThemePreference::Dark,
                    _ => return Err(USAGE),
                },
            );
        } else if argument == "--diagnostics" && options.diagnostics.is_none() {
            let path = args.next().filter(|value| !value.is_empty()).ok_or(USAGE)?;
            options.diagnostics = Some(path.into());
        } else {
            return Err(USAGE);
        }
    }
    Ok(options)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_is_opt_in_and_can_be_combined_with_theme() -> Result<(), &'static str> {
        let default = parse([])?;
        assert!(default.diagnostics.is_none());
        let selected =
            parse(["--theme", "light", "--diagnostics", "timing.jsonl"].map(OsString::from))?;
        assert_eq!(selected.theme, Some(ThemePreference::Light));
        assert_eq!(selected.diagnostics, Some(PathBuf::from("timing.jsonl")));
        assert!(parse([OsString::from("--help")])?.help);
        Ok(())
    }
    #[test]
    fn missing_unknown_and_duplicate_output_options_are_rejected() {
        for args in [
            vec!["--diagnostics"],
            vec!["--theme", "invalid"],
            vec!["--unknown"],
            vec!["--diagnostics", ""],
            vec!["--diagnostics", "one", "--diagnostics", "two"],
        ] {
            assert!(parse(args.into_iter().map(OsString::from)).is_err());
        }
    }
}
