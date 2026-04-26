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
pub mod model;
pub mod session;
pub mod static_text;
pub mod trivial;

use crate::types::{
    config::{Settings, WidgetConfig},
    payload::StatusPayload,
};

pub trait Widget: Send + Sync {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    #[allow(dead_code)]
    pub settings: &'a Settings,
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub const fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
        Self { payload, settings }
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

/// Stub Widget — placeholder для variants, чьи impl ещё не написаны
/// (Phase 3: вытесняется match-arms по мере роста кластеров T2–T7).
struct Stub(&'static str);
impl Widget for Stub {
    fn id(&self) -> &'static str {
        self.0
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        None
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

        // Phase 3 stubs — заменяются на реальные impl в T3–T7:
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
        WidgetConfig::Worktree => Box::new(Stub("Worktree")),
        WidgetConfig::WorktreeMode => Box::new(Stub("WorktreeMode")),
        WidgetConfig::WorktreeName => Box::new(Stub("WorktreeName")),
        WidgetConfig::WorktreeBranch => Box::new(Stub("WorktreeBranch")),
        WidgetConfig::WorktreeOriginalBranch => Box::new(Stub("WorktreeOriginalBranch")),
        WidgetConfig::CustomCommand { .. } => Box::new(Stub("CustomCommand")),
    }
}
