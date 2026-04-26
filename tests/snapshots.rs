//! Integration tests: skeleton reader.
//!
//! Phase 1 — verifies binary accepts stdin payloads from Phase 0 fixtures
//! and produces non-empty stdout with zero exit code. In Phase 2 this is
//! replaced by `insta::assert_snapshot!` with real rendered output.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::{error::Error, fs};

#[test]
fn renders_skeleton_for_each_phase0_sample() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in fs::read_dir("benches/samples")? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let payload = fs::read_to_string(&path)?;
            let output = Command::cargo_bin("cchud")?
                .write_stdin(payload.clone())
                .output()?;

            assert!(
                output.status.success(),
                "non-zero exit on {path:?}: stderr={}",
                String::from_utf8_lossy(&output.stderr),
            );
            assert!(!output.stdout.is_empty(), "empty stdout on {path:?}",);

            let stdout = String::from_utf8(output.stdout)?;
            assert!(
                stdout.contains("cchud (skeleton)"),
                "stdout missing skeleton marker for {path:?}: {stdout}",
            );
            assert!(
                stdout.contains(&payload.len().to_string()),
                "stdout missing byte count for {path:?}: {stdout}",
            );

            count += 1;
        }
    }
    assert!(count >= 4, "expected ≥4 payload samples, found {count}");
    Ok(())
}
