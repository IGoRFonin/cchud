//! `cchud install` — wires cchud into ~/.claude/settings.json statusLine.command.
//!
//! Behavior (matches spec decision 3):
//! - No existing statusLine → write our path, exit 0
//! - Existing statusLine NOT containing "cchud" + no --force → exit 1
//! - Existing statusLine NOT containing "cchud" + --force → overwrite
//! - Existing statusLine ALREADY containing "cchud" → silently update path
//!
//! Preserves all other keys in settings.json (mcpServers, theme, etc.) by
//! operating on `serde_json::Value` instead of typed `Settings`.
//!
//! Atomic write: writes to settings.json.tmp, then renames. Race-safe.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    let force = args.iter().any(|a| a == "--force");
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: cannot determine executable path: {e}");
            return ExitCode::from(1);
        }
    };
    let path = settings_path();

    let mut root = read_or_empty(&path);

    if let Some(cmd) = root
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(serde_json::Value::as_str)
    {
        if !cmd.contains("cchud") && !force {
            eprintln!("cchud: statusLine already set to: {cmd}");
            eprintln!("       use --force to overwrite, or remove it manually first.");
            return ExitCode::from(1);
        }
    }

    root["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "padding": 0,
    });

    if let Err(e) = write_atomic(&path, &root) {
        eprintln!("cchud: cannot write {}: {e}", path.display());
        return ExitCode::from(1);
    }
    println!("cchud installed: {}", exe.display());
    ExitCode::SUCCESS
}

fn settings_path() -> PathBuf {
    if let Ok(p) = std::env::var("CCHUD_SETTINGS") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/settings.json")
}

fn read_or_empty(path: &Path) -> serde_json::Value {
    let Ok(s) = std::fs::read_to_string(path) else {
        return serde_json::json!({});
    };
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({}))
}

fn write_atomic(path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let pretty = serde_json::to_string_pretty(value)?;
    std::fs::write(&tmp, pretty)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
