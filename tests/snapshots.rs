//! Integration tests: render pipeline.
//!
//! Phase 2 — verifies binary parses stdin payloads from Phase 0 fixtures
//! and produces stdout containing the model display name. Insta-based
//! snapshot tests with full output capture land in Task 9.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::{error::Error, fs};

#[test]
fn renders_model_for_each_phase0_sample() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in fs::read_dir("benches/samples")? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let payload = fs::read_to_string(&path)?;
            // Извлекаем ожидаемый display_name из JSON напрямую,
            // чтобы тест работал с любым семплом.
            let parsed: serde_json::Value = serde_json::from_str(&payload)?;
            let expected = parsed
                .get("model")
                .and_then(|m| m.get("display_name"))
                .and_then(|v| v.as_str())
                .ok_or("sample missing model.display_name")?;

            let output = Command::cargo_bin("cchud")?
                .write_stdin(payload.clone())
                .output()?;

            assert!(
                output.status.success(),
                "non-zero exit on {path:?}: stderr={}",
                String::from_utf8_lossy(&output.stderr),
            );
            let stdout = String::from_utf8(output.stdout)?;
            assert!(
                stdout.trim() == expected,
                "stdout {stdout:?} does not equal expected display_name {expected:?} for {path:?}",
            );

            count += 1;
        }
    }
    assert!(count >= 4, "expected ≥4 payload samples, found {count}");
    Ok(())
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
