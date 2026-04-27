//! Git head cluster — Phase 5 Task 2.
//!
//! Тонкие getter'ы над `GitInfo::head` и `GitInfo::root_dir` (T1).
//! Виджеты возвращают None если cwd вне git-репо или HEAD detached/unborn.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitBranch;
pub struct GitSha;
pub struct GitRootDir;

impl Widget for GitBranch {
    fn id(&self) -> &'static str {
        "GitBranch"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        ctx.git()?.head.branch.clone()
    }
}

impl Widget for GitSha {
    fn id(&self) -> &'static str {
        "GitSha"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        ctx.git()?.head.short_sha()
    }
}

impl Widget for GitRootDir {
    fn id(&self) -> &'static str {
        "GitRootDir"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let root = &ctx.git()?.root_dir;
        root.file_name().and_then(|n| n.to_str()).map(String::from)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::git::fixture::GitFixture;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_cwd(cwd: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
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
    fn git_branch_returns_main_in_fresh_repo() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx).as_deref(), Some("main"));
    }

    #[test]
    fn git_branch_returns_none_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload_with_cwd(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx), None);
    }

    #[test]
    fn git_branch_returns_none_for_detached_head() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.git(&["checkout", "--detach", "HEAD"]);
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx), None);
    }

    #[test]
    fn git_sha_returns_seven_chars() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let sha = GitSha.render(&ctx).expect("must have sha");
        assert_eq!(sha.len(), 7);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn git_sha_returns_none_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload_with_cwd(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitSha.render(&ctx), None);
    }

    #[test]
    fn git_root_dir_returns_repo_basename() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let root = GitRootDir.render(&ctx).expect("must have root");
        assert!(!root.is_empty());
        assert_eq!(root, f.path().file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn git_root_dir_works_from_subdirectory() {
        let f = GitFixture::new();
        f.write_file("nested/deep/file.txt", "x");
        let nested = f.path().join("nested/deep");
        let p = payload_with_cwd(nested.to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(
            GitRootDir.render(&ctx).as_deref(),
            Some(f.path().file_name().unwrap().to_str().unwrap())
        );
    }
}
