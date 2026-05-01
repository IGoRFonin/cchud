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
    pub widgets: Vec<WidgetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetItem {
    #[serde(flatten)]
    pub kind: WidgetConfig,
    #[serde(flatten, default)]
    pub style: WidgetStyleOverride,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WidgetStyleOverride {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub bold: Option<bool>,
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

    // Phase 5 — Task 2 (head cluster):
    GitBranch,
    GitSha,
    GitRootDir,
    // Phase 5 — Task 3 (status cluster):
    GitStatus,
    GitChanges,
    GitStaged,
    GitUnstaged,
    GitUntracked,
    GitConflicts,
    // Phase 5 — Task 4 (diff stat):
    GitInsertions,
    GitDeletions,
    // Phase 5 — Task 5 (tracking):
    GitAheadBehind,
    // Phase 5 — Task 6 (remote):
    GitOriginOwner,
    GitOriginRepo,
    GitOriginOwnerRepo,
    GitUpstreamOwner,
    GitUpstreamRepo,
    GitUpstreamOwnerRepo,
    GitIsFork,
    // Phase 5 — Task 7 (PR):
    GitPr,

    // Phase 6 — Task 6 (transcript tokens cluster):
    TokensCached,
    TokensTotal,
    InputSpeed,
    OutputSpeed,
    TotalSpeed,
    // Phase 6 — Task 7 (transcript timing cluster):
    BlockTimer,
    SessionDuration,
    // Phase 6 — Task 8 (transcript meta cluster):
    ThinkingEffort,

    // Phase 7 — usage cluster (payload.rate_limits):
    SessionUsage,
    WeeklyUsage,
    BlockResetTimer,
    WeeklyResetTimer,

    // Phase 7 — env cluster:
    ClaudeAccountEmail,
    FreeMemory,

    // Phase 7 — transcript meta:
    Skills,

    // Phase 7 — sentinel for auto_align (не считается в "60 widgets"):
    AlignRight,
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

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub kind: ThemeKind,
    #[serde(default)]
    pub theme_name: Option<String>,
    #[serde(default)]
    pub custom: Option<crate::render::themes::PowerlineTheme>,
    #[serde(default)]
    pub separators: Vec<String>,
    #[serde(default)]
    pub start_caps: Vec<String>,
    #[serde(default)]
    pub end_caps: Vec<String>,
    #[serde(default)]
    pub color_level: Option<crate::render::ColorLevel>,

    // Phase 7 — global theme settings (7.0b):
    #[serde(default)]
    pub global_bold: bool,
    #[serde(default)]
    pub inherit_separator_colors: bool,
    #[serde(default)]
    pub override_background_color: Option<String>,
    #[serde(default)]
    pub override_foreground_color: Option<String>,
    #[serde(default)]
    pub minimalist_mode: bool,
    #[serde(default)]
    pub flex_mode: FlexMode,
    #[serde(default = "default_compact_threshold")]
    pub compact_threshold: u32,
    #[serde(default)]
    pub auto_align: bool,
    #[serde(default)]
    pub continue_theme_across_lines: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            kind: ThemeKind::default(),
            theme_name: None,
            custom: None,
            separators: Vec::new(),
            start_caps: Vec::new(),
            end_caps: Vec::new(),
            color_level: None,
            global_bold: false,
            inherit_separator_colors: false,
            override_background_color: None,
            override_foreground_color: None,
            minimalist_mode: false,
            flex_mode: FlexMode::Full,
            compact_threshold: 60,
            auto_align: false,
            continue_theme_across_lines: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlexMode {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "full-minus-20")]
    FullMinus20,
    #[serde(rename = "full-minus-40")]
    FullMinus40,
    #[default]
    #[serde(rename = "disabled")]
    Disabled,
}

