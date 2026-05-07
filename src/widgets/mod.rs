//! Widget trait, render context, and registry/factory for widgets.
//!
//! The trait is small on purpose: each widget gets `RenderContext`
//! (immutable view of payload + settings) and returns `Option<String>`
//! (None = "nothing to show", filtered out by the renderer).
//!
//! `RenderContext` lazily resolves `git: OnceCell<Option<GitInfo>>` and
//! `transcript: OnceCell<Option<TranscriptStats>>` so widgets that don't
//! need them don't pay for IO.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod context;
pub mod custom_command;
pub mod env;
pub mod git_diff;
pub mod git_head;
pub mod git_pr;
pub mod git_remote;
pub mod git_status;
pub mod git_tracking;
pub mod model;
pub mod session;
pub mod static_text;
pub mod transcript_meta;
pub mod transcript_timing;
pub mod transcript_tokens;
pub mod trivial;
pub mod usage;
pub mod worktree;

#[cfg(test)]
pub mod test_helpers;

use crate::types::{
    config::{Settings, WidgetConfig, WidgetItem, WidgetStyleOverride},
    payload::StatusPayload,
};

pub trait Widget: Send + Sync {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
    /// Default upstream style. Themes may override via `widget_styles[id]`.
    /// Default impl picks the ccstatusline upstream foreground color from
    /// the central map; widgets needing bold/dim override directly.
    fn default_style(&self) -> crate::render::Style {
        upstream_color_ansi(self.id()).map_or_else(crate::render::Style::none, |n| {
            crate::render::Style::none().fg(crate::render::Color::Ansi256(n))
        })
    }
    /// Optional URL to wrap the rendered text in OSC 8. Default: none.
    fn hyperlink(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        None
    }
}

/// True if the widget renders an inherent label/icon prefix that
/// `WidgetItem.raw_value = true` strips (parity with ccstatusline `rawValue`).
#[must_use]
pub const fn widget_supports_raw_value(kind: &WidgetConfig) -> bool {
    matches!(
        kind,
        WidgetConfig::ContextLength
            | WidgetConfig::TokensInput
            | WidgetConfig::TokensOutput
            | WidgetConfig::TokensCached
            | WidgetConfig::TokensTotal
            | WidgetConfig::InputSpeed
            | WidgetConfig::OutputSpeed
            | WidgetConfig::TotalSpeed
            | WidgetConfig::ThinkingEffort
    )
}

