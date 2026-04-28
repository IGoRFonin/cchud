#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::cache::{MessageStats, TranscriptStats};
use crate::util::format_tokens::format_tokens;
use crate::widgets::{RenderContext, Widget};

pub struct TokensCached;
pub struct TokensTotal;
pub struct InputSpeed;
pub struct OutputSpeed;
pub struct TotalSpeed;

impl Widget for TokensCached {
    fn id(&self) -> &'static str {
        "TokensCached"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let cached = t
            .tokens_cache_read_total
            .saturating_add(t.tokens_cache_creation_total);
        if cached == 0 {
            return None;
        }
        Some(format!("cT: {}", format_tokens(cached)))
    }
}

impl Widget for TokensTotal {
    fn id(&self) -> &'static str {
        "TokensTotal"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let total = total_tokens(t);
        if total == 0 {
            return None;
        }
        Some(format!("totT: {}", format_tokens(total)))
    }
}

impl Widget for InputSpeed {
    fn id(&self) -> &'static str {
        "InputSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        Some(format!("↓{} t/s", speed(last.tokens_in, &last)))
    }
}

impl Widget for OutputSpeed {
    fn id(&self) -> &'static str {
        "OutputSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        Some(format!("↑{} t/s", speed(last.tokens_out, &last)))
    }
}

impl Widget for TotalSpeed {
    fn id(&self) -> &'static str {
        "TotalSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        let total = last.tokens_in.saturating_add(last.tokens_out);
        Some(format!("⇅{} t/s", speed(total, &last)))
    }
}

const fn total_tokens(t: &TranscriptStats) -> u64 {
    t.tokens_in_total
        .saturating_add(t.tokens_out_total)
        .saturating_add(t.tokens_cache_read_total)
        .saturating_add(t.tokens_cache_creation_total)
}

fn speed(tokens: u64, msg: &MessageStats) -> u64 {
    let duration_ms = msg.completed_at_ms.saturating_sub(msg.started_at_ms);
    let duration_sec = (duration_ms / 1000).max(1);
    tokens / duration_sec
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::TranscriptStats;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};
    use crate::widgets::RenderContext;

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

    fn ctx_with_stats<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
        stats: TranscriptStats,
    ) -> RenderContext<'a> {
        let ctx = RenderContext::new(p, s);
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn tokens_cached_none_when_transcript_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(TokensCached.render(&ctx).is_none());
    }

    #[test]
    fn tokens_cached_none_when_zero() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats::default();
        let ctx = ctx_with_stats(&p, &s, stats);
        assert!(TokensCached.render(&ctx).is_none());
    }

    #[test]
    fn tokens_cached_sums_read_and_creation() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            tokens_cache_read_total: 700,
            tokens_cache_creation_total: 300,
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(TokensCached.render(&ctx).as_deref(), Some("cT: 1k"));
    }

    #[test]
    fn tokens_total_none_when_zero() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats::default();
        let ctx = ctx_with_stats(&p, &s, stats);
        assert!(TokensTotal.render(&ctx).is_none());
    }

    #[test]
    fn tokens_total_sums_all_four_fields() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            tokens_in_total: 1000,
            tokens_out_total: 500,
            tokens_cache_read_total: 2000,
            tokens_cache_creation_total: 1000,
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        // 1000 + 500 + 2000 + 1000 = 4500 → "4.5k"
        assert_eq!(TokensTotal.render(&ctx).as_deref(), Some("totT: 4.5k"));
    }

    #[test]
    fn input_speed_none_when_no_last_assistant() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with_stats(&p, &s, TranscriptStats::default());
        assert!(InputSpeed.render(&ctx).is_none());
    }

    #[test]
    fn input_speed_uses_max_one_second_floor() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 1_000,
                completed_at_ms: 1_100, // 100 ms — duration < 1s, floor = 1s
                tokens_in: 1500,
                tokens_out: 500,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(InputSpeed.render(&ctx).as_deref(), Some("↓1500 t/s"));
    }

    #[test]
    fn output_speed_uses_real_duration_when_more_than_1s() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 0,
                completed_at_ms: 5_000, // 5 sec
                tokens_in: 0,
                tokens_out: 250,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(OutputSpeed.render(&ctx).as_deref(), Some("↑50 t/s"));
    }

    #[test]
    fn total_speed_sums_in_and_out() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 0,
                completed_at_ms: 2_000, // 2 sec
                tokens_in: 100,
                tokens_out: 100,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        // (100 + 100) / 2 = 100
        assert_eq!(TotalSpeed.render(&ctx).as_deref(), Some("⇅100 t/s"));
    }
}
