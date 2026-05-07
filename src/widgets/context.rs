//! Context-window cluster.
//!
//! Шесть виджетов читают `payload.context_window: Option<ContextWindowInfo>`.
//! `ContextPercentageUsable` дополнительно зависит
//! от `payload.model.id` через `util::model_context_size::max_tokens_for`.
//! `ContextBar` использует `util::ascii_bar::render` + `params.width`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::{
    config::ContextBarParams,
    payload::{ContextWindowInfo, CurrentUsage},
};
use crate::util::{ascii_bar, format_tokens::format_tokens, model_context_size};
use crate::widgets::{RenderContext, Widget};

#[derive(Default)]
pub struct ContextLength {
    pub raw_value: bool,
}

impl Widget for ContextLength {
    fn id(&self) -> &'static str {
        "ContextLength"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let total = total_tokens(cw)?;
        let formatted = format_tokens(total);
        Some(if self.raw_value {
            formatted
        } else {
            format!("Ctx: {formatted}")
        })
    }
}

pub struct ContextPercentage;

impl Widget for ContextPercentage {
    fn id(&self) -> &'static str {
        "ContextPercentage"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let pct = cw.used_percentage?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = pct.round() as u64;
        Some(format!("{n}%"))
    }
}

pub struct ContextPercentageUsable;

impl Widget for ContextPercentageUsable {
    fn id(&self) -> &'static str {
        "ContextPercentageUsable"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let total = total_tokens(cw)?;
        let max = model_context_size::max_tokens_for(&ctx.payload.model.id)?;
        if max == 0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let pct = (total as f64) / (max as f64) * 100.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = pct.round() as u64;
        Some(format!("{n}%"))
    }
}

pub struct ContextBar {
    pub params: ContextBarParams,
}

impl Widget for ContextBar {
    fn id(&self) -> &'static str {
        "ContextBar"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let pct = cw.used_percentage?;
        Some(ascii_bar::render(pct, self.params.width))
    }
}

#[derive(Default)]
pub struct TokensInput {
    pub raw_value: bool,
}

impl Widget for TokensInput {
    fn id(&self) -> &'static str {
        "TokensInput"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let usage = cw.current_usage.as_ref()?;
        match usage {
            CurrentUsage::Detailed { input_tokens, .. } => {
                let v = input_tokens.as_ref()?;
                let formatted = format_tokens(*v);
                Some(if self.raw_value {
                    formatted
                } else {
                    format!("In: {formatted}")
                })
            }
            CurrentUsage::Total(_) => None,
        }
    }
}

#[derive(Default)]
pub struct TokensOutput {
    pub raw_value: bool,
}

impl Widget for TokensOutput {
    fn id(&self) -> &'static str {
        "TokensOutput"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let cw = ctx.payload.context_window.as_ref()?;
        let usage = cw.current_usage.as_ref()?;
        match usage {
            CurrentUsage::Detailed { output_tokens, .. } => {
                let v = output_tokens.as_ref()?;
                let formatted = format_tokens(*v);
                Some(if self.raw_value {
                    formatted
                } else {
                    format!("Out: {formatted}")
                })
            }
            CurrentUsage::Total(_) => None,
        }
    }
}