const fn default_compact_threshold() -> u32 {
    60
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
        assert!(matches!(
            s.lines[0].widgets[0].kind,
            WidgetConfig::Model { .. }
        ));
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
                widgets: vec![WidgetItem {
                    kind: WidgetConfig::Model {
                        params: ModelParams::default(),
                    },
                    style: WidgetStyleOverride::default(),
                }],
            }],
            theme: ThemeConfig::default(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.lines.len(), 1);
        assert!(matches!(
            back.lines[0].widgets[0].kind,
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
        assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::Version));
        assert!(matches!(
            s.lines[0].widgets[1].kind,
            WidgetConfig::ClaudeSessionId
        ));
        match &s.lines[0].widgets[2].kind {
            WidgetConfig::ContextBar { params } => assert_eq!(params.width, 20),
            other => panic!("expected ContextBar, got {other:?}"),
        }
        match &s.lines[0].widgets[3].kind {
            WidgetConfig::CustomText { params } => assert_eq!(params.text, "hello"),
            other => panic!("expected CustomText, got {other:?}"),
        }
        match &s.lines[0].widgets[6].kind {
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
        match &s.lines[0].widgets[0].kind {
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
    fn parses_phase5_head_widgets() {
        let json = r#"[
            { "type": "git-branch" },
            { "type": "git-sha" },
            { "type": "git-root-dir" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::GitBranch));
        assert!(matches!(widgets[1].kind, WidgetConfig::GitSha));
        assert!(matches!(widgets[2].kind, WidgetConfig::GitRootDir));
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

    #[test]
    fn parses_phase5_status_widgets() {
        let json = r#"[
            { "type": "git-status" },
            { "type": "git-changes" },
            { "type": "git-staged" },
            { "type": "git-unstaged" },
            { "type": "git-untracked" },
            { "type": "git-conflicts" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert_eq!(widgets.len(), 6);
        assert!(matches!(widgets[0].kind, WidgetConfig::GitStatus));
        assert!(matches!(widgets[5].kind, WidgetConfig::GitConflicts));
    }

    #[test]
    fn parses_phase5_tracking_widget() {
        let json = r#"[{ "type": "git-ahead-behind" }]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::GitAheadBehind));
    }

    #[test]
    fn parses_phase5_diff_widgets() {
        let json = r#"[
            { "type": "git-insertions" },
            { "type": "git-deletions" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::GitInsertions));
        assert!(matches!(widgets[1].kind, WidgetConfig::GitDeletions));
    }

    #[test]
    fn parses_phase5_pr_widget() {
        let json = r#"[{ "type": "git-pr" }]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::GitPr));
    }

    #[test]
    fn parses_phase6_thinking_widget() {
        let json = r#"[{ "type": "thinking-effort" }]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::ThinkingEffort));
    }

    #[test]
    fn flex_mode_default_is_full() {
        let s = Settings::default();
        assert_eq!(s.theme.flex_mode, FlexMode::Full);
        assert!(!s.theme.global_bold);
        assert_eq!(s.theme.compact_threshold, 60);
        assert!(!s.theme.auto_align);
        assert!(!s.theme.continue_theme_across_lines);
    }

    #[test]
    fn theme_globals_parse_from_json() {
        let json = r##"{
            "theme": {
                "global_bold": true,
                "inherit_separator_colors": true,
                "override_background_color": "#aabbcc",
                "minimalist_mode": true,
                "flex_mode": "full-minus-40",
                "compact_threshold": 80,
                "auto_align": true,
                "continue_theme_across_lines": true
            }
        }"##;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert!(s.theme.global_bold);
        assert!(s.theme.inherit_separator_colors);
        assert_eq!(
            s.theme.override_background_color.as_deref(),
            Some("#aabbcc")
        );
        assert!(s.theme.minimalist_mode);
        assert_eq!(s.theme.flex_mode, FlexMode::FullMinus40);
        assert_eq!(s.theme.compact_threshold, 80);
        assert!(s.theme.auto_align);
        assert!(s.theme.continue_theme_across_lines);
    }

    #[test]
    fn align_right_widget_parses() {
        let json = r#"{"lines": [{"widgets": [{"type": "align-right"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert!(matches!(
            s.lines[0].widgets[0].kind,
            WidgetConfig::AlignRight
        ));
    }

    #[test]
    fn phase7_widget_variants_parse() {
        let json = r#"[
            {"type": "session-usage"}, {"type": "weekly-usage"},
            {"type": "block-reset-timer"}, {"type": "weekly-reset-timer"},
            {"type": "claude-account-email"}, {"type": "free-memory"},
            {"type": "skills"}
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert!(matches!(widgets[0].kind, WidgetConfig::SessionUsage));
        assert!(matches!(widgets[6].kind, WidgetConfig::Skills));
    }

    #[test]
    fn parses_phase6_timing_widgets() {
        let json = r#"[
            { "type": "block-timer" },
            { "type": "session-duration" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert_eq!(widgets.len(), 2);
        assert!(matches!(widgets[0].kind, WidgetConfig::BlockTimer));
        assert!(matches!(widgets[1].kind, WidgetConfig::SessionDuration));
    }

    #[test]
    fn parses_phase6_token_widgets() {
        let json = r#"[
            { "type": "tokens-cached" },
            { "type": "tokens-total" },
            { "type": "input-speed" },
            { "type": "output-speed" },
            { "type": "total-speed" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert_eq!(widgets.len(), 5);
        assert!(matches!(widgets[0].kind, WidgetConfig::TokensCached));
        assert!(matches!(widgets[4].kind, WidgetConfig::TotalSpeed));
    }

    #[test]
    fn parses_phase5_remote_widgets() {
        let json = r#"[
            { "type": "git-origin-owner" },
            { "type": "git-origin-repo" },
            { "type": "git-origin-owner-repo" },
            { "type": "git-upstream-owner" },
            { "type": "git-upstream-repo" },
            { "type": "git-upstream-owner-repo" },
            { "type": "git-is-fork" }
        ]"#;
        let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
        assert_eq!(widgets.len(), 7);
        assert!(matches!(widgets[6].kind, WidgetConfig::GitIsFork));
    }

    #[test]
    fn widget_item_parses_with_style_overrides() {
        let json = r##"{
            "lines": [{"widgets": [
                {"type": "model", "color": "#fafafa", "bold": true},
                {"type": "git-branch", "background_color": "#00ff00"}
            ]}]
        }"##;
        let s: Settings = serde_json::from_str(json).unwrap();
        let w0 = &s.lines[0].widgets[0];
        assert!(matches!(w0.kind, WidgetConfig::Model { .. }));
        assert_eq!(w0.style.color.as_deref(), Some("#fafafa"));
        assert_eq!(w0.style.bold, Some(true));
        let w1 = &s.lines[0].widgets[1];
        assert_eq!(w1.style.background_color.as_deref(), Some("#00ff00"));
        assert!(w1.style.bold.is_none());
    }

    #[test]
    fn widget_item_without_style_keeps_kind() {
        let json = r#"{"lines": [{"widgets": [{"type": "version"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let w = &s.lines[0].widgets[0];
        assert!(matches!(w.kind, WidgetConfig::Version));
        assert!(w.style.color.is_none());
        assert!(w.style.bold.is_none());
    }
}
