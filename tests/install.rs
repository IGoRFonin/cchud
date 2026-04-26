//! Integration tests for `cchud install`.
//!
//! Each test runs in a fresh `tempfile::TempDir` with `HOME` overridden
//! to that directory. `serial_test::serial` ensures they don't race on
//! the shared `HOME` env var.
//!
//! Covers spec decision 3 (detect + --force) and PRD AC-008 (preserve
//! unrelated keys).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn settings_path(home: &TempDir) -> PathBuf {
    home.path().join(".claude/settings.json")
}

fn write_settings(home: &TempDir, content: &str) {
    let path = settings_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
}

fn read_settings(home: &TempDir) -> serde_json::Value {
    let s = fs::read_to_string(settings_path(home)).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn cchud_install(home: &TempDir, extra_args: &[&str]) -> std::process::Output {
    Command::cargo_bin("cchud")
        .unwrap()
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .arg("install")
        .args(extra_args)
        .output()
        .unwrap()
}

#[test]
#[serial]
fn install_creates_settings_when_absent() {
    let home = TempDir::new().unwrap();
    let out = cchud_install(&home, &[]);
    assert!(
        out.status.success(),
        "install must succeed when no settings.json exists; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let value = read_settings(&home);
    let cmd = value
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .expect("statusLine.command must be set");
    assert!(
        cmd.contains("cchud"),
        "statusLine.command should reference cchud binary, got {cmd}"
    );
    assert_eq!(
        value
            .get("statusLine")
            .and_then(|s| s.get("type"))
            .and_then(|t| t.as_str()),
        Some("command"),
        "statusLine.type must be 'command'"
    );
}

#[test]
#[serial]
fn install_refuses_existing_non_cchud_statusline() {
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{"statusLine":{"type":"command","command":"/usr/bin/somethingelse"}}"#,
    );
    let original = fs::read_to_string(settings_path(&home)).unwrap();

    let out = cchud_install(&home, &[]);
    assert!(
        !out.status.success(),
        "install must FAIL when foreign statusLine exists without --force"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("statusLine already set to: /usr/bin/somethingelse"),
        "stderr must mention existing path; got {stderr:?}"
    );
    assert!(
        stderr.contains("--force"),
        "stderr must hint --force; got {stderr:?}"
    );

    let after = fs::read_to_string(settings_path(&home)).unwrap();
    assert_eq!(original, after, "settings.json must be untouched on refuse");
}

#[test]
#[serial]
fn install_force_overwrites_existing() {
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{"statusLine":{"type":"command","command":"/usr/bin/somethingelse"}}"#,
    );
    let out = cchud_install(&home, &["--force"]);
    assert!(
        out.status.success(),
        "install --force must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value = read_settings(&home);
    let cmd = value
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap();
    assert!(
        cmd.contains("cchud"),
        "after --force, command must reference cchud, got {cmd}"
    );
    assert!(
        !cmd.contains("somethingelse"),
        "old command must be replaced, got {cmd}"
    );
}

#[test]
#[serial]
fn install_preserves_unrelated_keys() {
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{
            "theme": "dark",
            "mcpServers": {"example": {"command": "/usr/bin/foo"}},
            "customApiKeyResponses": {"approved": ["sk-test"]}
        }"#,
    );
    let out = cchud_install(&home, &[]);
    assert!(
        out.status.success(),
        "install must succeed without statusLine"
    );
    let value = read_settings(&home);

    assert_eq!(
        value.get("theme").and_then(|t| t.as_str()),
        Some("dark"),
        "theme must be preserved"
    );
    assert!(
        value.get("mcpServers").is_some(),
        "mcpServers must be preserved"
    );
    assert_eq!(
        value
            .get("mcpServers")
            .and_then(|m| m.get("example"))
            .and_then(|e| e.get("command"))
            .and_then(|c| c.as_str()),
        Some("/usr/bin/foo"),
        "nested mcpServers content must be preserved"
    );
    assert!(
        value.get("customApiKeyResponses").is_some(),
        "customApiKeyResponses must be preserved"
    );
    assert!(
        value.get("statusLine").is_some(),
        "statusLine must be added"
    );
}

#[test]
#[serial]
fn install_repeat_silently_updates_path() {
    let home = TempDir::new().unwrap();
    let out1 = cchud_install(&home, &[]);
    assert!(out1.status.success());
    let cmd1 = read_settings(&home)
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap()
        .to_string();
    assert!(cmd1.contains("cchud"));

    let out2 = cchud_install(&home, &[]);
    assert!(
        out2.status.success(),
        "repeat install on cchud-managed statusLine must succeed without --force; stderr={}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let cmd2 = read_settings(&home)
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap()
        .to_string();
    assert!(cmd2.contains("cchud"));
}
