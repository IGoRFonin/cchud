//! Insta snapshot tests over Phase 0 payload samples.
//!
//! Each sample → its own .snap file. Any change in rendered output
//! (display_name format, separator, widget order, etc.) requires
//! `cargo insta review` to acknowledge. See `tests/snapshots/`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::error::Error;
use std::path::PathBuf;
use tempfile::TempDir;

fn run_with_home_and_payload(home: &PathBuf, payload: &str) -> String {
    let output = Command::cargo_bin("cchud")
        .unwrap()
        .env("HOME", home)
        .env("USERPROFILE", home)
        .write_stdin(payload.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "non-zero exit: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

fn write_settings(home: &PathBuf, settings_json: &str) {
    let config_dir = home.join(".config/cchud");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("settings.json"), settings_json).unwrap();
}

#[test]
fn render_default_line_for_phase0_samples() {
    insta::glob!("../benches/samples", "payload-*.json", |path| {
        // Phase 3: synthetic-семплы рендерятся явными snapshot-сценариями
        // в Task 8 на специфичных configs, не на default-line.
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if fname.starts_with("payload-synthetic-") {
            return;
        }
        let payload = std::fs::read_to_string(path).unwrap();
        let output = Command::cargo_bin("cchud")
            .unwrap()
            .write_stdin(payload)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "non-zero exit on {path:?}: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        // Trim trailing newline from println! for stable snapshot
        let stdout = stdout.trim_end();
        insta::assert_snapshot!(stdout);
    });
}

#[test]
fn graceful_fallback_on_broken_json() -> Result<(), Box<dyn Error>> {
    let output = Command::cargo_bin("cchud")?
        .write_stdin("not json {{")
        .output()?;
    assert!(
        output.status.success(),
        "exit must be 0 on broken JSON (AC-007)"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on broken JSON, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.starts_with("cchud:"),
        "stderr must start with 'cchud:', got {stderr:?}"
    );
    Ok(())
}

/// Scenario 2: 22-widget config против реального Phase 0 sonnet-xlarge payload.
/// TerminalWidth исключён (no-TTY под cargo test → None → пустой сегмент).
/// CustomCommand отдельно в Scenario 5.
#[test]
fn scenario_2_full_22_widgets_sonnet_xlarge() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = include_str!("../benches/configs/phase-3-22w.json");
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_full_22w_sonnet", stdout);
}

/// Scenario 3: Worktree-кластер + VimMode на synthetic-семпле.
#[test]
fn scenario_3_worktree_vim() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "version": 1,
        "lines": [{
            "widgets": [
                {"type": "model"},
                {"type": "vim-mode"},
                {"type": "worktree"},
                {"type": "worktree-mode"},
                {"type": "worktree-name"},
                {"type": "worktree-branch"},
                {"type": "worktree-original-branch"}
            ]
        }],
        "theme": {}
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-synthetic-vim-worktree.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_worktree_vim", stdout);
}

/// Scenario 4: Tokens + ContextBar + ContextPercentage + ContextPercentageUsable
/// — лочит untagged enum CurrentUsage и форматы.
#[test]
fn scenario_4_context_cluster() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "version": 1,
        "lines": [{
            "widgets": [
                {"type": "tokens-input"},
                {"type": "tokens-output"},
                {"type": "context-length"},
                {"type": "context-percentage"},
                {"type": "context-percentage-usable"},
                {"type": "context-bar", "width": 10}
            ]
        }],
        "theme": {}
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_context_cluster", stdout);
}

/// Scenario 5: Static cluster + CustomCommand. Unix-only — Windows
/// behavioural тесты subprocess отложены до Phase 9.
#[cfg(unix)]
#[test]
fn scenario_5_static_and_command() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let settings = r#"{
        "version": 1,
        "lines": [{
            "widgets": [
                {"type": "custom-text", "text": "demo"},
                {"type": "custom-symbol", "symbol": "★"},
                {"type": "link", "url": "https://example.com", "label": "Ex"},
                {"type": "custom-command", "command": "echo", "args": ["phase-3"], "timeout_ms": 1000}
            ]
        }],
        "theme": {}
    }"#;
    write_settings(&home, settings);
    let payload = include_str!("../benches/samples/payload-cchud-sonnet-xlarge.json");
    let stdout = run_with_home_and_payload(&home, payload);
    insta::assert_snapshot!("phase3_static_and_command", stdout);
}