/// ccstatusline upstream default foreground colors per widget id.
/// Mirrors `getDefaultColor()` in upstream `src/widgets/*.ts`.
/// Returns ANSI 0–15 code; consumers wrap as `Color::Ansi256(n)`.
#[must_use]
pub fn upstream_color_ansi(id: &str) -> Option<u8> {
    Some(match id {
        // ── cyan (6) — info & identifiers ───────────────────────
        "Model" | "ClaudeSessionId" | "FreeMemory" | "GitAheadBehind"
        | "GitOriginOwner" | "GitOriginRepo" | "GitOriginOwnerRepo"
        | "GitPr" | "GitRootDir" | "InputSpeed" | "OutputSpeed"
        | "TotalSpeed" | "OutputStyle" | "SessionName" | "TokensCached"
        | "TokensTotal" => 6,

        // ── blue (4) ─────────────────────────────────────────────
        "ClaudeAccountEmail" | "ContextBar" | "ContextPercentage"
        | "CurrentWorkingDir" | "TokensInput" | "Worktree" => 4,

        // ── green (2) ────────────────────────────────────────────
        "ContextPercentageUsable" | "GitInsertions" | "GitStaged"
        | "SessionCost" | "VimMode" => 2,

        // ── yellow (3) ───────────────────────────────────────────
        "BlockTimer" | "GitChanges" | "GitIsFork" | "GitStatus"
        | "GitUnstaged" | "SessionClock" | "WorktreeMode"
        | "WorktreeName" | "WorktreeBranch" | "WorktreeOriginalBranch" => 3,

        // ── red (1) ──────────────────────────────────────────────
        "GitConflicts" | "GitDeletions" | "GitUntracked" => 1,

        // ── magenta (5) ──────────────────────────────────────────
        "GitBranch" | "GitUpstreamOwner" | "GitUpstreamRepo"
        | "GitUpstreamOwnerRepo" | "ThinkingEffort" | "Skills" => 5,

        // ── white (7) ────────────────────────────────────────────
        "CustomCommand" | "TokensOutput" => 7,

        // ── gray / brightBlack (8) ───────────────────────────────
        "ContextLength" | "GitSha" | "TerminalWidth" | "Version" => 8,

        // ── brightBlue (12) ──────────────────────────────────────
        "BlockResetTimer" | "SessionUsage" | "WeeklyResetTimer"
        | "WeeklyUsage" => 12,

        _ => return None,
    })
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    #[allow(dead_code)]
    pub settings: &'a Settings,
    /// Lazy git discover. None если cwd не git-репо.
    #[allow(dead_code)]
    git: std::cell::OnceCell<Option<crate::git::GitInfo>>,
    /// Lazy transcript-кэш. None если payload без `transcript_path`
    /// или транскрипт недоступен.
    #[allow(dead_code)]
    transcript: std::cell::OnceCell<Option<crate::cache::TranscriptStats>>,
    /// Текущее время в Unix-ms. Дефолт = `unix_now_ms()`.
    /// Тесты могут перезаписать через field-init синтаксис.
    #[allow(dead_code)]
    pub now_ms: u64,
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
        Self {
            payload,
            settings,
            git: std::cell::OnceCell::new(),
            transcript: std::cell::OnceCell::new(),
            now_ms: crate::util::now::unix_now_ms(),
        }
    }

    /// Lazy: вызывает `gix::discover(cwd)` максимум один раз. None если
    /// payload без cwd или cwd вне git-репо.
    #[allow(dead_code)]
    pub fn git(&self) -> Option<&crate::git::GitInfo> {
        self.git
            .get_or_init(|| {
                let cwd = self.payload.workspace.current_dir.as_str();
                crate::git::GitInfo::discover(std::path::Path::new(cwd))
            })
            .as_ref()
    }

    /// Lazy: парсит JSONL-транскрипт через `cache::load_or_build_incremental`
    /// максимум один раз за render. None если `payload.transcript_path`
    /// пусто, файл не читается или JSONL битый.
    #[allow(dead_code)]
    pub fn transcript(&self) -> Option<&crate::cache::TranscriptStats> {
        self.transcript
            .get_or_init(|| {
                let path = self.payload.transcript_path.as_deref()?;
                crate::cache::load_or_build_incremental(std::path::Path::new(path))
            })
            .as_ref()
    }

    /// Test-only: pre-populate transcript cell с фиксированной `TranscriptStats`.
    /// Используется в unit-тестах transcript-кластеров чтобы не зависеть от файлового IO.
    #[cfg(test)]
    pub fn set_transcript_for_tests(&self, stats: Option<crate::cache::TranscriptStats>) {
        let _ = self.transcript.set(stats);
    }
}

