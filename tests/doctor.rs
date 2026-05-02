//! Phase 9 Task 5 — doctor command tests.
#![allow(clippy::unwrap_used)]

use cchud::commands::doctor::testing::{CheckStatus, DoctorEnv, run_checks};
use std::fs;
use tempfile::tempdir;

#[test]
fn all_checks_pass_returns_exit_zero() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: Some(dir.path().join("cchud.json")),
        cache_dir: Some(dir.path().join("cache")),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::create_dir_all(env.cache_dir.as_ref().unwrap()).unwrap();
    fs::write(
        env.claude_settings.as_ref().unwrap(),
        r#"{"statusLine":{"type":"command","command":"cchud"}}"#,
    )
    .unwrap();
    fs::write(
        env.cchud_settings.as_ref().unwrap(),
        r#"{"version":1,"lines":[],"theme":{}}"#,
    )
    .unwrap();

    let report = run_checks(&env);
    assert_eq!(report.exit_code, 0, "checks: {:?}", report.checks);
    assert!(report.summary.fail == 0);
}

#[test]
fn invalid_claude_settings_yields_warn() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::write(env.claude_settings.as_ref().unwrap(), "not-json").unwrap();
    let report = run_checks(&env);
    let claude = report
        .checks
        .iter()
        .find(|c| c.name == "claude_settings")
        .unwrap();
    assert!(matches!(claude.status, CheckStatus::Warn));
}

#[test]
fn missing_binary_yields_fail() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: Some(dir.path().join("nonexistent_binary")),
        gh_required: false,
    };
    let report = run_checks(&env);
    let bin = report
        .checks
        .iter()
        .find(|c| c.name == "binary_path")
        .unwrap();
    assert!(matches!(bin.status, CheckStatus::Fail));
    assert_eq!(report.exit_code, 1);
}

#[test]
fn no_git_pr_widget_skips_gh_check() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let gh = report.checks.iter().find(|c| c.name == "gh_cli").unwrap();
    assert!(
        matches!(gh.status, CheckStatus::Skip),
        "got: {:?}",
        gh.status
    );
}

#[test]
fn warn_without_fail_yields_exit_two() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::write(env.claude_settings.as_ref().unwrap(), "not-json").unwrap();
    let report = run_checks(&env);
    assert!(report.summary.fail == 0);
    assert!(report.summary.warn >= 1);
    assert_eq!(report.exit_code, 2);
}

#[test]
fn missing_cache_dir_when_parent_writable_yields_warn() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().join("subdir/cchud_cache")),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let cache = report
        .checks
        .iter()
        .find(|c| c.name == "cache_dir")
        .unwrap();
    assert!(matches!(cache.status, CheckStatus::Warn));
}

#[test]
fn json_output_serializes_summary() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("summary").is_some());
    assert!(json.get("checks").unwrap().is_array());
    assert!(json.get("exit_code").is_some());
}
