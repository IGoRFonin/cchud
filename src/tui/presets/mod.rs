//! Presets — built-in + user-saved snapshots of (lines + theme).
//!
//! Built-in: 5 шт., embedded через `include_str!` (см. `builtins.rs`).
//! User-saved: `~/.config/cchud/presets/*.json` (формат — `PresetData`).

#![deny(clippy::unwrap_used, clippy::expect_used)]
// list_all/apply/save_as wired in T5/T7; suppress for the staging build.
#![allow(dead_code)]

mod builtins;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::tui::app::App;
use crate::types::config::{Line, Settings, ThemeConfig};

#[derive(Debug, Clone)]
pub enum PresetSource {
    Builtin,
    UserSaved(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub source: PresetSource,
    pub data: PresetData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    #[serde(default)]
    pub lines: Vec<Line>,
    #[serde(default)]
    pub theme: ThemeConfig,
}

/// Список built-in + scan user-dir. Невалидные user-файлы — silently skipped.
#[must_use]
pub fn list_all() -> Vec<Preset> {
    let mut out: Vec<Preset> = builtins::ALL
        .iter()
        .map(|(name, data)| Preset {
            name: (*name).to_string(),
            source: PresetSource::Builtin,
            data: data.clone(),
        })
        .collect();

    if let Some(dir) = user_presets_dir() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let Ok(bytes) = fs::read(&path) else { continue };
                let Ok(data) = serde_json::from_slice::<PresetData>(&bytes) else {
                    continue;
                };
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
                    .unwrap_or_default();
                if !name.is_empty() {
                    out.push(Preset {
                        name,
                        source: PresetSource::UserSaved(path),
                        data,
                    });
                }
            }
        }
    }
    out
}

/// Apply preset — only lines + theme; всё остальное в Settings сохраняется.
pub fn apply(app: &mut App, p: &Preset) {
    p.data.lines.clone_into(&mut app.editable.lines);
    p.data.theme.clone_into(&mut app.editable.theme);
    app.selected_line = 0;
    app.selected_widget = app
        .editable
        .lines
        .first()
        .and_then(|l| if l.widgets.is_empty() { None } else { Some(0) });
}

/// Save current settings as a user-preset. Sanitizes name; rejects empty.
///
/// # Errors
/// `io::Error` если sanitize ничего не оставил, или IO упал.
pub fn save_as(name: &str, settings: &Settings) -> io::Result<PathBuf> {
    let sanitized = sanitize_name(name);
    if sanitized.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "preset name must contain at least one [a-zA-Z0-9_-] character",
        ));
    }
    let dir = user_presets_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cannot resolve presets dir (HOME unset?)",
        )
    })?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{sanitized}.json"));

    let data = PresetData {
        lines: settings.lines.clone(),
        theme: settings.theme.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut tmp = tempfile::NamedTempFile::new_in(&dir)?;
    tmp.write_all(&bytes)?;
    tmp.flush()?;
    tmp.persist(&path).map_err(|e| e.error)?;
    Ok(path)
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

fn user_presets_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CCHUD_PRESETS_DIR") {
        return Some(PathBuf::from(p));
    }
    dirs::home_dir().map(|h| h.join(".config/cchud/presets"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::Path;
    use tempfile::TempDir;

    fn override_presets_dir(tmp: &Path) {
        // SAFETY: serial_test gates these against parallel access to env.
        unsafe {
            std::env::set_var("CCHUD_PRESETS_DIR", tmp);
        }
    }

    #[test]
    fn builtin_list_has_5_entries() {
        let n = builtins::ALL.len();
        assert_eq!(n, 5);
    }

    #[test]
    fn builtins_all_parse_without_panic() {
        for (name, data) in builtins::ALL.iter() {
            assert!(!data.lines.is_empty(), "{name} has empty lines");
        }
    }

    #[test]
    #[serial]
    fn list_all_returns_only_builtins_when_user_dir_empty() {
        let tmp = TempDir::new().unwrap();
        override_presets_dir(tmp.path());
        let presets = list_all();
        assert_eq!(presets.len(), 5);
        assert!(presets.iter().all(|p| matches!(p.source, PresetSource::Builtin)));
    }

    #[test]
    #[serial]
    fn list_all_includes_user_saved() {
        let tmp = TempDir::new().unwrap();
        override_presets_dir(tmp.path());
        let p = tmp.path().join("my-laptop.json");
        std::fs::write(&p, r#"{"lines":[],"theme":{"kind":"plain"}}"#).unwrap();
        let presets = list_all();
        assert_eq!(presets.len(), 6);
        let user = presets.iter().find(|x| x.name == "my-laptop").unwrap();
        assert!(matches!(user.source, PresetSource::UserSaved(_)));
    }

    #[test]
    #[serial]
    fn list_all_skips_invalid_user_files() {
        let tmp = TempDir::new().unwrap();
        override_presets_dir(tmp.path());
        std::fs::write(tmp.path().join("bad.json"), b"not json").unwrap();
        let presets = list_all();
        assert_eq!(presets.len(), 5);
    }

    #[test]
    #[serial]
    fn apply_replaces_lines_and_theme_only() {
        use crate::tui::sample;
        let (payload, fixture) = sample::payload();
        let mut app = App::new(Settings::default(), payload, fixture);
        let preset = list_all().into_iter().find(|p| p.name == "minimal").unwrap();
        apply(&mut app, &preset);
        assert!(!app.editable.lines.is_empty());
        assert_eq!(app.editable.version, Settings::default().version);
    }

    #[test]
    #[serial]
    fn save_as_writes_json_with_lines_and_theme() {
        let tmp = TempDir::new().unwrap();
        override_presets_dir(tmp.path());
        let mut s = Settings::default();
        s.lines.push(Line::default());
        let path = save_as("my preset!", &s).unwrap();
        assert!(path.ends_with("mypreset.json"));
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: PresetData = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.lines.len(), 1);
    }

    #[test]
    #[serial]
    fn save_as_rejects_empty_after_sanitize() {
        let tmp = TempDir::new().unwrap();
        override_presets_dir(tmp.path());
        let s = Settings::default();
        let err = save_as("***", &s).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
