//! User config schema — parsed from `~/.config/cchud/settings.json`.
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
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WidgetConfig {
    // Phase 2:
    Model {
        #[serde(flatten, default)]
        params: ModelParams,
    },

    // Phase 3 — без параметров:
    Version,
    ClaudeSessionId,
    TerminalWidth,
    OutputStyle,
    VimMode,
    SessionName,
    SessionClock,
    SessionCost,
    ContextLength,
    ContextPercentage,
    ContextPercentageUsable,
    TokensInput,
    TokensOutput,
    Worktree,
    WorktreeMode,
    WorktreeName,
    WorktreeBranch,
    WorktreeOriginalBranch,

    // Phase 3 — с параметрами:
    CustomText {
        #[serde(flatten)]
        params: CustomTextParams,
    },
    CustomSymbol {
        #[serde(flatten)]
        params: CustomSymbolParams,
    },
    Link {
        #[serde(flatten)]
        params: LinkParams,
    },
    CustomCommand {
        #[serde(flatten)]
        params: CustomCommandParams,
    },
    ContextBar {
        #[serde(flatten, default)]
        params: ContextBarParams,
    },
}

/// Per-widget parameters. Phase 7 adds custom format strings, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTextParams {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSymbolParams {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkParams {
    pub url: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommandParams {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_command_timeout_ms")]
    pub timeout_ms: u64,
}
const fn default_command_timeout_ms() -> u64 {
    200
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextBarParams {
    #[serde(default = "default_context_bar_width")]
    pub width: u32,
}
const fn default_context_bar_width() -> u32 {
    10
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub kind: ThemeKind,
    /// Built-in theme name. Ignored when `kind != Powerline` or `custom` is set.
    #[serde(default)]
    pub theme_name: Option<String>,
    /// Full custom theme (overrides built-ins). Powerline only.
    #[serde(default)]
    pub custom: Option<crate::render::themes::PowerlineTheme>,
    /// Override separator glyphs (Powerline). First element = primary separator.
    #[serde(default)]
    pub separators: Vec<String>,
    #[serde(default)]
    pub start_caps: Vec<String>,
    #[serde(default)]
    pub end_caps: Vec<String>,
    /// `None` means auto-detect at runtime.
    #[serde(default)]
    pub color_level: Option<crate::render::ColorLevel>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeKind {
    #[default]
    Plain,
    Powerline,
}

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
            "lines": [{"widgets": [{"type": "model"}]}],
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
            "lines": [{"widgets": [{"type": "branch"}]}]
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

    #[test]
    fn parses_phase3_widget_kinds() {
        let json = r#"{
            "lines": [{"widgets": [
                {"type": "version"},
                {"type": "claude-session-id"},
                {"type": "context-bar", "width": 20},
                {"type": "custom-text", "text": "hello"},
                {"type": "custom-symbol", "symbol": "★"},
                {"type": "link", "url": "https://x.com", "label": "X"},
                {"type": "custom-command", "command": "echo", "args": ["hi"]}
            ]}]
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.lines[0].widgets.len(), 7);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Version));
        assert!(matches!(
            s.lines[0].widgets[1],
            WidgetConfig::ClaudeSessionId
        ));
        match &s.lines[0].widgets[2] {
            WidgetConfig::ContextBar { params } => assert_eq!(params.width, 20),
            other => panic!("expected ContextBar, got {other:?}"),
        }
        match &s.lines[0].widgets[3] {
            WidgetConfig::CustomText { params } => assert_eq!(params.text, "hello"),
            other => panic!("expected CustomText, got {other:?}"),
        }
        match &s.lines[0].widgets[6] {
            WidgetConfig::CustomCommand { params } => {
                assert_eq!(params.command, "echo");
                assert_eq!(params.args, vec!["hi".to_string()]);
                assert_eq!(params.timeout_ms, 200, "default timeout_ms = 200");
            }
            other => panic!("expected CustomCommand, got {other:?}"),
        }
    }

    #[test]
    fn context_bar_default_width_is_ten() {
        let json = r#"{"lines":[{"widgets":[{"type":"context-bar"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        match &s.lines[0].widgets[0] {
            WidgetConfig::ContextBar { params } => assert_eq!(params.width, 10),
            other => panic!("expected ContextBar, got {other:?}"),
        }
    }

    #[test]
    fn theme_config_default_is_plain_kind() {
        let s = Settings::default();
        assert_eq!(s.theme.kind, ThemeKind::Plain);
        assert!(s.theme.custom.is_none());
        assert!(s.theme.color_level.is_none());
    }

    #[test]
    fn theme_config_parses_powerline_with_name() {
        let json = r#"{
            "lines": [],
            "theme": {"kind": "powerline", "theme_name": "dracula"}
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.theme.kind, ThemeKind::Powerline);
        assert_eq!(s.theme.theme_name.as_deref(), Some("dracula"));
    }

    #[test]
    fn theme_config_rejects_unknown_kind() {
        let json = r#"{"theme": {"kind": "rainbow"}}"#;
        assert!(serde_json::from_str::<Settings>(json).is_err());
    }

    #[test]
    fn theme_config_parses_color_level_override() {
        let json = r#"{"theme": {"color_level": "true-color"}}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(
            s.theme.color_level,
            Some(crate::render::ColorLevel::TrueColor)
        );
    }
}
