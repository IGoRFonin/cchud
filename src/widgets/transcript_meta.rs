//! Transcript meta cluster — Phase 6 Task 8.
//!
//! `ThinkingEffort` отображает уровень thinking из последнего
//! assistant-сообщения. Принимаем любую непустую строку (low/medium/high/
//! xhigh/max — текущие значения CC; future-proof).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct ThinkingEffort;

impl Widget for ThinkingEffort {
    fn id(&self) -> &'static str {
        "ThinkingEffort"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let level = ctx.transcript()?.last_thinking_effort.as_deref()?;
        if level.is_empty() {
            return None;
        }
        Some(format!("🧠 {level}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::TranscriptStats;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_no_transcript() -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "m".into(),
                display_name: "M".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn ctx_with<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
        stats: TranscriptStats,
    ) -> RenderContext<'a> {
        let ctx = RenderContext::new(p, s);
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(ThinkingEffort.render(&ctx).is_none());
    }

    #[test]
    fn none_when_field_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default());
        assert!(ThinkingEffort.render(&ctx).is_none());
    }

    #[test]
    fn renders_high_level() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some("high".into()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert_eq!(ThinkingEffort.render(&ctx).as_deref(), Some("🧠 high"));
    }

    #[test]
    fn renders_unknown_future_level_unchanged() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some("ultra".into()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert_eq!(ThinkingEffort.render(&ctx).as_deref(), Some("🧠 ultra"));
    }

    #[test]
    fn empty_string_returns_none() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some(String::new()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert!(ThinkingEffort.render(&ctx).is_none());
    }
}
