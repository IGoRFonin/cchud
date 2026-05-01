//! Usage cluster widgets — Phase 7 Task 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::payload::{RateBucket, RateLimits};
use crate::util::format_duration_long::{format_long, format_short};
use crate::widgets::{RenderContext, Widget};

pub struct SessionUsage;
pub struct WeeklyUsage;
pub struct BlockResetTimer;
pub struct WeeklyResetTimer;

const fn buckets<'a>(ctx: &'a RenderContext<'a>) -> Option<&'a RateLimits> {
    ctx.payload.rate_limits.as_ref()
}

fn pct_str(b: &RateBucket, prefix: &str) -> Option<String> {
    let pct = b.used_percentage?;
    Some(format!("{prefix}: {pct:.0}%"))
}

fn timer_str(b: &RateBucket, now_ms: u64, long: bool) -> Option<String> {
    let resets_at = b.resets_at?;
    let now_unix_s = i64::try_from(now_ms / 1000).unwrap_or(i64::MAX);
    let remaining = resets_at - now_unix_s;
    let body = if long {
        format_long(remaining)
    } else {
        format_short(remaining)
    };
    Some(format!("⏳ {body}"))
}

impl Widget for SessionUsage {
    fn id(&self) -> &'static str {
        "session-usage"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        pct_str(buckets(ctx)?.five_hour.as_ref()?, "5h")
    }
}

impl Widget for WeeklyUsage {
    fn id(&self) -> &'static str {
        "weekly-usage"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        pct_str(buckets(ctx)?.seven_day.as_ref()?, "7d")
    }
}

impl Widget for BlockResetTimer {
    fn id(&self) -> &'static str {
        "block-reset-timer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        timer_str(buckets(ctx)?.five_hour.as_ref()?, ctx.now_ms, false)
    }
}

impl Widget for WeeklyResetTimer {
    fn id(&self) -> &'static str {
        "weekly-reset-timer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        timer_str(buckets(ctx)?.seven_day.as_ref()?, ctx.now_ms, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::test_helpers::payload_no_transcript;

    fn payload_with_buckets(
        five_pct: f64,
        five_resets: i64,
        weekly_pct: f64,
        weekly_resets: i64,
    ) -> crate::types::payload::StatusPayload {
        let mut p = payload_no_transcript();
        p.rate_limits = Some(RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(five_pct),
                resets_at: Some(five_resets),
            }),
            seven_day: Some(RateBucket {
                used_percentage: Some(weekly_pct),
                resets_at: Some(weekly_resets),
            }),
        });
        p
    }

    #[test]
    fn session_usage_renders_pct() {
        let p = payload_with_buckets(1.0, 0, 6.0, 0);
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionUsage.render(&ctx), Some("5h: 1%".into()));
    }

    #[test]
    fn weekly_usage_renders_pct() {
        let p = payload_with_buckets(1.0, 0, 6.0, 0);
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(WeeklyUsage.render(&ctx), Some("7d: 6%".into()));
    }

    #[test]
    fn session_usage_returns_none_without_payload() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(SessionUsage.render(&ctx).is_none());
    }

    #[test]
    fn block_reset_timer_renders_short_format() {
        let now_ms = 1_700_000_000_000_u64;
        let resets_at_unix_s = (now_ms / 1000) as i64 + 4 * 3600 + 32 * 60;
        let p = payload_with_buckets(1.0, resets_at_unix_s, 6.0, 0);
        let s = crate::config::default_line();
        let mut ctx = RenderContext::new(&p, &s);
        ctx.now_ms = now_ms;
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ 4h32m".into()));
    }

    #[test]
    fn weekly_reset_timer_renders_long_format() {
        let now_ms = 1_700_000_000_000_u64;
        let resets_at = (now_ms / 1000) as i64 + 5 * 86400 + 14 * 3600;
        let p = payload_with_buckets(1.0, 0, 6.0, resets_at);
        let s = crate::config::default_line();
        let mut ctx = RenderContext::new(&p, &s);
        ctx.now_ms = now_ms;
        assert_eq!(WeeklyResetTimer.render(&ctx), Some("⏳ 5d 14h".into()));
    }

    #[test]
    fn reset_timer_in_past_returns_under_1m() {
        let now_ms = 1_700_000_000_000_u64;
        let resets_at = (now_ms / 1000) as i64 - 100;
        let p = payload_with_buckets(1.0, resets_at, 6.0, 0);
        let s = crate::config::default_line();
        let mut ctx = RenderContext::new(&p, &s);
        ctx.now_ms = now_ms;
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ < 1m".into()));
    }

    #[test]
    fn reset_timer_returns_none_without_resets_at() {
        let mut p = payload_no_transcript();
        p.rate_limits = Some(RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(1.0),
                resets_at: None,
            }),
            seven_day: None,
        });
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(BlockResetTimer.render(&ctx).is_none());
    }

    #[test]
    fn percentage_floor_rounds_using_format_default() {
        let p = payload_with_buckets(1.9, 0, 6.7, 0);
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        // {pct:.0} использует banker rounding → 1.9 → 2; 6.7 → 7.
        assert_eq!(SessionUsage.render(&ctx), Some("5h: 2%".into()));
        assert_eq!(WeeklyUsage.render(&ctx), Some("7d: 7%".into()));
    }
}
