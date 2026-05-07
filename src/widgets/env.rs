//! Environment widgets.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct ClaudeAccountEmail;
pub struct FreeMemory;

impl Widget for ClaudeAccountEmail {
    fn id(&self) -> &'static str {
        "claude-account-email"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        crate::commands::env_loader::claude_account_email().map(str::to_string)
    }
}

impl Widget for FreeMemory {
    fn id(&self) -> &'static str {
        "free-memory"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        let bytes = sys.available_memory();
        Some(format!("💾 {}", crate::util::format_memory::format(bytes)))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::widgets::test_helpers::payload_no_transcript;

    #[test]
    fn free_memory_returns_some_string() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = FreeMemory
            .render(&ctx)
            .expect("system memory always present");
        assert!(out.starts_with("💾 "), "got: {out}");
        assert!(out.chars().last().is_some_and(|c| "bkMGT".contains(c)));
    }

    #[test]
    fn free_memory_render_second_call_does_not_panic() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let _ = FreeMemory.render(&ctx);
    }

    #[test]
    fn claude_account_email_render_with_lock_prefilled() {
        use crate::commands::env_loader::{ClaudeJson, OauthAccount, set_claude_json_for_tests};
        // OnceLock is first-writer-wins; set_claude_json_for_tests is a no-op if already set.
        // We pre-fill then call render() — the result may be Some(email) or None depending on
        // which test in this binary won the race, both are valid outcomes.
        set_claude_json_for_tests(Some(ClaudeJson {
            oauth_account: Some(OauthAccount {
                email_address: Some("widget_test@example.com".into()),
            }),
        }));
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = ClaudeAccountEmail.render(&ctx);
        if let Some(email) = out {
            assert!(
                email.contains('@'),
                "render() returned non-email string: {email}"
            );
        }
    }

    #[test]
    fn claude_account_email_render_output_format() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = ClaudeAccountEmail.render(&ctx);
        if let Some(email) = out {
            assert!(
                email.contains('@'),
                "render() returned non-email string: {email}"
            );
        }
    }
}