#[must_use]
pub fn build_widgets(settings: &Settings) -> Vec<Vec<(Box<dyn Widget>, WidgetStyleOverride)>> {
    settings
        .lines
        .iter()
        .map(|line| {
            line.widgets
                .iter()
                .map(|item| (build_one(item), item.style.clone()))
                .collect()
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod build_widgets_tests {
    use super::*;
    use crate::types::config::Settings;

    #[test]
    fn build_widgets_returns_one_inner_vec_per_line() {
        let json = r#"{
            "lines": [
                {"widgets": [{"type": "model"}]},
                {"widgets": [{"type": "git-branch"}, {"type": "git-status"}]}
            ]
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let lines = build_widgets(&s);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[1].len(), 2);
    }

    #[test]
    fn build_widgets_empty_settings_yields_empty_vec() {
        let lines = build_widgets(&Settings::default());
        assert!(lines.is_empty());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod default_style_tests {
    use super::*;

    #[test]
    fn model_default_style_is_bold_cyan() {
        let s = model::Model.default_style();
        assert!(s.bold);
        assert_eq!(s.fg, Some(crate::render::Color::Ansi256(6)));
    }

    #[test]
    fn worktree_default_style_uses_upstream_colors() {
        // Upstream: GitWorktree = blue, mode/name/branches = yellow.
        assert_eq!(
            worktree::Worktree.default_style().fg,
            Some(crate::render::Color::Ansi256(4))
        );
        for w in [
            &worktree::WorktreeMode as &dyn Widget,
            &worktree::WorktreeName,
            &worktree::WorktreeBranch,
            &worktree::WorktreeOriginalBranch,
        ] {
            assert_eq!(
                w.default_style().fg,
                Some(crate::render::Color::Ansi256(3)),
                "{} should be yellow",
                w.id()
            );
        }
    }

    #[test]
    fn session_cost_has_green_fg() {
        let s = session::SessionCost.default_style();
        assert_eq!(s.fg, Some(crate::render::Color::Ansi256(2)));
    }

    #[test]
    fn session_clock_has_yellow_fg() {
        let s = session::SessionClock.default_style();
        assert_eq!(s.fg, Some(crate::render::Color::Ansi256(3)));
    }

    #[test]
    fn upstream_color_map_returns_none_for_unknown() {
        assert_eq!(upstream_color_ansi("DoesNotExist"), None);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod transcript_ctx_tests {
    use super::*;
    use crate::config::default_line;
    use crate::widgets::test_helpers::payload_no_transcript;

    #[test]
    fn transcript_returns_none_when_path_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(ctx.transcript().is_none());
    }

    #[test]
    fn transcript_returns_none_for_invalid_path() {
        let mut p = payload_no_transcript();
        p.transcript_path = Some("/nonexistent/__cchud_test_does_not_exist.jsonl".into());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(ctx.transcript().is_none());
    }

    #[test]
    fn now_ms_can_be_overridden_for_tests() {
        let p = payload_no_transcript();
        let s = default_line();
        let mut ctx = RenderContext::new(&p, &s);
        ctx.now_ms = 1_234_567_890;
        assert_eq!(ctx.now_ms, 1_234_567_890);
    }
}

struct SeparatorWidget;
impl Widget for SeparatorWidget {
    fn id(&self) -> &'static str {
        "separator"
    }
    fn render(&self, _: &RenderContext<'_>) -> Option<String> {
        None
    }
}

fn build_one(item: &WidgetItem) -> Box<dyn Widget> {
    let raw = item.raw_value;
    match &item.kind {
        WidgetConfig::Model { .. } => Box::new(model::Model),
        WidgetConfig::Separator => Box::new(SeparatorWidget),

        // static cluster:
        WidgetConfig::CustomText { params } => Box::new(static_text::CustomText {
            params: params.clone(),
        }),
        WidgetConfig::CustomSymbol { params } => Box::new(static_text::CustomSymbol {
            params: params.clone(),
        }),
        WidgetConfig::Link { params } => Box::new(static_text::Link {
            params: params.clone(),
        }),

        // trivial cluster:
        WidgetConfig::Version => Box::new(trivial::Version),
        WidgetConfig::ClaudeSessionId => Box::new(trivial::ClaudeSessionId),
        WidgetConfig::TerminalWidth => Box::new(trivial::TerminalWidth),
        WidgetConfig::OutputStyle => Box::new(trivial::OutputStyle),
        WidgetConfig::VimMode => Box::new(trivial::VimMode),
        WidgetConfig::SessionName => Box::new(session::SessionName),
        // cost cluster:
        WidgetConfig::SessionClock => Box::new(session::SessionClock),
        WidgetConfig::SessionCost => Box::new(session::SessionCost),
        // context cluster:
        WidgetConfig::ContextLength => Box::new(context::ContextLength { raw_value: raw }),
        WidgetConfig::ContextPercentage => Box::new(context::ContextPercentage),
        WidgetConfig::ContextPercentageUsable => Box::new(context::ContextPercentageUsable),
        WidgetConfig::ContextBar { params } => Box::new(context::ContextBar {
            params: params.clone(),
        }),
        WidgetConfig::TokensInput => Box::new(context::TokensInput { raw_value: raw }),
        WidgetConfig::TokensOutput => Box::new(context::TokensOutput { raw_value: raw }),
        // worktree cluster:
        WidgetConfig::Worktree => Box::new(worktree::Worktree),
        WidgetConfig::WorktreeMode => Box::new(worktree::WorktreeMode),
        WidgetConfig::WorktreeName => Box::new(worktree::WorktreeName),
        WidgetConfig::WorktreeBranch => Box::new(worktree::WorktreeBranch),
        WidgetConfig::WorktreeOriginalBranch => Box::new(worktree::WorktreeOriginalBranch),
        // custom-command:
        WidgetConfig::CustomCommand { params } => Box::new(custom_command::CustomCommand {
            params: params.clone(),
        }),

        // head cluster:
        WidgetConfig::GitBranch => Box::new(git_head::GitBranch),
        WidgetConfig::GitSha => Box::new(git_head::GitSha),
        WidgetConfig::GitRootDir => Box::new(git_head::GitRootDir),
        // status cluster:
        WidgetConfig::GitStatus => Box::new(git_status::GitStatus),
        WidgetConfig::GitChanges => Box::new(git_status::GitChanges),
        WidgetConfig::GitStaged => Box::new(git_status::GitStaged),
        WidgetConfig::GitUnstaged => Box::new(git_status::GitUnstaged),
        WidgetConfig::GitUntracked => Box::new(git_status::GitUntracked),
        WidgetConfig::GitConflicts => Box::new(git_status::GitConflicts),
        // diff stat:
        WidgetConfig::GitInsertions => Box::new(git_diff::GitInsertions),
        WidgetConfig::GitDeletions => Box::new(git_diff::GitDeletions),
        // tracking:
        WidgetConfig::GitAheadBehind => Box::new(git_tracking::GitAheadBehind),
        // remote:
        WidgetConfig::GitOriginOwner => Box::new(git_remote::GitOriginOwner),
        WidgetConfig::GitOriginRepo => Box::new(git_remote::GitOriginRepo),
        WidgetConfig::GitOriginOwnerRepo => Box::new(git_remote::GitOriginOwnerRepo),
        WidgetConfig::GitUpstreamOwner => Box::new(git_remote::GitUpstreamOwner),
        WidgetConfig::GitUpstreamRepo => Box::new(git_remote::GitUpstreamRepo),
        WidgetConfig::GitUpstreamOwnerRepo => Box::new(git_remote::GitUpstreamOwnerRepo),
        WidgetConfig::GitIsFork => Box::new(git_remote::GitIsFork),
        // PR:
        WidgetConfig::GitPr => Box::new(git_pr::GitPr),

        // transcript tokens cluster:
        WidgetConfig::TokensCached => {
            Box::new(transcript_tokens::TokensCached { raw_value: raw })
        }
        WidgetConfig::TokensTotal => Box::new(transcript_tokens::TokensTotal { raw_value: raw }),
        WidgetConfig::InputSpeed => Box::new(transcript_tokens::InputSpeed { raw_value: raw }),
        WidgetConfig::OutputSpeed => Box::new(transcript_tokens::OutputSpeed { raw_value: raw }),
        WidgetConfig::TotalSpeed => Box::new(transcript_tokens::TotalSpeed { raw_value: raw }),
        // transcript timing cluster:
        WidgetConfig::BlockTimer => Box::new(transcript_timing::BlockTimer),
        WidgetConfig::SessionDuration => Box::new(transcript_timing::SessionDuration),
        // transcript meta cluster:
        WidgetConfig::ThinkingEffort => Box::new(transcript_meta::ThinkingEffort {
            raw_value: raw,
        }),

        // usage cluster:
        WidgetConfig::SessionUsage => Box::new(usage::SessionUsage),
        WidgetConfig::WeeklyUsage => Box::new(usage::WeeklyUsage),
        WidgetConfig::BlockResetTimer => Box::new(usage::BlockResetTimer),
        WidgetConfig::WeeklyResetTimer => Box::new(usage::WeeklyResetTimer),

        // env cluster:
        WidgetConfig::ClaudeAccountEmail => Box::new(env::ClaudeAccountEmail),
        WidgetConfig::FreeMemory => Box::new(env::FreeMemory),
        WidgetConfig::CurrentWorkingDir { params } => Box::new(env::CurrentWorkingDir {
            params: params.clone(),
        }),

        // transcript meta:
        WidgetConfig::Skills => Box::new(transcript_meta::Skills),

        // AlignRight sentinel:
        WidgetConfig::AlignRight => Box::new(AlignRightSentinel),
    }
}

struct AlignRightSentinel;
impl Widget for AlignRightSentinel {
    fn id(&self) -> &'static str {
        "align-right"
    }
    fn render(&self, _: &RenderContext<'_>) -> Option<String> {
        Some(String::new())
    }
}
