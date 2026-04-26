//! Insta snapshot tests over Phase 0 payload samples.
//!
//! Each sample → its own .snap file. Any change in rendered output
//! (display_name format, separator, widget order, etc.) requires
//! `cargo insta review` to acknowledge. See `tests/snapshots/`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::error::Error;

#[test]
fn render_default_line_for_phase0_samples() {
    insta::glob!("../benches/samples", "payload-*.json", |path| {
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
