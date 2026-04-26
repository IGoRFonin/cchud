//! Minimal `StatusPayload` — Phase 2 walking skeleton.
//! Full envelope (15 fields incl. `cost`/`context_window`/`rate_limits`
//! as `Option<serde_json::Value>`) is added in Task 9.

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
    }

    #[test]
    fn rejects_missing_required_fields() {
        let bad = r#"{"session_id":"x"}"#;
        assert!(serde_json::from_str::<StatusPayload>(bad).is_err());
    }
}
