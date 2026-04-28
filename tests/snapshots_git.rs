//! Phase 5 git scenario snapshot tests.
//!
//! Покрывает 5 канонических git-состояний: clean, dirty, conflicts,
//! fork, detached HEAD. Каждое — через GitFixture + 20-widget config.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cchud::git::fixture::GitFixture;
use cchud::types::config::Settings;
use cchud::types::payload::{ModelInfo, StatusPayload, Workspace};
use cchud::widgets::{RenderContext, build_widgets};

fn payload(cwd: &str) -> StatusPayload {
    StatusPayload {
        session_id: "snap".into(),
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

/// 20-widget conf — все git-виджеты Phase 5 (без git-pr — сетевой).
fn full_git_settings() -> Settings {
    let json = r#"{
      "lines": [{
        "widgets": [
          { "type": "git-branch" },
          { "type": "git-sha" },
          { "type": "git-root-dir" },
          { "type": "git-status" },
          { "type": "git-changes" },
          { "type": "git-staged" },
          { "type": "git-unstaged" },
          { "type": "git-untracked" },
          { "type": "git-conflicts" },
          { "type": "git-insertions" },
          { "type": "git-deletions" },
          { "type": "git-ahead-behind" },
          { "type": "git-origin-owner" },
          { "type": "git-origin-repo" },
          { "type": "git-origin-owner-repo" },
          { "type": "git-upstream-owner" },
          { "type": "git-upstream-repo" },
          { "type": "git-upstream-owner-repo" },
          { "type": "git-is-fork" }
        ]
      }]
    }"#;
    serde_json::from_str(json).unwrap()
}

fn render_all(payload: &StatusPayload, settings: &Settings) -> Vec<String> {
    let ctx = RenderContext::new(payload, settings);
    let widgets = build_widgets(settings);
    widgets
        .iter()
        .map(|w| {
            let id = w.id();
            let val = w.render(&ctx).unwrap_or_default();
            // Normalize volatile values so snapshots are stable across runs.
            let val = match id {
                "GitSha" if !val.is_empty() => "[SHA]".into(),
                "GitRootDir" if !val.is_empty() => "[TMPDIR]".into(),
                _ => val,
            };
            format!("{id} = {val}")
        })
        .collect()
}

#[test]
fn snapshot_clean_repo() {
    let f = GitFixture::new();
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("clean", out);
}

#[test]
fn snapshot_dirty_repo() {
    let f = GitFixture::new();
    f.write_file("staged.txt", "1\n");
    f.git(&["add", "staged.txt"]);
    f.write_file("unstaged.txt", "u\n");
    f.git(&["add", "unstaged.txt"]);
    f.commit("c2");
    f.write_file("unstaged.txt", "u2\n");
    f.write_file("untracked.txt", "x");
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("dirty", out);
}

#[test]
fn snapshot_conflicts_repo() {
    let f = GitFixture::new();
    f.write_file("a.txt", "main\n");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.git(&["checkout", "-b", "feature"]);
    f.write_file("a.txt", "feature\n");
    f.git(&["add", "a.txt"]);
    f.commit("on feature");
    f.git(&["checkout", "main"]);
    f.write_file("a.txt", "main2\n");
    f.git(&["add", "a.txt"]);
    f.commit("on main");
    // Merge — будет конфликт.
    let _ = std::process::Command::new("git")
        .current_dir(f.path())
        .args(["merge", "feature"])
        .output();
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("conflicts", out);
}

#[test]
fn snapshot_fork_repo() {
    let f = GitFixture::new();
    f.add_remote("origin", "git@github.com:me/myrepo.git");
    f.add_remote("upstream", "git@github.com:them/myrepo.git");
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("fork", out);
}

#[test]
fn snapshot_detached_head() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.git(&["checkout", "--detach", "HEAD"]);
    let p = payload(f.path().to_str().unwrap());
    let settings = full_git_settings();
    let out = render_all(&p, &settings).join("\n");
    insta::assert_snapshot!("detached", out);
}
