//! `cchud import` — best-effort migration from ccstatusline. Phase 8 Task 13.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

use crate::types::config::Settings;

pub struct ImportArgs {
    pub from: Option<PathBuf>,
    pub then_configure: bool,
    pub force: bool,
}

#[must_use]
pub fn run(raw_args: &[String]) -> ExitCode {
    let args = match parse_args(raw_args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("cchud import: {e}");
            eprintln!("       usage: cchud import [--from <path>] [--then-configure] [--force]");
            return ExitCode::from(2);
        }
    };

    let Some(src) = resolve_source(args.from.as_ref()) else {
        eprintln!("cchud import: source not found; use --from <path> to specify");
        return ExitCode::from(1);
    };

    let raw = match std::fs::read_to_string(&src) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cchud import: cannot read {}: {e}", src.display());
            return ExitCode::from(1);
        }
    };

    let value: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cchud import: source is not valid JSON: {e}");
            return ExitCode::from(1);
        }
    };

    let Some(cc_section) = extract_ccstatusline(&value) else {
        eprintln!(
            "cchud import: no ccstatusline section found in {}",
            src.display()
        );
        return ExitCode::from(1);
    };

    let (settings, skipped) = parse_best_effort(cc_section);
    for name in &skipped {
        eprintln!("cchud import: skipped unknown widget: {name}");
    }
    let total_widgets: usize = settings.lines.iter().map(|l| l.widgets.len()).sum();
    if total_widgets == 0 {
        eprintln!("cchud import: nothing imported (all widgets unknown)");
        return ExitCode::from(1);
    }

    let dest = config_destination();
    if dest.exists() && !args.force {
        eprintln!(
            "cchud import: {} already exists; use --force to overwrite",
            dest.display()
        );
        return ExitCode::from(1);
    }

    let backup = match crate::tui::save::atomic_save(&settings, &dest) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cchud import: cannot write {}: {e}", dest.display());
            return ExitCode::from(1);
        }
    };

    println!(
        "cchud import: {} widgets across {} lines{}",
        total_widgets,
        settings.lines.len(),
        backup.map_or(String::new(), |p| format!("; backup at {}", p.display()))
    );

    if args.then_configure {
        return crate::commands::configure::run(&[]);
    }
    ExitCode::SUCCESS
}

fn parse_args(raw: &[String]) -> Result<ImportArgs, String> {
    let mut from: Option<PathBuf> = None;
    let mut then_configure = false;
    let mut force = false;
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--from" => {
                i += 1;
                let p = raw
                    .get(i)
                    .ok_or_else(|| "--from requires <path>".to_string())?;
                from = Some(PathBuf::from(p));
            }
            "--then-configure" => then_configure = true,
            "--force" => force = true,
            other => return Err(format!("unknown arg: {other}")),
        }
        i += 1;
    }
    Ok(ImportArgs {
        from,
        then_configure,
        force,
    })
}

fn resolve_source(from: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(p) = from {
        return Some(p.clone());
    }
    let claude = dirs::home_dir()?.join(".claude/settings.json");
    claude.exists().then_some(claude)
}

fn config_destination() -> PathBuf {
    if let Ok(p) = std::env::var("CCHUD_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/cchud/settings.json")
}

fn extract_ccstatusline(v: &Value) -> Option<Value> {
    if let Some(sec) = v.get("ccstatusline") {
        return Some(sec.clone());
    }
    // Top-level Value сам выглядит как Settings? (имеет lines или theme)
    if v.get("lines").is_some() || v.get("theme").is_some() {
        return Some(v.clone());
    }
    None
}

/// Best-effort парсинг. Сначала пробует `serde_json::from_value::<Settings>`;
/// если fail — фильтрует `lines[].widgets[]` по whitelist `widget_meta::lookup_by_kebab`.
pub fn parse_best_effort(mut value: Value) -> (Settings, Vec<String>) {
    if let Ok(s) = <Settings as serde::Deserialize>::deserialize(&value) {
        return (s, Vec::new());
    }

    let mut skipped: Vec<String> = Vec::new();
    if let Some(lines) = value.get_mut("lines").and_then(Value::as_array_mut) {
        for line in lines.iter_mut() {
            if let Some(widgets) = line.get_mut("widgets").and_then(Value::as_array_mut) {
                widgets.retain(|w| {
                    let kind = w.get("type").and_then(Value::as_str).unwrap_or("");
                    let known = crate::tui::widget_meta::lookup_by_kebab(kind).is_some();
                    if !known && !kind.is_empty() {
                        skipped.push(kind.to_string());
                    }
                    known
                });
            }
        }
    }

    match serde_json::from_value::<Settings>(value) {
        Ok(s) => (s, skipped),
        Err(_) => (Settings::default(), skipped),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_best_effort_full_config_no_skipped() {
        let v = json!({
            "version": 1,
            "lines": [{"widgets": [{"type": "model"}, {"type": "git-branch"}]}],
            "theme": {}
        });
        let (s, skipped) = parse_best_effort(v);
        assert_eq!(s.lines[0].widgets.len(), 2);
        assert!(skipped.is_empty());
    }

    #[test]
    fn parse_best_effort_skips_unknown_widget_types() {
        let v = json!({
            "lines": [{"widgets": [{"type": "model"}, {"type": "totally-fake-widget"}]}],
            "theme": {}
        });
        let (s, skipped) = parse_best_effort(v);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert_eq!(skipped, vec!["totally-fake-widget"]);
    }

    #[test]
    fn extract_finds_ccstatusline_section() {
        let v = json!({
            "ccstatusline": {"lines": [{"widgets": [{"type": "model"}]}]},
            "other": "stuff"
        });
        let extracted = extract_ccstatusline(&v).unwrap();
        assert!(extracted.get("lines").is_some());
    }

    #[test]
    fn extract_recognizes_top_level_settings_shape() {
        let v = json!({"lines": [{"widgets": []}]});
        assert!(extract_ccstatusline(&v).is_some());
    }

    #[test]
    fn extract_returns_none_for_unrelated_json() {
        let v = json!({"foo": 1, "bar": 2});
        assert!(extract_ccstatusline(&v).is_none());
    }

    #[test]
    fn parse_args_handles_all_flags() {
        let raw = vec![
            "--from".into(),
            "/some/path".into(),
            "--then-configure".into(),
            "--force".into(),
        ];
        let a = parse_args(&raw).unwrap();
        assert_eq!(a.from, Some(PathBuf::from("/some/path")));
        assert!(a.then_configure);
        assert!(a.force);
    }
}
