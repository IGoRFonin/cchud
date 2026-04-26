//! Session-cluster widgets — Phase 3 Task 3 (`SessionName`) + Task 5
//! (`SessionClock`, `SessionCost`).
//!
//! Объединены в один файл, потому что все три читают `payload.session_id`,
//! `payload.transcript_path` или `payload.cost` — общий контекст сессии.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use crate::widgets::{RenderContext, Widget};

pub struct SessionName;

impl Widget for SessionName {
    fn id(&self) -> &'static str {
        "SessionName"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let path = ctx.payload.transcript_path.as_deref()?;
        let stem = Path::new(path).file_stem()?.to_str()?;
        if stem.is_empty() {
            None
        } else {
            Some(stem.to_string())
        }
    }
}

pub struct SessionCost;

impl Widget for SessionCost {
    fn id(&self) -> &'static str {
        "SessionCost"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cost = ctx.payload.cost.as_ref()?.total_cost_usd?;
        Some(format!("${cost:.2}"))
    }
}

pub struct SessionClock;

impl Widget for SessionClock {
    fn id(&self) -> &'static str {
        "SessionClock"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let ms = ctx.payload.cost.as_ref()?.total_duration_ms?;
        Some(crate::util::duration::format_duration(ms))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_transcript(path: Option<&str>) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: path.map(String::from),
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

    #[test]
    fn session_name_strips_dot_jsonl() {
        let p = payload_with_transcript(Some(
            "/Users/igor/.claude/projects/-tmp/abc-123-def-456.jsonl",
        ));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionName.render(&ctx), Some("abc-123-def-456".into()));
    }

    #[test]
    fn session_name_returns_none_when_path_absent() {
        let p = payload_with_transcript(None);
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionName.render(&ctx), None);
    }

    #[test]
    fn session_name_handles_path_without_extension() {
        let p = payload_with_transcript(Some("/tmp/abc"));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        // file_stem на пути без extension возвращает basename
        assert_eq!(SessionName.render(&ctx), Some("abc".into()));
    }

    // ─── SessionCost ────────────────────────────────────────────

    fn payload_with_cost(cost: Option<crate::types::payload::CostInfo>) -> StatusPayload {
        let mut p = payload_with_transcript(None);
        p.cost = cost;
        p
    }

    #[test]
    fn session_cost_renders_two_decimals() {
        let p = payload_with_cost(Some(crate::types::payload::CostInfo {
            total_cost_usd: Some(0.7181426),
            total_duration_ms: Some(0),
            total_api_duration_ms: None,
            total_lines_added: None,
            total_lines_removed: None,
        }));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionCost.render(&ctx), Some("$0.72".into()));
    }

    #[test]
    fn session_cost_renders_integer_part() {
        let p = payload_with_cost(Some(crate::types::payload::CostInfo {
            total_cost_usd: Some(12.5),
            total_duration_ms: None,
            total_api_duration_ms: None,
            total_lines_added: None,
            total_lines_removed: None,
        }));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionCost.render(&ctx), Some("$12.50".into()));
    }

    #[test]
    fn session_cost_returns_none_without_cost() {
        let p = payload_with_cost(None);
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionCost.render(&ctx), None);
    }

    #[test]
    fn session_cost_returns_none_without_total_cost_usd_field() {
        let p = payload_with_cost(Some(crate::types::payload::CostInfo {
            total_cost_usd: None,
            total_duration_ms: Some(1234),
            total_api_duration_ms: None,
            total_lines_added: None,
            total_lines_removed: None,
        }));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionCost.render(&ctx), None);
    }

    // ─── SessionClock ───────────────────────────────────────────

    #[test]
    fn session_clock_under_hour() {
        // 478472 ms = 7m58s
        let p = payload_with_cost(Some(crate::types::payload::CostInfo {
            total_cost_usd: None,
            total_duration_ms: Some(478_472),
            total_api_duration_ms: None,
            total_lines_added: None,
            total_lines_removed: None,
        }));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionClock.render(&ctx), Some("07:58".into()));
    }

    #[test]
    fn session_clock_over_hour() {
        // 1h 30m 0s = 5_400_000 ms
        let p = payload_with_cost(Some(crate::types::payload::CostInfo {
            total_cost_usd: None,
            total_duration_ms: Some(5_400_000),
            total_api_duration_ms: None,
            total_lines_added: None,
            total_lines_removed: None,
        }));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionClock.render(&ctx), Some("01:30:00".into()));
    }

    #[test]
    fn session_clock_returns_none_without_cost() {
        let p = payload_with_cost(None);
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionClock.render(&ctx), None);
    }
}
