//! Static-text cluster — Phase 3 Task 2.
//!
//! Виджеты этого модуля не читают payload; они рендерят буквальный текст
//! из конфига. `Link` использует OSC 8 hyperlink escape-последовательность
//! (без detect terminal capability — Phase 7 добавит graceful fallback).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::config::{CustomSymbolParams, CustomTextParams, LinkParams};
use crate::widgets::{RenderContext, Widget};

pub struct CustomText {
    pub params: CustomTextParams,
}

impl Widget for CustomText {
    fn id(&self) -> &'static str {
        "CustomText"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.text.is_empty() {
            None
        } else {
            Some(self.params.text.clone())
        }
    }
}

pub struct CustomSymbol {
    pub params: CustomSymbolParams,
}

impl Widget for CustomSymbol {
    fn id(&self) -> &'static str {
        "CustomSymbol"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.symbol.is_empty() {
            None
        } else {
            Some(self.params.symbol.clone())
        }
    }
}

pub struct Link {
    pub params: LinkParams,
}

impl Widget for Link {
    fn id(&self) -> &'static str {
        "Link"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        if self.params.url.is_empty() {
            return None;
        }
        let label = self.params.label.as_deref().unwrap_or(&self.params.url);
        // OSC 8 hyperlink: ESC ] 8 ; ; URL ST  TEXT  ESC ] 8 ; ; ST
        // ST (string terminator) = ESC \ (0x1b 0x5c).
        Some(format!(
            "\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\",
            url = self.params.url,
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn empty_payload() -> StatusPayload {
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

    #[test]
    fn custom_text_renders_literal() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomText {
            params: CustomTextParams {
                text: "hello".into(),
            },
        };
        assert_eq!(w.render(&ctx), Some("hello".into()));
    }

    #[test]
    fn custom_text_returns_none_for_empty() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomText {
            params: CustomTextParams {
                text: String::new(),
            },
        };
        assert_eq!(w.render(&ctx), None);
    }

    #[test]
    fn custom_symbol_renders_literal() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomSymbol {
            params: CustomSymbolParams {
                symbol: "★".into()
            },
        };
        assert_eq!(w.render(&ctx), Some("★".into()));
    }

    #[test]
    fn custom_symbol_returns_none_for_empty() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = CustomSymbol {
            params: CustomSymbolParams {
                symbol: String::new(),
            },
        };
        assert_eq!(w.render(&ctx), None);
    }

    #[test]
    fn link_emits_osc8_with_label() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: "https://example.com".into(),
                label: Some("Example".into()),
            },
        };
        let out = w.render(&ctx).unwrap();
        // Проверяем точные байты OSC 8: ESC ] 8 ; ; URL ESC \ TEXT ESC ] 8 ; ; ESC \
        assert_eq!(
            out,
            "\u{1b}]8;;https://example.com\u{1b}\\Example\u{1b}]8;;\u{1b}\\"
        );
        // И семантические проверки:
        assert!(out.starts_with("\u{1b}]8;;"));
        assert!(out.contains("Example"));
        assert!(out.ends_with("\u{1b}]8;;\u{1b}\\"));
    }

    #[test]
    fn link_falls_back_to_url_when_label_absent() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: "https://x.com".into(),
                label: None,
            },
        };
        let out = w.render(&ctx).unwrap();
        // label секция = url
        assert_eq!(
            out,
            "\u{1b}]8;;https://x.com\u{1b}\\https://x.com\u{1b}]8;;\u{1b}\\"
        );
    }

    #[test]
    fn link_returns_none_for_empty_url() {
        let p = empty_payload();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let w = Link {
            params: LinkParams {
                url: String::new(),
                label: Some("ignored".into()),
            },
        };
        assert_eq!(w.render(&ctx), None);
    }
}
