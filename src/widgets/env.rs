//! Environment widgets — Phase 7 Task 9.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct ClaudeAccountEmail;
pub struct FreeMemory;

impl Widget for ClaudeAccountEmail {
    fn id(&self) -> &'static str { "claude-account-email" }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        crate::commands::env_loader::claude_account_email().map(str::to_string)
    }
}

impl Widget for FreeMemory {
    fn id(&self) -> &'static str { "free-memory" }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        let bytes = sys.available_memory();
        Some(format!("💾 {}", crate::util::format_memory::format(bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::test_helpers::payload_no_transcript;

    #[test]
    fn free_memory_returns_some_string() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = FreeMemory.render(&ctx).expect("system memory always present");
        assert!(out.starts_with("💾 "), "got: {out}");
        // суффикс: одна из b/k/M/G/T
        assert!(out.chars().last().is_some_and(|c| "bkMGT".contains(c)));
    }

    #[test]
    fn claude_account_email_reads_from_test_file() {
        // Hermetic: не используем глобальный OnceLock.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        std::fs::write(&path, r#"{"oauth_account":{"email_address":"u@x.com"}}"#).unwrap();
        let json = crate::commands::env_loader::read_from_path(&path).unwrap();
        assert_eq!(
            json.oauth_account.unwrap().email_address.as_deref(),
            Some("u@x.com")
        );
    }
}
