//! Usage cluster widgets.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::payload::RateBucket;
use crate::util::format_duration_long::{format_long, format_short};
use crate::widgets::{RenderContext, Widget};

pub struct SessionUsage;
pub struct WeeklyUsage;
pub struct BlockResetTimer;
pub struct WeeklyResetTimer;

fn pct_str(b: Option<&RateBucket>, prefix: &str) -> String {
    let pct = b.and_then(|x| x.used_percentage);
    pct.map_or_else(
        || format!("{prefix}: --%"),
        |p| format!("{prefix}: {p:.0}%"),
    )
}

fn timer_str(b: Option<&RateBucket>, now_ms: u64, long: bool) -> String {
    let resets_at = b.and_then(|x| x.resets_at);
    let Some(resets_at) = resets_at else {
        return "⏳ --".to_string();
    };
    let now_unix_s = i64::try_from(now_ms / 1000).unwrap_or(i64::MAX);
    let remaining = resets_at - now_unix_s;
    let body = if long {
        format_long(remaining)
    } else {
        format_short(remaining)
    };
    format!("⏳ {body}")
}

impl Widget for SessionUsage {
    fn id(&self) -> &'static str {
        "session-usage"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let rl = ctx.effective_rate_limits();
        Some(pct_str(rl.and_then(|r| r.five_hour.as_ref()), "5h"))
    }
}

impl Widget for WeeklyUsage {
    fn id(&self) -> &'static str {
        "weekly-usage"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let rl = ctx.effective_rate_limits();
        Some(pct_str(rl.and_then(|r| r.seven_day.as_ref()), "7d"))
    }
}

impl Widget for BlockResetTimer {
    fn id(&self) -> &'static str {
        "block-reset-timer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let rl = ctx.effective_rate_limits();
        Some(timer_str(
            rl.and_then(|r| r.five_hour.as_ref()),
            ctx.now_ms,
            false,
        ))
    }
}

impl Widget for WeeklyResetTimer {
    fn id(&self) -> &'static str {
        "weekly-reset-timer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let rl = ctx.effective_rate_limits();
        Some(timer_str(
            rl.and_then(|r| r.seven_day.as_ref()),
            ctx.now_ms,
            true,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::payload::RateLimits;
    use crate::widgets::test_helpers::payload_no_transcript;

    fn rl_with_buckets(
        five_pct: f64,
        five_resets: i64,
        weekly_pct: f64,
        weekly_resets: i64,
    ) -> RateLimits {
        RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(five_pct),
                resets_at: Some(five_resets),
            }),
            seven_day: Some(RateBucket {
                used_percentage: Some(weekly_pct),
                resets_at: Some(weekly_resets),
            }),
        }
    }

    /// Pre-populate `rate_limits` cell, чтобы `effective_rate_limits()` не лез
    /// ни в payload, ни в disk-cache, ни в `claude_account_email()`.
    fn ctx_with_rl<'a>(
        payload: &'a crate::types::payload::StatusPayload,
        settings: &'a crate::types::config::Settings,
        rl: Option<RateLimits>,
        now_ms: u64,
    ) -> RenderContext<'a> {
        let mut ctx = RenderContext::new(payload, settings);
        ctx.now_ms = now_ms;
        ctx.set_rate_limits_for_tests(rl);
        ctx
    }

    #[test]
    fn session_usage_renders_pct() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, Some(rl_with_buckets(1.0, 0, 6.0, 0)), 0);
        assert_eq!(SessionUsage.render(&ctx), Some("5h: 1%".into()));
    }

    #[test]
    fn weekly_usage_renders_pct() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, Some(rl_with_buckets(1.0, 0, 6.0, 0)), 0);
        assert_eq!(WeeklyUsage.render(&ctx), Some("7d: 6%".into()));
    }

    #[test]
    fn session_usage_renders_placeholder_without_data() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, None, 0);
        assert_eq!(SessionUsage.render(&ctx), Some("5h: --%".into()));
    }

    #[test]
    fn weekly_usage_renders_placeholder_without_data() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, None, 0);
        assert_eq!(WeeklyUsage.render(&ctx), Some("7d: --%".into()));
    }

    #[test]
    fn block_reset_timer_renders_short_format() {
        let now_ms = 1_700_000_000_000_u64;
        #[allow(clippy::cast_possible_wrap)]
        let resets_at_unix_s = (now_ms / 1000) as i64 + 4 * 3600 + 32 * 60;
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(
            &p,
            &s,
            Some(rl_with_buckets(1.0, resets_at_unix_s, 6.0, 0)),
            now_ms,
        );
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ 4h32m".into()));
    }

    #[test]
    fn weekly_reset_timer_renders_long_format() {
        let now_ms = 1_700_000_000_000_u64;
        #[allow(clippy::cast_possible_wrap)]
        let resets_at = (now_ms / 1000) as i64 + 5 * 86400 + 14 * 3600;
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(
            &p,
            &s,
            Some(rl_with_buckets(1.0, 0, 6.0, resets_at)),
            now_ms,
        );
        assert_eq!(WeeklyResetTimer.render(&ctx), Some("⏳ 5d 14h".into()));
    }

    #[test]
    fn reset_timer_in_past_returns_under_1m() {
        let now_ms = 1_700_000_000_000_u64;
        #[allow(clippy::cast_possible_wrap)]
        let resets_at = (now_ms / 1000) as i64 - 100;
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(
            &p,
            &s,
            Some(rl_with_buckets(1.0, resets_at, 6.0, 0)),
            now_ms,
        );
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ < 1m".into()));
    }

    #[test]
    fn reset_timer_renders_placeholder_without_resets_at() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let rl = RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(1.0),
                resets_at: None,
            }),
            seven_day: None,
        };
        let ctx = ctx_with_rl(&p, &s, Some(rl), 0);
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ --".into()));
    }

    #[test]
    fn block_reset_timer_renders_placeholder_without_data() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, None, 0);
        assert_eq!(BlockResetTimer.render(&ctx), Some("⏳ --".into()));
    }

    #[test]
    fn weekly_reset_timer_renders_placeholder_without_data() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, None, 0);
        assert_eq!(WeeklyResetTimer.render(&ctx), Some("⏳ --".into()));
    }

    #[test]
    fn percentage_floor_rounds_using_format_default() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = ctx_with_rl(&p, &s, Some(rl_with_buckets(1.9, 0, 6.7, 0)), 0);
        // {pct:.0} использует banker rounding → 1.9 → 2; 6.7 → 7.
        assert_eq!(SessionUsage.render(&ctx), Some("5h: 2%".into()));
        assert_eq!(WeeklyUsage.render(&ctx), Some("7d: 7%".into()));
    }
}
