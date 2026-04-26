//! Widget trait, render context, and registry/factory for widgets.
//!
//! The trait is small on purpose: each widget gets `RenderContext`
//! (immutable view of payload + settings) and returns `Option<String>`
//! (None = "nothing to show", filtered out by the renderer).
//!
//! Phase 5 will add `git: OnceCell<Option<GitInfo>>` to `RenderContext`.
//! Phase 6 will add `transcript: OnceCell<Option<TranscriptCache>>`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::payload::StatusPayload;

pub mod model;

pub trait Widget: Send + Sync {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub const fn new(payload: &'a StatusPayload) -> Self {
        Self { payload }
    }
}

/// Build the list of widgets. In Task 4 returns a hardcoded `[Model]`.
/// In Task 6 (after `Settings` exists) it will iterate `settings.lines[0].widgets`.
#[must_use]
pub fn build_widgets() -> Vec<Box<dyn Widget>> {
    vec![Box::new(model::Model)]
}
