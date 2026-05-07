//! Git remote widgets.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::git::GitInfo;
use crate::widgets::{RenderContext, Widget};

pub struct GitOriginOwner;
pub struct GitOriginRepo;
pub struct GitOriginOwnerRepo;
pub struct GitUpstreamOwner;
pub struct GitUpstreamRepo;
pub struct GitUpstreamOwnerRepo;
pub struct GitIsFork;

fn remote_owner(g: &GitInfo, name: &str) -> Option<String> {
    g.remotes.get(name)?.owner.clone()
}

fn remote_repo(g: &GitInfo, name: &str) -> Option<String> {
    g.remotes.get(name)?.repo.clone()
}

fn remote_owner_repo(g: &GitInfo, name: &str) -> Option<String> {
    let r = g.remotes.get(name)?;
    let owner = r.owner.as_deref()?;
    let repo = r.repo.as_deref()?;
    Some(format!("{owner}/{repo}"))
}

impl Widget for GitOriginOwner {
    fn id(&self) -> &'static str {
        "GitOriginOwner"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner(ctx.git()?, "origin")
    }
}
impl Widget for GitOriginRepo {
    fn id(&self) -> &'static str {
        "GitOriginRepo"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_repo(ctx.git()?, "origin")
    }
}
impl Widget for GitOriginOwnerRepo {
    fn id(&self) -> &'static str {
        "GitOriginOwnerRepo"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner_repo(ctx.git()?, "origin")
    }
}
impl Widget for GitUpstreamOwner {
    fn id(&self) -> &'static str {
        "GitUpstreamOwner"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner(ctx.git()?, "upstream")
    }
}
impl Widget for GitUpstreamRepo {
    fn id(&self) -> &'static str {
        "GitUpstreamRepo"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_repo(ctx.git()?, "upstream")
    }
}
impl Widget for GitUpstreamOwnerRepo {
    fn id(&self) -> &'static str {
        "GitUpstreamOwnerRepo"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner_repo(ctx.git()?, "upstream")
    }
}
impl Widget for GitIsFork {
    fn id(&self) -> &'static str {
        "GitIsFork"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let git = ctx.git()?;
        let origin_owner = git.remotes.get("origin")?.owner.as_deref()?;
        let upstream_owner = git.remotes.get("upstream")?.owner.as_deref()?;
        (origin_owner != upstream_owner).then(|| "fork".into())
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
    fn origin_owner_repo_from_ssh_url() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitOriginOwner.render(&ctx).as_deref(), Some("foo"));
        assert_eq!(GitOriginRepo.render(&ctx).as_deref(), Some("bar"));
        assert_eq!(GitOriginOwnerRepo.render(&ctx).as_deref(), Some("foo/bar"));
    }

    #[test]
    fn upstream_owner_from_https_url() {
        let f = GitFixture::new();
        f.add_remote("upstream", "https://github.com/baz/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitUpstreamOwner.render(&ctx).as_deref(), Some("baz"));
        assert_eq!(GitUpstreamRepo.render(&ctx).as_deref(), Some("bar"));
        assert_eq!(
            GitUpstreamOwnerRepo.render(&ctx).as_deref(),
            Some("baz/bar")
        );
    }

    #[test]
    fn no_remotes_yields_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitOriginOwner.render(&ctx), None);
        assert_eq!(GitUpstreamOwner.render(&ctx), None);
        assert_eq!(GitIsFork.render(&ctx), None);
    }

    #[test]
    fn is_fork_when_origin_differs_from_upstream() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        f.add_remote("upstream", "git@github.com:them/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx).as_deref(), Some("fork"));
    }

    #[test]
    fn is_fork_none_when_owners_match() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        f.add_remote("upstream", "git@github.com:me/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx), None);
    }

    #[test]
    fn is_fork_none_with_only_origin() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx), None);
    }
}
