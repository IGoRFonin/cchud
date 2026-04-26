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
}
