//! Widget trait, render context, and registry/factory for widgets.
//!
//! The trait is small on purpose: each widget gets `RenderContext`
//! (immutable view of payload + settings) and returns `Option<String>`
//! (None = "nothing to show", filtered out by the renderer).
//!
//! Phase 5 added `git: OnceCell<Option<GitInfo>>` to `RenderContext`.
//! Phase 6 added `transcript: OnceCell<Option<TranscriptStats>>`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod context;
pub mod custom_command;
pub mod env;       // Phase 7 — env cluster
pub mod usage;     // Phase 7 — usage cluster
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
pub mod worktree;

#[cfg(test)]
pub(crate) mod test_helpers;

use crate::types::{
    config::{Settings, WidgetConfig},
    payload::StatusPayload,
};

pub trait Widget: Send + Sync {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
    /// Default upstream style. Themes may override via `widget_styles[id]`.
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none()
    }
    /// Optional URL to wrap the rendered text in OSC 8. Default: none.
    fn hyperlink(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        None
    }
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    #[allow(dead_code)]
    pub settings: &'a Settings,
    /// Phase 5: lazy git discover. None если cwd не git-репо.
    #[allow(dead_code)]
    git: std::cell::OnceCell<Option<crate::git::GitInfo>>,
    /// Phase 6: lazy transcript-кэш. None если payload без `transcript_path`
    /// или транскрипт недоступен.
    #[allow(dead_code)]
    transcript: std::cell::OnceCell<Option<crate::cache::TranscriptStats>>,
    /// Phase 6: текущее время в Unix-ms. Дефолт = `unix_now_ms()`.
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
    /// Используется в unit-тестах T6/T7/T8 чтобы не зависеть от файлового IO.
    #[cfg(test)]
    pub fn set_transcript_for_tests(&self, stats: Option<crate::cache::TranscriptStats>) {
        let _ = self.transcript.set(stats);
    }
}

