//! Transcript timing cluster.
//!
//! - `BlockTimer`: time-to-end текущего 5h billing-блока.
//! - `SessionDuration`: диапазон между первым и последним сообщением.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::util::duration::format_duration;
use crate::widgets::{RenderContext, Widget};

pub struct BlockTimer;
pub struct SessionDuration;

impl Widget for BlockTimer {
    fn id(&self) -> &'static str {
        "BlockTimer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let active = ctx.transcript()?.blocks.last()?;
        if ctx.now_ms >= active.ends_at_ms {
            return None;
        }
        let remaining_ms = active.ends_at_ms - ctx.now_ms;
        Some(format!("⏰ {}", format_duration(remaining_ms)))
    }
}

impl Widget for SessionDuration {
    fn id(&self) -> &'static str {
        "SessionDuration"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let start = t.session_started_at_ms?;
        let end = t.session_last_at_ms?;
        if end < start {
            return None;
        }
        Some(format_duration(end - start))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::{BillingBlock, TranscriptStats};
    use crate::config::default_line;
    use crate::types::payload::StatusPayload;
    use crate::widgets::test_helpers::payload_no_transcript;

    fn ctx_with<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
        stats: TranscriptStats,
        now_ms: u64,
    ) -> RenderContext<'a> {
        let mut ctx = RenderContext::new(p, s);
        ctx.now_ms = now_ms;
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn block_timer_none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn block_timer_none_when_no_blocks() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default(), 1_000_000);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn block_timer_renders_remaining_minutes() {
        let p = payload_no_transcript();
        let s = default_line();
        let block = BillingBlock {
            started_at_ms: 0,
            ends_at_ms: 5 * 3600 * 1000, // 5 часов
        };
        let stats = TranscriptStats {
            blocks: vec![block],
            ..TranscriptStats::default()
        };
        // now = 4ч 30мин
        let now_ms = (4 * 3600 + 30 * 60) * 1000;
        let ctx = ctx_with(&p, &s, stats, now_ms);
        // remaining = 30 мин = "30:00"
        assert_eq!(BlockTimer.render(&ctx).as_deref(), Some("⏰ 30:00"));
    }

    #[test]
    fn block_timer_none_when_now_past_end() {
        let p = payload_no_transcript();
        let s = default_line();
        let block = BillingBlock {
            started_at_ms: 0,
            ends_at_ms: 1000,
        };
        let stats = TranscriptStats {
            blocks: vec![block],
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 5000);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(SessionDuration.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_none_when_bounds_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default(), 0);
        assert!(SessionDuration.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_formats_minutes_seconds() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            session_started_at_ms: Some(0),
            session_last_at_ms: Some(125_000), // 2:05
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 0);
        assert_eq!(SessionDuration.render(&ctx).as_deref(), Some("02:05"));
    }

    #[test]
    fn session_duration_formats_hours_when_long() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            session_started_at_ms: Some(0),
            session_last_at_ms: Some(3_661_000), // 1ч 01м 01с
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 0);
        assert_eq!(SessionDuration.render(&ctx).as_deref(), Some("01:01:01"));
    }
}