fn total_tokens(cw: &ContextWindowInfo) -> Option<u64> {
    let inp = cw.total_input_tokens?;
    let out = cw.total_output_tokens?;
    Some(inp + out)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_cw(cw: Option<ContextWindowInfo>, model_id: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: model_id.into(),
                display_name: "Sonnet 4.6".into(),
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
            context_window: cw,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn cw_full() -> ContextWindowInfo {
        ContextWindowInfo {
            context_window_size: Some(200_000),
            total_input_tokens: Some(399),
            total_output_tokens: Some(7062),
            current_usage: Some(CurrentUsage::Detailed {
                input_tokens: Some(1),
                output_tokens: Some(232),
                cache_creation_input_tokens: Some(241),
                cache_read_input_tokens: Some(49543),
            }),
            used_percentage: Some(25.0),
            remaining_percentage: Some(75.0),
        }
    }

    fn ctx_with<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
    ) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    // ─── ContextLength ──────────────────────────────────────────

    #[test]
    fn context_length_renders_with_prefix_and_compact_format() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        // 399 + 7062 = 7461 → "7.5k"
        assert_eq!(
            ContextLength::default().render(&ctx_with(&p, &s)),
            Some("Ctx: 7.5k".into())
        );
    }

    #[test]
    fn context_length_raw_drops_prefix() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        let w = ContextLength { raw_value: true };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("7.5k".into()));
    }

    #[test]
    fn context_length_returns_none_without_cw() {
        let p = payload_with_cw(None, "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(ContextLength::default().render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn context_length_returns_none_with_partial_tokens() {
        let mut cw = cw_full();
        cw.total_output_tokens = None;
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(ContextLength::default().render(&ctx_with(&p, &s)), None);
    }

    // ─── ContextPercentage ──────────────────────────────────────

    #[test]
    fn context_percentage_renders_rounded() {
        let mut cw = cw_full();
        cw.used_percentage = Some(25.7);
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(
            ContextPercentage.render(&ctx_with(&p, &s)),
            Some("26%".into())
        );
    }

    #[test]
    fn context_percentage_none_without_pct_field() {
        let mut cw = cw_full();
        cw.used_percentage = None;
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(ContextPercentage.render(&ctx_with(&p, &s)), None);
    }

    // ─── ContextPercentageUsable ────────────────────────────────

    #[test]
    fn context_percentage_usable_for_known_model() {
        // 7461 / 200_000 ≈ 3.7305 → round = 4
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(
            ContextPercentageUsable.render(&ctx_with(&p, &s)),
            Some("4%".into())
        );
    }

    #[test]
    fn context_percentage_usable_none_for_unknown_model() {
        let p = payload_with_cw(Some(cw_full()), "gpt-4o");
        let s = default_line();
        assert_eq!(ContextPercentageUsable.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn context_percentage_usable_none_without_tokens() {
        let mut cw = cw_full();
        cw.total_input_tokens = None;
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(ContextPercentageUsable.render(&ctx_with(&p, &s)), None);
    }

    // ─── ContextBar ─────────────────────────────────────────────

    #[test]
    fn context_bar_default_width() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        let w = ContextBar {
            params: ContextBarParams { width: 10 },
        };
        // 25% of 10 → 3 filled (round)
        // round(2.5) = 3.0 в Rust (round-half-away-from-zero).
        let out = w.render(&ctx_with(&p, &s)).unwrap();
        assert!(out.starts_with('['));
        assert!(out.ends_with(']'));
        assert_eq!(out.chars().filter(|c| *c == '█').count(), 3);
        assert_eq!(out.chars().filter(|c| *c == '░').count(), 7);
    }

    #[test]
    fn context_bar_custom_width() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        let w = ContextBar {
            params: ContextBarParams { width: 4 },
        };
        // 25% of 4 → exactly 1 filled
        let out = w.render(&ctx_with(&p, &s)).unwrap();
        assert_eq!(out, "[█░░░]");
    }

    #[test]
    fn context_bar_returns_none_without_pct() {
        let mut cw = cw_full();
        cw.used_percentage = None;
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        let w = ContextBar {
            params: ContextBarParams { width: 10 },
        };
        assert_eq!(w.render(&ctx_with(&p, &s)), None);
    }

    // ─── TokensInput / TokensOutput ─────────────────────────────

    #[test]
    fn tokens_input_renders_detailed() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(
            TokensInput::default().render(&ctx_with(&p, &s)),
            Some("In: 1".into())
        );
    }

    #[test]
    fn tokens_input_raw_drops_prefix() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        let w = TokensInput { raw_value: true };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("1".into()));
    }

    #[test]
    fn tokens_input_returns_none_for_total_form() {
        let mut cw = cw_full();
        cw.current_usage = Some(CurrentUsage::Total(12345));
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(TokensInput::default().render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn tokens_output_renders_detailed() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(
            TokensOutput::default().render(&ctx_with(&p, &s)),
            Some("Out: 232".into())
        );
    }

    #[test]
    fn tokens_output_raw_drops_prefix() {
        let p = payload_with_cw(Some(cw_full()), "claude-sonnet-4-6");
        let s = default_line();
        let w = TokensOutput { raw_value: true };
        assert_eq!(w.render(&ctx_with(&p, &s)), Some("232".into()));
    }

    #[test]
    fn tokens_output_returns_none_for_total_form() {
        let mut cw = cw_full();
        cw.current_usage = Some(CurrentUsage::Total(12345));
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(TokensOutput::default().render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn tokens_input_none_when_input_field_missing() {
        let mut cw = cw_full();
        cw.current_usage = Some(CurrentUsage::Detailed {
            input_tokens: None,
            output_tokens: Some(232),
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        });
        let p = payload_with_cw(Some(cw), "claude-sonnet-4-6");
        let s = default_line();
        assert_eq!(TokensInput::default().render(&ctx_with(&p, &s)), None);
    }
}
