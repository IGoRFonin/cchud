//! `GitPr` widget.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::git::pr;
use crate::widgets::{RenderContext, Widget};

pub struct GitPr;

impl Widget for GitPr {
    fn id(&self) -> &'static str {
        "GitPr"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let git = ctx.git()?;
        let origin = git.remotes.get("origin")?;
        let owner = origin.owner.as_deref()?;
        let repo = origin.repo.as_deref()?;
        let branch = git.head.branch.as_deref()?;
        let info = pr::lookup_or_fetch(owner, repo, branch)?;
        Some(format!("PR #{}", info.number))
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
    fn no_origin_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitPr.render(&ctx), None);
    }

    #[test]
    fn no_branch_returns_none() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        f.write_file("a.txt", "x");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.git(&["checkout", "--detach", "HEAD"]);
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitPr.render(&ctx), None);
    }

    // Note: интеграционный тест GitPr с моком api.github.com сложен —
    // lookup_or_fetch хардкодит "https://api.github.com". Тесты в pr::tests
    // покрывают fetch через lookup_or_fetch_with_base. Этот widget — тонкая обёртка.
}
