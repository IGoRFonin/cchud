//! Worktree cluster.
//!
//! Пять виджетов читают `payload.worktree: Option<Worktree>`. CC шлёт
//! поле только если активная сессия идёт в git-worktree (`git worktree add`).
//! Реальные семплы worktree не содержат — тесты используют synthetic-семпл
//! `payload-synthetic-vim-worktree.json` + struct-литералы.
//!
//! `Worktree` (без суффикса) и `WorktreeName` дают один и тот же контент;
//! upstream ccstatusline даёт оба, паритет требует обоих.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct Worktree;

impl Widget for Worktree {
    fn id(&self) -> &'static str {
        "Worktree"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.worktree.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().dim()
    }
}

pub struct WorktreeMode;

impl Widget for WorktreeMode {
    fn id(&self) -> &'static str {
        "WorktreeMode"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        if ctx.payload.worktree.is_some() {
            Some("WT".to_string())
        } else {
            None
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().dim()
    }
}

pub struct WorktreeName;

impl Widget for WorktreeName {
    fn id(&self) -> &'static str {
        "WorktreeName"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.worktree.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().dim()
    }
}

pub struct WorktreeBranch;

impl Widget for WorktreeBranch {
    fn id(&self) -> &'static str {
        "WorktreeBranch"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let b = ctx.payload.worktree.as_ref()?.branch.as_deref()?;
        if b.is_empty() {
            None
        } else {
            Some(b.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().dim()
    }
}

pub struct WorktreeOriginalBranch;

impl Widget for WorktreeOriginalBranch {
    fn id(&self) -> &'static str {
        "WorktreeOriginalBranch"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let b = ctx.payload.worktree.as_ref()?.original_branch.as_deref()?;
        if b.is_empty() {
            None
        } else {
            Some(b.to_string())
        }
    }
    fn default_style(&self) -> crate::render::Style {
        crate::render::Style::none().dim()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace, Worktree as WorktreePayload};

    const SYNTHETIC_SAMPLE: &str =
        include_str!("../../benches/samples/payload-synthetic-vim-worktree.json");

    fn payload_with_worktree(wt: Option<WorktreePayload>) -> StatusPayload {
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
            worktree: wt,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn full_worktree() -> WorktreePayload {
        WorktreePayload {
            name: Some("wt-feature".into()),
            path: Some("/tmp/wt-feature".into()),
            branch: Some("feature/x".into()),
            original_cwd: Some("/tmp/main".into()),
            original_branch: Some("main".into()),
        }
    }

    fn ctx_with<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
    ) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    // ─── Worktree (alias for name) ──────────────────────────────

    #[test]
    fn worktree_renders_name() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            Worktree.render(&ctx_with(&p, &s)),
            Some("wt-feature".into())
        );
    }

    #[test]
    fn worktree_returns_none_without_field() {
        let p = payload_with_worktree(None);
        let s = default_line();
        assert_eq!(Worktree.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeMode ───────────────────────────────────────────

    #[test]
    fn worktree_mode_returns_wt_flag_when_active() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), Some("WT".into()));
    }

    #[test]
    fn worktree_mode_returns_none_when_inactive() {
        let p = payload_with_worktree(None);
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn worktree_mode_returns_wt_even_for_empty_worktree_object() {
        // worktree: {} — все поля None, но envelope-объект есть.
        // Mode реагирует на наличие объекта, а не на содержимое.
        let p = payload_with_worktree(Some(WorktreePayload {
            name: None,
            path: None,
            branch: None,
            original_cwd: None,
            original_branch: None,
        }));
        let s = default_line();
        assert_eq!(WorktreeMode.render(&ctx_with(&p, &s)), Some("WT".into()));
    }

    // ─── WorktreeName ───────────────────────────────────────────

    #[test]
    fn worktree_name_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeName.render(&ctx_with(&p, &s)),
            Some("wt-feature".into())
        );
    }

    #[test]
    fn worktree_name_returns_none_without_name_field() {
        let mut wt = full_worktree();
        wt.name = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeName.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeBranch ─────────────────────────────────────────

    #[test]
    fn worktree_branch_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeBranch.render(&ctx_with(&p, &s)),
            Some("feature/x".into())
        );
    }

    #[test]
    fn worktree_branch_returns_none_without_branch() {
        let mut wt = full_worktree();
        wt.branch = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeBranch.render(&ctx_with(&p, &s)), None);
    }

    // ─── WorktreeOriginalBranch ─────────────────────────────────

    #[test]
    fn worktree_original_branch_renders() {
        let p = payload_with_worktree(Some(full_worktree()));
        let s = default_line();
        assert_eq!(
            WorktreeOriginalBranch.render(&ctx_with(&p, &s)),
            Some("main".into())
        );
    }

    #[test]
    fn worktree_original_branch_returns_none_without_field() {
        let mut wt = full_worktree();
        wt.original_branch = None;
        let p = payload_with_worktree(Some(wt));
        let s = default_line();
        assert_eq!(WorktreeOriginalBranch.render(&ctx_with(&p, &s)), None);
    }

    // ─── Integration с synthetic-семплом ────────────────────────

    #[test]
    fn synthetic_sample_renders_full_cluster() {
        let p: StatusPayload =
            serde_json::from_str(SYNTHETIC_SAMPLE).expect("synthetic sample must parse");
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(Worktree.render(&ctx), Some("wt-feature".into()));
        assert_eq!(WorktreeMode.render(&ctx), Some("WT".into()));
        assert_eq!(WorktreeName.render(&ctx), Some("wt-feature".into()));
        assert_eq!(
            WorktreeBranch.render(&ctx),
            Some("feature/synthetic".into())
        );
        assert_eq!(WorktreeOriginalBranch.render(&ctx), Some("main".into()));
    }
}
