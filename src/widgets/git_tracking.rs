#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitAheadBehind;

impl Widget for GitAheadBehind {
    fn id(&self) -> &'static str {
        "GitAheadBehind"
    }

    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.git()?.tracking()?;
        if t.ahead == 0 && t.behind == 0 {
            return None;
        }
        let mut s = String::with_capacity(8);
        if t.ahead > 0 {
            s.push('↑');
            s.push_str(&t.ahead.to_string());
        }
        if t.behind > 0 {
            s.push('↓');
            s.push_str(&t.behind.to_string());
        }
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::git::fixture::GitFixture;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload(cwd: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "m".into(),
                display_name: "M".into(),
            },
            workspace: Workspace {
                current_dir: cwd.into(),
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
    fn no_upstream_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx), None);
    }

    #[test]
    fn ahead_only_renders_arrow_up() {
        let f = GitFixture::new();
        let bare = tempfile::tempdir().unwrap();
        let bare_path = bare.path().to_str().unwrap();
        f.git(&["init", "--bare", bare_path]);
        f.add_remote("origin", bare_path);
        f.git(&["push", "-u", "origin", "main"]);
        f.write_file("a.txt", "x");
        f.git(&["add", "a.txt"]);
        f.commit("c2");

        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx).as_deref(), Some("↑1"));
    }

    #[test]
    fn outside_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx), None);
    }
}
