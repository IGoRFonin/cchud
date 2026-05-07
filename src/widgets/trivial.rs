//! Trivial single-field widgets.
//!
//! Каждый виджет читает одно поле payload (или одно env-значение) и
//! форматирует его минимально. Все impl следуют единой схеме:
//! `payload.foo.as_ref()?.bar.as_deref()?.into()`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct Version;

impl Widget for Version {
    fn id(&self) -> &'static str {
        "Version"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let v = ctx.payload.version.as_deref()?;
        if v.is_empty() {
            None
        } else {
            Some(v.to_string())
        }
    }
}

pub struct ClaudeSessionId;

impl Widget for ClaudeSessionId {
    fn id(&self) -> &'static str {
        "ClaudeSessionId"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let id = &ctx.payload.session_id;
        if id.is_empty() {
            return None;
        }
        let short: String = id.chars().take(8).collect();
        Some(short)
    }
}

pub struct TerminalWidth;

impl Widget for TerminalWidth {
    fn id(&self) -> &'static str {
        "TerminalWidth"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let (terminal_size::Width(w), _h) = terminal_size::terminal_size()?;
        Some(format!("{w}"))
    }
}

pub struct OutputStyle;

impl Widget for OutputStyle {
    fn id(&self) -> &'static str {
        "OutputStyle"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.output_style.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().italic()
    }
}

pub struct VimMode;

impl Widget for VimMode {
    fn id(&self) -> &'static str {
        "VimMode"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let mode = ctx.payload.vim.as_ref()?.mode.as_deref()?;
        if mode.is_empty() {
            None
        } else {
            Some(mode.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().bold()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{
        ModelInfo, OutputStyle as PayloadOutputStyle, StatusPayload, VimState, Workspace,
    };

    fn base_payload() -> StatusPayload {
        StatusPayload {
            session_id: "abcd1234-5678-9012-3456-789012345678".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
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
    ) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    #[test]
    fn version_renders_when_present() {
        let mut p = base_payload();
        p.version = Some("2.1.119".into());
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), Some("2.1.119".into()));
    }

    #[test]
    fn version_returns_none_when_missing() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn version_returns_none_when_empty_string() {
        let mut p = base_payload();
        p.version = Some(String::new());
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn claude_session_id_takes_first_eight_chars() {
        let p = base_payload();
        let s = default_line();
        // "abcd1234-5678-..." → "abcd1234"
        assert_eq!(
            ClaudeSessionId.render(&ctx_with(&p, &s)),
            Some("abcd1234".into())
        );
    }

    #[test]
    fn claude_session_id_returns_none_for_empty() {
        let mut p = base_payload();
        p.session_id = String::new();
        let s = default_line();
        assert_eq!(ClaudeSessionId.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn claude_session_id_handles_short_id() {
        let mut p = base_payload();
        p.session_id = "abc".into();
        let s = default_line();
        // < 8 символов — берём всё, что есть.
        assert_eq!(
            ClaudeSessionId.render(&ctx_with(&p, &s)),
            Some("abc".into())
        );
    }

    #[test]
    fn terminal_width_returns_none_under_cargo_test() {
        let p = base_payload();
        let s = default_line();
        // cargo test обычно НЕ имеет TTY; ожидаем None.
        // Если CI прогоняет под TTY (редкий случай) — тест пройдёт и для Some(_),
        // потому проверяем "либо None, либо Some(>0)".
        let out = TerminalWidth.render(&ctx_with(&p, &s));
        match out {
            None => {} // expected path under cargo test
            Some(s) => {
                let n: u32 = s.parse().expect("width must be numeric");
                assert!(n > 0, "if Some, width must be > 0");
            }
        }
    }

    #[test]
    fn output_style_renders_name() {
        let mut p = base_payload();
        p.output_style = Some(PayloadOutputStyle {
            name: Some("default".into()),
        });
        let s = default_line();
        assert_eq!(
            OutputStyle.render(&ctx_with(&p, &s)),
            Some("default".into())
        );
    }

    #[test]
    fn output_style_returns_none_without_field() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(OutputStyle.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn output_style_returns_none_with_empty_name() {
        let mut p = base_payload();
        p.output_style = Some(PayloadOutputStyle {
            name: Some(String::new()),
        });
        let s = default_line();
        assert_eq!(OutputStyle.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn vim_mode_renders_when_present() {
        let mut p = base_payload();
        p.vim = Some(VimState {
            mode: Some("INSERT".into()),
        });
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), Some("INSERT".into()));
    }

    #[test]
    fn vim_mode_returns_none_when_disabled() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn vim_mode_returns_none_for_empty_object() {
        let mut p = base_payload();
        p.vim = Some(VimState { mode: None });
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), None);
    }
}
