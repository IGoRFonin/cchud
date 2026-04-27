use crate::widgets::{RenderContext, Widget};

pub struct GitStatus;
pub struct GitChanges;
pub struct GitStaged;
pub struct GitUnstaged;
pub struct GitUntracked;
pub struct GitConflicts;

fn nonzero(n: u32) -> Option<String> {
    if n == 0 { None } else { Some(n.to_string()) }
}

impl Widget for GitStatus {
    fn id(&self) -> &'static str {
        "GitStatus"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let s = ctx.git()?.status_counts()?;
        if s.total() == 0 {
            return None;
        }
        let mut parts: Vec<String> = Vec::with_capacity(4);
        if s.staged > 0 {
            parts.push(format!("M{}", s.staged));
        }
        if s.unstaged > 0 {
            parts.push(format!("~{}", s.unstaged));
        }
        if s.untracked > 0 {
            parts.push(format!("?{}", s.untracked));
        }
        if s.conflicts > 0 {
            parts.push(format!("✗{}", s.conflicts));
        }
        Some(parts.join(" "))
    }
}

impl Widget for GitChanges {
    fn id(&self) -> &'static str {
        "GitChanges"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.total())
    }
}

impl Widget for GitStaged {
    fn id(&self) -> &'static str {
        "GitStaged"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.staged)
    }
}

impl Widget for GitUnstaged {
    fn id(&self) -> &'static str {
        "GitUnstaged"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.unstaged)
    }
}

impl Widget for GitUntracked {
    fn id(&self) -> &'static str {
        "GitUntracked"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.untracked)
    }
}

impl Widget for GitConflicts {
    fn id(&self) -> &'static str {
        "GitConflicts"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.conflicts)
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
    fn clean_repo_returns_none_for_all_status_widgets() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitStatus.render(&ctx), None);
        assert_eq!(GitChanges.render(&ctx), None);
        assert_eq!(GitStaged.render(&ctx), None);
        assert_eq!(GitUnstaged.render(&ctx), None);
        assert_eq!(GitUntracked.render(&ctx), None);
        assert_eq!(GitConflicts.render(&ctx), None);
    }

    #[test]
    fn untracked_file_lights_up_untracked_and_changes_only() {
        let f = GitFixture::new();
        f.write_file("new.txt", "x");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitUntracked.render(&ctx).as_deref(), Some("1"));
        assert_eq!(GitChanges.render(&ctx).as_deref(), Some("1"));
        assert_eq!(GitStaged.render(&ctx), None);
        assert_eq!(GitUnstaged.render(&ctx), None);
        assert_eq!(GitConflicts.render(&ctx), None);
        assert_eq!(GitStatus.render(&ctx).as_deref(), Some("?1"));
    }

    #[test]
    fn staged_and_unstaged_combine_in_summary() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "2");
        f.git(&["add", "a.txt"]);
        f.write_file("a.txt", "3");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let summary = GitStatus.render(&ctx).expect("must render");
        assert!(summary.contains("M1"));
        assert!(summary.contains("~1"));
    }

    #[test]
    fn outside_repo_all_status_widgets_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitStatus.render(&ctx), None);
        assert_eq!(GitChanges.render(&ctx), None);
    }
}
