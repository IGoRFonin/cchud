//! Full `StatusPayload` envelope — типизация всех полей.
//!
//! `CurrentUsage` — untagged enum: upstream zod допускает форму
//! `current_usage?: number | { ... } | null`. На практике приходит только
//! object | null, но защитный fallback на `Total(u64)` стоит копейки.
//!
//! Top-level envelope полностью типизирован; snapshot-тесты ловят
//! schema drift в именах/наличии полей.

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

    // typed:
    #[serde(default)]
    pub output_style: Option<OutputStyle>,
    #[serde(default)]
    pub cost: Option<CostInfo>,
    #[serde(default)]
    pub context_window: Option<ContextWindowInfo>,
    #[serde(default)]
    pub worktree: Option<Worktree>,
    #[serde(default)]
    pub vim: Option<VimState>,

    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
    // Остаются Value — типизация позже:
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

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub total_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_api_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_lines_added: Option<u64>,
    #[serde(default)]
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextWindowInfo {
    #[serde(default)]
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub total_input_tokens: Option<u64>,
    #[serde(default)]
    pub total_output_tokens: Option<u64>,
    #[serde(default)]
    pub current_usage: Option<CurrentUsage>,
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub remaining_percentage: Option<f64>,
}

/// Upstream zod допускает `current_usage?: number | object | null`.
/// На практике приходит только object; Total — защита от потенциального
/// упрощения схемы Anthropic.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CurrentUsage {
    Detailed {
        #[serde(default)]
        input_tokens: Option<u64>,
        #[serde(default)]
        output_tokens: Option<u64>,
        #[serde(default)]
        cache_creation_input_tokens: Option<u64>,
        #[serde(default)]
        cache_read_input_tokens: Option<u64>,
    },
    Total(u64),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Worktree {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub original_cwd: Option<String>,
    #[serde(default)]
    pub original_branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VimState {
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputStyle {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<RateBucket>,
    #[serde(default)]
    pub seven_day: Option<RateBucket>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RateBucket {
    #[serde(default)]
    pub used_percentage: Option<f64>,
    /// Unix seconds, UTC.
    #[serde(default)]
    pub resets_at: Option<i64>,
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
        let minimal = r#"{
            "session_id": "x",
            "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
            "workspace": {"current_dir": "/tmp"}
        }"#;
        let p: StatusPayload = serde_json::from_str(minimal).unwrap();
        assert!(p.cost.is_none());
        assert!(p.context_window.is_none());
        assert!(p.fast_mode.is_none());
        assert!(p.vim.is_none());
        assert!(p.worktree.is_none());
    }

    #[test]
    fn parses_current_usage_detailed_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {
                "current_usage": {
                    "input_tokens": 1,
                    "output_tokens": 2,
                    "cache_creation_input_tokens": 3,
                    "cache_read_input_tokens": 4
                }
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Detailed {
                input_tokens,
                output_tokens,
                ..
            } => {
                assert_eq!(input_tokens, Some(1));
                assert_eq!(output_tokens, Some(2));
            }
            CurrentUsage::Total(_) => panic!("expected Detailed"),
        }
    }

    #[test]
    fn parses_current_usage_total_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {"current_usage": 12345}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Total(v) => assert_eq!(v, 12345),
            CurrentUsage::Detailed { .. } => panic!("expected Total"),
        }
    }

    #[test]
    fn parses_vim_and_worktree_envelope_fields() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "vim": {"mode": "NORMAL"},
            "worktree": {
                "name": "wt-feature",
                "branch": "feature/x",
                "original_branch": "main"
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            p.vim.as_ref().and_then(|v| v.mode.as_deref()),
            Some("NORMAL")
        );
        let wt = p.worktree.expect("worktree present");
        assert_eq!(wt.name.as_deref(), Some("wt-feature"));
        assert_eq!(wt.branch.as_deref(), Some("feature/x"));
        assert_eq!(wt.original_branch.as_deref(), Some("main"));
    }

    #[test]
    fn parses_rate_limits_typed() {
        let p: StatusPayload = serde_json::from_str(SAMPLE).unwrap();
        let rl = p.rate_limits.expect("rate_limits present in sample");
        let five = rl.five_hour.expect("five_hour bucket present");
        assert_eq!(five.used_percentage, Some(1.0));
        assert_eq!(five.resets_at, Some(1_777_198_800));
        let seven = rl.seven_day.expect("seven_day bucket present");
        assert_eq!(seven.used_percentage, Some(6.0));
        assert_eq!(seven.resets_at, Some(1_777_485_600));
    }

    #[test]
    fn rate_limits_absent_yields_none() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert!(p.rate_limits.is_none());
    }

    #[test]
    fn rate_limits_missing_buckets_yields_none_buckets() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "rate_limits": {}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let rl = p.rate_limits.unwrap();
        assert!(rl.five_hour.is_none());
        assert!(rl.seven_day.is_none());
    }

    #[test]
    fn parses_typed_cost_and_output_style() {
        let p: StatusPayload = serde_json::from_str(SAMPLE).unwrap();
        let cost = p.cost.expect("cost present");
        assert!(cost.total_cost_usd.unwrap_or(0.0) > 0.0);
        assert!(cost.total_duration_ms.unwrap_or(0) > 0);
        let style = p.output_style.expect("output_style present");
        assert_eq!(style.name.as_deref(), Some("default"));
    }
}
