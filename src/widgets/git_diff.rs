use crate::widgets::{RenderContext, Widget};

pub struct GitInsertions;
pub struct GitDeletions;

impl Widget for GitInsertions {
    fn id(&self) -> &'static str {
        "GitInsertions"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let n = ctx.git()?.diff_stat()?.insertions;
        if n == 0 { None } else { Some(format!("+{n}")) }
    }
}

impl Widget for GitDeletions {
    fn id(&self) -> &'static str {
        "GitDeletions"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let n = ctx.git()?.diff_stat()?.deletions;
        if n == 0 { None } else { Some(format!("-{n}")) }
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
    fn insertions_renders_plus_prefix() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n2\n3\n");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx).as_deref(), Some("+2"));
        assert_eq!(GitDeletions.render(&ctx), None);
    }

    #[test]
    fn deletions_renders_minus_prefix() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n2\n3\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitDeletions.render(&ctx).as_deref(), Some("-2"));
        assert_eq!(GitInsertions.render(&ctx), None);
    }

    #[test]
    fn zero_diff_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx), None);
        assert_eq!(GitDeletions.render(&ctx), None);
    }

    #[test]
    fn outside_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx), None);
        assert_eq!(GitDeletions.render(&ctx), None);
    }
}
