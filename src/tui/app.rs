//! TUI App state — Phase 8 Task 6.
#![deny(clippy::unwrap_used, clippy::expect_used)]

/// Placeholder — расширяется в Task 6.
pub struct App {
    pub settings: crate::types::config::Settings,
    pub sample: crate::types::payload::StatusPayload,
    pub transcript: Option<tempfile::NamedTempFile>,
}

impl App {
    pub fn new(
        settings: crate::types::config::Settings,
        sample: crate::types::payload::StatusPayload,
        transcript: Option<tempfile::NamedTempFile>,
    ) -> Self {
        Self { settings, sample, transcript }
    }
}
