//! Full `StatusPayload` envelope — Phase 2 Task 9 expansion.
//!
//! All 15 envelope fields from Claude Code statusLine payload (Phase 0
//! research). Heavy nested structures (`cost`, `context_window`,
//! `rate_limits`, `effort`, `thinking`, `output_style`) are kept as
//! `Option<serde_json::Value>` until the phase that consumes them:
//! - Phase 6: `cost`, `context_window`, `rate_limits` → typed
//! - Phase 7: `effort`, `thinking`, `output_style` → typed (or stay Value)
//!
//! Top-level envelope is fully typed so snapshot tests catch any
//! Anthropic schema drift in field names/presence.

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StatusPayload {
    pub session_id: String,
    pub model: ModelInfo,
    pub workspace: Workspace,

    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub fast_mode: Option<bool>,
    #[serde(default)]
    pub exceeds_200k_tokens: Option<bool>,

    // Heavy sub-structures — kept as Value, typed in Phase 6/7.
    #[serde(default)]
    pub output_style: Option<serde_json::Value>,
    #[serde(default)]
    pub cost: Option<serde_json::Value>,
    #[serde(default)]
    pub context_window: Option<serde_json::Value>,
    #[serde(default)]
    pub rate_limits: Option<serde_json::Value>,
    #[serde(default)]
    pub effort: Option<serde_json::Value>,
    #[serde(default)]
    pub thinking: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub current_dir: String,
    #[serde(default)]
    pub project_dir: Option<String>,
    #[serde(default)]
    pub added_dirs: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    const SAMPLE: &str = include_str!("../../benches/samples/payload-cchud-sonnet-xlarge.json");

    #[test]
    fn parses_real_payload_sample() {
        let payload: StatusPayload = serde_json::from_str(SAMPLE).expect("sample must parse");
        assert_eq!(payload.model.id, "claude-sonnet-4-6");
        assert_eq!(payload.model.display_name, "Sonnet 4.6");
        assert!(!payload.session_id.is_empty());
        assert_eq!(
            payload.workspace.current_dir,
            "/Users/igor/mp/startup/cchud"
        );
        // Heavy sub-structures should now parse into Some(Value)
        assert!(payload.cost.is_some(), "cost field must parse from sample");
        assert!(
            payload.context_window.is_some(),
            "context_window must parse"
        );
        assert!(payload.rate_limits.is_some(), "rate_limits must parse");
        assert_eq!(payload.version.as_deref(), Some("2.1.119"));
    }

    #[test]
    fn rejects_missing_required_fields() {
        let bad = r#"{"session_id":"x"}"#;
        assert!(serde_json::from_str::<StatusPayload>(bad).is_err());
    }

    #[test]
    fn parses_minimal_payload_without_heavy_fields() {
        // Минимум, который Anthropic мог бы прислать в worst-case
        let minimal = r#"{
            "session_id": "x",
            "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
            "workspace": {"current_dir": "/tmp"}
        }"#;
        let p: StatusPayload = serde_json::from_str(minimal).unwrap();
        assert!(p.cost.is_none());
        assert!(p.context_window.is_none());
        assert!(p.fast_mode.is_none());
    }
}