#[must_use]
pub fn build_widgets(settings: &Settings) -> Vec<Box<dyn Widget>> {
    settings
        .lines
        .first()
        .map(|line| line.widgets.iter().map(build_one).collect())
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod default_style_tests {
    use super::*;
    use crate::render::Style;

    #[test]
    fn model_default_style_is_bold() {
        let s = model::Model.default_style();
        assert!(s.bold);
    }

    #[test]
    fn worktree_default_style_is_dim() {
        for w in [
            &worktree::Worktree as &dyn Widget,
            &worktree::WorktreeMode,
            &worktree::WorktreeName,
            &worktree::WorktreeBranch,
            &worktree::WorktreeOriginalBranch,
        ] {
            assert!(w.default_style().dim, "{} should be dim", w.id());
        }
    }

    #[test]
    fn session_cost_has_green_fg() {
        let s = session::SessionCost.default_style();
        assert!(s.fg.is_some());
    }

    #[test]
    fn unaffected_widgets_use_style_none() {
        assert_eq!(model::Model.default_style().fg, None); // bold-only, no fg
        assert_eq!(
            session::SessionClock.default_style(),
            Style::none(),
            "SessionClock has no upstream style"
        );
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

fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
    match cfg {
        WidgetConfig::Model { .. } => Box::new(model::Model),

        // Phase 3 — Task 2 (static cluster):
        WidgetConfig::CustomText { params } => Box::new(static_text::CustomText {
            params: params.clone(),
        }),
        WidgetConfig::CustomSymbol { params } => Box::new(static_text::CustomSymbol {
            params: params.clone(),
        }),
        WidgetConfig::Link { params } => Box::new(static_text::Link {
            params: params.clone(),
        }),

        // Phase 3 — Task 3 (trivial cluster):
        WidgetConfig::Version => Box::new(trivial::Version),
        WidgetConfig::ClaudeSessionId => Box::new(trivial::ClaudeSessionId),
        WidgetConfig::TerminalWidth => Box::new(trivial::TerminalWidth),
        WidgetConfig::OutputStyle => Box::new(trivial::OutputStyle),
        WidgetConfig::VimMode => Box::new(trivial::VimMode),
        WidgetConfig::SessionName => Box::new(session::SessionName),
        // Phase 3 — Task 5 (cost cluster):
        WidgetConfig::SessionClock => Box::new(session::SessionClock),
        WidgetConfig::SessionCost => Box::new(session::SessionCost),
        // Phase 3 — Task 4 (context cluster):
        WidgetConfig::ContextLength => Box::new(context::ContextLength),
        WidgetConfig::ContextPercentage => Box::new(context::ContextPercentage),
        WidgetConfig::ContextPercentageUsable => Box::new(context::ContextPercentageUsable),
        WidgetConfig::ContextBar { params } => Box::new(context::ContextBar {
            params: params.clone(),
        }),
        WidgetConfig::TokensInput => Box::new(context::TokensInput),
        WidgetConfig::TokensOutput => Box::new(context::TokensOutput),
        // Phase 3 — Task 6 (worktree cluster):
        WidgetConfig::Worktree => Box::new(worktree::Worktree),
        WidgetConfig::WorktreeMode => Box::new(worktree::WorktreeMode),
        WidgetConfig::WorktreeName => Box::new(worktree::WorktreeName),
        WidgetConfig::WorktreeBranch => Box::new(worktree::WorktreeBranch),
        WidgetConfig::WorktreeOriginalBranch => Box::new(worktree::WorktreeOriginalBranch),
        // Phase 3 — Task 7 (custom-command):
        WidgetConfig::CustomCommand { params } => Box::new(custom_command::CustomCommand {
            params: params.clone(),
        }),

        // Phase 5 — Task 2 (head cluster):
        WidgetConfig::GitBranch => Box::new(git_head::GitBranch),
        WidgetConfig::GitSha => Box::new(git_head::GitSha),
        WidgetConfig::GitRootDir => Box::new(git_head::GitRootDir),
        // Phase 5 — Task 3 (status cluster):
        WidgetConfig::GitStatus => Box::new(git_status::GitStatus),
        WidgetConfig::GitChanges => Box::new(git_status::GitChanges),
        WidgetConfig::GitStaged => Box::new(git_status::GitStaged),
        WidgetConfig::GitUnstaged => Box::new(git_status::GitUnstaged),
        WidgetConfig::GitUntracked => Box::new(git_status::GitUntracked),
        WidgetConfig::GitConflicts => Box::new(git_status::GitConflicts),
        // Phase 5 — Task 4 (diff stat):
        WidgetConfig::GitInsertions => Box::new(git_diff::GitInsertions),
        WidgetConfig::GitDeletions => Box::new(git_diff::GitDeletions),
        // Phase 5 — Task 5 (tracking):
        WidgetConfig::GitAheadBehind => Box::new(git_tracking::GitAheadBehind),
        // Phase 5 — Task 6 (remote):
        WidgetConfig::GitOriginOwner => Box::new(git_remote::GitOriginOwner),
        WidgetConfig::GitOriginRepo => Box::new(git_remote::GitOriginRepo),
        WidgetConfig::GitOriginOwnerRepo => Box::new(git_remote::GitOriginOwnerRepo),
        WidgetConfig::GitUpstreamOwner => Box::new(git_remote::GitUpstreamOwner),
        WidgetConfig::GitUpstreamRepo => Box::new(git_remote::GitUpstreamRepo),
        WidgetConfig::GitUpstreamOwnerRepo => Box::new(git_remote::GitUpstreamOwnerRepo),
        WidgetConfig::GitIsFork => Box::new(git_remote::GitIsFork),
        // Phase 5 — Task 7 (PR):
        WidgetConfig::GitPr => Box::new(git_pr::GitPr),

        // Phase 6 — Task 6 (transcript tokens cluster):
        WidgetConfig::TokensCached => Box::new(transcript_tokens::TokensCached),
        WidgetConfig::TokensTotal => Box::new(transcript_tokens::TokensTotal),
        WidgetConfig::InputSpeed => Box::new(transcript_tokens::InputSpeed),
        WidgetConfig::OutputSpeed => Box::new(transcript_tokens::OutputSpeed),
        WidgetConfig::TotalSpeed => Box::new(transcript_tokens::TotalSpeed),
        // Phase 6 — Task 7 (transcript timing cluster):
        WidgetConfig::BlockTimer => Box::new(transcript_timing::BlockTimer),
        WidgetConfig::SessionDuration => Box::new(transcript_timing::SessionDuration),
        // Phase 6 — Task 8 (transcript meta cluster):
        WidgetConfig::ThinkingEffort => Box::new(transcript_meta::ThinkingEffort),
    }
}
