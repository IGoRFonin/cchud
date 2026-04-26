//! Config loader — reads `~/.claude/settings.json`, extracts the `cchud`
//! block, gracefully falls back to default-line on any error (AC-007 spirit).
//!
//! Never panics. Never propagates errors to caller — caller gets `Settings`.
//! Diagnostic warnings go to stderr with `cchud:` prefix.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use serde::Deserialize;

use crate::types::config::{Line, ModelParams, Settings, ThemeConfig, WidgetConfig};

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
struct RootSettings {
    #[serde(default)]
    cchud: Option<Settings>,
}

#[must_use]
pub fn load() -> Settings {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return default_line();
    };
    match serde_json::from_str::<RootSettings>(&content) {
        Ok(root) => root.cchud.unwrap_or_else(default_line),
        Err(e) => {
            eprintln!("cchud: invalid cchud config block, using defaults: {e}");
            default_line()
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/settings.json")
}

#[must_use]
pub fn default_line() -> Settings {
    Settings {
        version: 1,
        lines: vec![Line {
            widgets: vec![WidgetConfig::Model {
                params: ModelParams::default(),
            }],
        }],
        theme: ThemeConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn default_line_has_one_model_widget() {
        let s = default_line();
        assert_eq!(s.lines.len(), 1);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Model { .. }));
    }

    #[test]
    fn default_line_version_is_one() {
        assert_eq!(default_line().version, 1);
    }

    #[test]
    fn root_settings_parses_with_cchud_block() {
        let json = r#"{"cchud":{"version":1,"lines":[{"widgets":[{"type":"Model"}]}]}}"#;
        let root: RootSettings = serde_json::from_str(json).unwrap();
        assert!(root.cchud.is_some());
        let cchud = root.cchud.unwrap();
        assert_eq!(cchud.lines.len(), 1);
    }

    #[test]
    fn root_settings_parses_without_cchud_block() {
        let json = r#"{"theme":"dark","mcpServers":{}}"#;
        let root: RootSettings = serde_json::from_str(json).unwrap();
        assert!(root.cchud.is_none());
    }
}
