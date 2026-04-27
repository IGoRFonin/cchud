//! Widget trait, render context, and registry/factory for widgets.
//!
//! The trait is small on purpose: each widget gets `RenderContext`
//! (immutable view of payload + settings) and returns `Option<String>`
//! (None = "nothing to show", filtered out by the renderer).
//!
//! Phase 5 will add `git: OnceCell<Option<GitInfo>>` to `RenderContext`.
//! Phase 6 will add `transcript: OnceCell<Option<TranscriptCache>>`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod context;
pub mod custom_command;
pub mod git_head;
pub mod git_status;
pub mod model;
pub mod session;
pub mod static_text;
pub mod trivial;
pub mod worktree;

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
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub const fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
        Self {
            payload,
            settings,
            git: std::cell::OnceCell::new(),
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
    }
}
