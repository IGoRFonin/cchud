//! Config loader — reads `~/.config/cchud/settings.json`, auto-creates on
//! first run, falls back to defaults on any error.
//!
//! Never panics. Never propagates errors to caller — caller gets `Settings`.
//! Diagnostic warnings go to stderr with `cchud:` prefix.
//!
//! Override path via `CCHUD_CONFIG=/path/to/file` env var (useful for tests
//! and non-standard installs).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io;
use std::path::{Path, PathBuf};

use crate::types::config::{Line, ModelParams, Settings, ThemeConfig, WidgetConfig, WidgetItem, WidgetStyleOverride};

#[must_use]
pub fn load() -> Settings {
    let path = config_path();
    if !path.exists() {
        let defaults = default_line();
        if let Err(e) = write_defaults(&path, &defaults) {
            eprintln!("cchud: could not create config at {}: {e}", path.display());
        }
        return defaults;
    }
    match std::fs::read_to_string(&path) {
        Err(e) => {
            eprintln!("cchud: cannot read config {}: {e}", path.display());
            default_line()
        }
        Ok(content) => match serde_json::from_str::<Settings>(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cchud: invalid config, using defaults: {e}");
                default_line()
            }
        },
    }
}

fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("CCHUD_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/cchud/settings.json")
}

fn write_defaults(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    std::fs::write(path, json)
}

#[must_use]
pub fn default_line() -> Settings {
    Settings {
        version: 1,
        lines: vec![Line {
            widgets: vec![WidgetItem {
                kind: WidgetConfig::Model {
                    params: ModelParams::default(),
                },
                style: WidgetStyleOverride::default(),
            }],
        }],
        theme: ThemeConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn set_config(path: &std::path::Path) -> impl Drop {
        // SAFETY: tests using CCHUD_CONFIG must not run in parallel;
        // serial_test handles sequencing via #[serial].
        unsafe { std::env::set_var("CCHUD_CONFIG", path) };
        struct Unset;
        impl Drop for Unset {
            fn drop(&mut self) {
                // SAFETY: same serial guarantee as above.
                unsafe { std::env::remove_var("CCHUD_CONFIG") };
            }
        }
        Unset
    }

    #[test]
    #[serial_test::serial]
    fn default_line_has_one_model_widget() {
        let s = default_line();
        assert_eq!(s.lines.len(), 1);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::Model { .. }));
    }

    #[test]
    #[serial_test::serial]
    fn default_line_version_is_one() {
        assert_eq!(default_line().version, 1);
    }

    #[test]
    #[serial_test::serial]
    fn creates_default_file_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cchud/settings.json");
        let _g = set_config(&path);

        let s = load();

        assert!(path.exists(), "settings.json should be created");
        assert_eq!(s.lines.len(), 1);
        assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::Model { .. }));
        let written = fs::read_to_string(&path).unwrap();
        let reparsed: Settings = serde_json::from_str(&written).unwrap();
        assert_eq!(reparsed.version, 1);
    }

    #[test]
    #[serial_test::serial]
    fn loads_existing_valid_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let json = r#"{"version":1,"lines":[{"widgets":[{"type":"model"}]}],"theme":{}}"#;
        fs::write(&path, json).unwrap();
        let _g = set_config(&path);

        let s = load();

        assert_eq!(s.version, 1);
        assert_eq!(s.lines.len(), 1);
        assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::Model { .. }));
    }

    #[test]
    #[serial_test::serial]
    fn falls_back_to_defaults_on_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"not json at all!!!").unwrap();
        let _g = set_config(&path);

        let s = load();

        assert_eq!(s.lines.len(), 1);
        assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::Model { .. }));
    }
}
