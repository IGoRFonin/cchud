//! User config schema — parsed from `~/.claude/settings.json` `cchud` block.
//!
//! Phase 2 supports only `WidgetConfig::Model`. Phase 3 adds 9 more variants.
//! `ThemeConfig` is an empty slot — populated in Phase 4 (Powerline colors,
//! separator overrides). Migrations infrastructure intentionally omitted —
//! current `version: 1` is the only version that exists.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub lines: Vec<Line>,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    #[serde(default)]
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetConfig {
    Model {
        #[serde(flatten, default)]
        params: ModelParams,
    },
}

/// Per-widget parameters. Phase 7 adds custom format strings, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelParams {}

/// Empty in Phase 2. Phase 4 will populate (`powerline_colors`, `separator_override`, ...).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {}

const fn default_version() -> u32 {
    1
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: default_version(),
            lines: Vec::new(),
            theme: ThemeConfig::default(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_cchud_block() {
        let json = r#"{
            "version": 1,
            "lines": [{"widgets": [{"type": "Model"}]}],
            "theme": {}
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(s.lines.len(), 1);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Model { .. }));
    }

    #[test]
    fn defaults_fill_missing_fields() {
        let json = r#"{}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.version, 1);
        assert!(s.lines.is_empty());
    }

    #[test]
    fn rejects_unknown_widget_type() {
        let json = r#"{
            "lines": [{"widgets": [{"type": "Branch"}]}]
        }"#;
        // Phase 2 не знает про Branch — должен упасть на парсинге.
        // Phase 3 добавит вариант, тест обновится.
        assert!(serde_json::from_str::<Settings>(json).is_err());
    }

    #[test]
    fn serde_roundtrip_preserves_shape() {
        let original = Settings {
            version: 1,
            lines: vec![Line {
                widgets: vec![WidgetConfig::Model {
                    params: ModelParams::default(),
                }],
            }],
            theme: ThemeConfig::default(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.lines.len(), 1);
        assert!(matches!(
            back.lines[0].widgets[0],
            WidgetConfig::Model { .. }
        ));
    }
}
