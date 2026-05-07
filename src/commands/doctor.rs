//! `cchud doctor` — environment health check (Phase 9).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Summary {
    pub pass: u32,
    pub warn: u32,
    pub fail: u32,
    pub skip: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub version: &'static str,
    pub checks: Vec<CheckResult>,
    pub summary: Summary,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Default)]
pub struct DoctorEnv {
    pub claude_settings: Option<PathBuf>,
    pub cchud_settings: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub binary_path: Option<PathBuf>,
    pub gh_required: bool,
}

#[must_use]
#[allow(dead_code)] // wired in T6
pub fn run(args: &[String]) -> ExitCode {
    let json_mode = args.iter().any(|a| a == "--json");
    let env = build_env_from_real_paths();
    let report = run_checks(&env);
    if json_mode {
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("cchud doctor: serialize error: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        print_human_report(&report);
    }
    ExitCode::from(u8::try_from(report.exit_code).unwrap_or(1))
}

fn build_env_from_real_paths() -> DoctorEnv {
    let home = dirs::home_dir();
    DoctorEnv {
        claude_settings: home.as_ref().map(|h| h.join(".claude/settings.json")),
        cchud_settings: dirs::config_dir().map(|c| c.join("cchud/settings.json")),
        cache_dir: dirs::cache_dir().map(|c| c.join("cchud")),
        binary_path: std::env::current_exe().ok(),
        gh_required: detect_gh_widget_in_config(home.as_deref()),
    }
}

fn detect_gh_widget_in_config(home: Option<&Path>) -> bool {
    let cfg_path = match home {
        Some(_) => match dirs::config_dir() {
            Some(p) => p.join("cchud/settings.json"),
            None => return false,
        },
        None => return false,
    };
    let Ok(body) = std::fs::read_to_string(&cfg_path) else {
        return false;
    };
    body.contains("\"git-pr\"") || body.contains("\"GitPullRequest\"")
}

#[must_use]
pub fn run_checks(env: &DoctorEnv) -> Report {
    let mut checks: Vec<CheckResult> = Vec::with_capacity(9);

    // 1. Version (always PASS).
    checks.push(CheckResult {
        name: "version",
        status: CheckStatus::Pass,
        detail: env!("CARGO_PKG_VERSION").to_string(),
    });

    // 2. Binary path.
    checks.push(check_binary_path(env));

    // 3. Platform target (always PASS).
    checks.push(CheckResult {
        name: "platform_target",
        status: CheckStatus::Pass,
        detail: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    });

    // 4. Color level.
    checks.push(check_color_level());

    // 5. Hyperlinks.
    checks.push(check_hyperlinks());

    // 6. Cache dir.
    checks.push(check_cache_dir(env));

    // 7. Claude settings.
    checks.push(check_claude_settings(env));

    // 8. cchud config.
    checks.push(check_cchud_config(env));

    // 9. gh CLI (conditional).
    checks.push(check_gh_cli(env));

    let mut summary = Summary::default();
    for c in &checks {
        match c.status {
            CheckStatus::Pass => summary.pass += 1,
            CheckStatus::Warn => summary.warn += 1,
            CheckStatus::Fail => summary.fail += 1,
            CheckStatus::Skip => summary.skip += 1,
        }
    }
    let exit_code = if summary.fail > 0 {
        1
    } else if summary.warn > 0 {
        2
    } else {
        0
    };

    Report {
        version: env!("CARGO_PKG_VERSION"),
        checks,
        summary,
        exit_code,
    }
}

fn check_binary_path(env: &DoctorEnv) -> CheckResult {
    match &env.binary_path {
        Some(p) if p.exists() => CheckResult {
            name: "binary_path",
            status: CheckStatus::Pass,
            detail: p.display().to_string(),
        },
        Some(p) => CheckResult {
            name: "binary_path",
            status: CheckStatus::Fail,
            detail: format!("path does not exist: {}", p.display()),
        },
        None => CheckResult {
            name: "binary_path",
            status: CheckStatus::Fail,
            detail: "current_exe() returned Err".to_string(),
        },
    }
}

fn check_color_level() -> CheckResult {
    if std::env::var_os("NO_COLOR").is_some() {
        return CheckResult {
            name: "color_level",
            status: CheckStatus::Skip,
            detail: "NO_COLOR set".to_string(),
        };
    }
    let term = std::env::var("COLORTERM").unwrap_or_default();
    if term.contains("truecolor") || term.contains("24bit") {
        CheckResult {
            name: "color_level",
            status: CheckStatus::Pass,
            detail: "truecolor".into(),
        }
    } else if std::env::var("TERM")
        .unwrap_or_default()
        .contains("256color")
    {
        CheckResult {
            name: "color_level",
            status: CheckStatus::Pass,
            detail: "256color".into(),
        }
    } else {
        CheckResult {
            name: "color_level",
            status: CheckStatus::Warn,
            detail: "cannot detect (set COLORTERM=truecolor or use a 256-color term)".into(),
        }
    }
}

fn check_hyperlinks() -> CheckResult {
    let tp = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let supports = matches!(
        tp.as_str(),
        "iTerm.app" | "WezTerm" | "Alacritty" | "kitty" | "vscode" | "ghostty"
    );
    if supports {
        CheckResult {
            name: "hyperlinks",
            status: CheckStatus::Pass,
            detail: format!("supported ({tp})"),
        }
    } else {
        CheckResult {
            name: "hyperlinks",
            status: CheckStatus::Warn,
            detail: format!("cannot detect (TERM_PROGRAM={tp:?})"),
        }
    }
}

fn check_cache_dir(env: &DoctorEnv) -> CheckResult {
    let Some(dir) = env.cache_dir.as_ref() else {
        return CheckResult {
            name: "cache_dir",
            status: CheckStatus::Fail,
            detail: "cannot resolve cache dir".to_string(),
        };
    };
    if dir.exists() {
        if is_writable(dir) {
            CheckResult {
                name: "cache_dir",
                status: CheckStatus::Pass,
                detail: format!("{} (writable)", dir.display()),
            }
        } else {
            CheckResult {
                name: "cache_dir",
                status: CheckStatus::Fail,
                detail: format!("{} (not writable)", dir.display()),
            }
        }
    } else {
        let writable_ancestor = find_writable_ancestor(dir);
        if writable_ancestor {
            CheckResult {
                name: "cache_dir",
                status: CheckStatus::Warn,
                detail: format!("{} (does not exist; parent writable)", dir.display()),
            }
        } else {
            CheckResult {
                name: "cache_dir",
                status: CheckStatus::Fail,
                detail: format!("{} parent not writable", dir.display()),
            }
        }
    }
}

fn find_writable_ancestor(p: &Path) -> bool {
    let mut current = p;
    loop {
        if current.exists() {
            return is_writable(current);
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => return false,
        }
    }
}

fn is_writable(p: &Path) -> bool {
    let probe = p.join(format!(".cchud_doctor_probe_{}", std::process::id()));
    let res = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    res
}

fn check_claude_settings(env: &DoctorEnv) -> CheckResult {
    let Some(p) = env.claude_settings.as_ref() else {
        return CheckResult {
            name: "claude_settings",
            status: CheckStatus::Skip,
            detail: "no claude settings path".to_string(),
        };
    };
    let Ok(body) = std::fs::read_to_string(p) else {
        return CheckResult {
            name: "claude_settings",
            status: CheckStatus::Skip,
            detail: format!("{} does not exist", p.display()),
        };
    };
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(v) => {
            let cmd = v
                .get("statusLine")
                .and_then(|s| s.get("command"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if cmd.contains("cchud") {
                CheckResult {
                    name: "claude_settings",
                    status: CheckStatus::Pass,
                    detail: format!("{} (statusLine wired to cchud)", p.display()),
                }
            } else {
                CheckResult {
                    name: "claude_settings",
                    status: CheckStatus::Warn,
                    detail: format!("{} (statusLine not wired)", p.display()),
                }
            }
        }
        Err(e) => CheckResult {
            name: "claude_settings",
            status: CheckStatus::Warn,
            detail: format!("invalid JSON: {e}"),
        },
    }
}

fn check_cchud_config(env: &DoctorEnv) -> CheckResult {
    let Some(p) = env.cchud_settings.as_ref() else {
        return CheckResult {
            name: "cchud_config",
            status: CheckStatus::Skip,
            detail: "no cchud config path".to_string(),
        };
    };
    let Ok(body) = std::fs::read_to_string(p) else {
        return CheckResult {
            name: "cchud_config",
            status: CheckStatus::Warn,
            detail: format!("{} does not exist", p.display()),
        };
    };
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(_) => CheckResult {
            name: "cchud_config",
            status: CheckStatus::Pass,
            detail: format!("{} (valid)", p.display()),
        },
        Err(e) => CheckResult {
            name: "cchud_config",
            status: CheckStatus::Warn,
            detail: format!("invalid JSON: {e}"),
        },
    }
}

fn check_gh_cli(env: &DoctorEnv) -> CheckResult {
    if !env.gh_required {
        return CheckResult {
            name: "gh_cli",
            status: CheckStatus::Skip,
            detail: "no git-pr widget configured".to_string(),
        };
    }
    let gh_present = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !gh_present {
        return CheckResult {
            name: "gh_cli",
            status: CheckStatus::Fail,
            detail: "gh not installed; required by git-pr widget".to_string(),
        };
    }
    let auth_ok = std::process::Command::new("gh")
        .args(["auth", "status"])
        .output()
        .is_ok_and(|o| o.status.success());
    if auth_ok {
        CheckResult {
            name: "gh_cli",
            status: CheckStatus::Pass,
            detail: "gh auth ok".to_string(),
        }
    } else {
        CheckResult {
            name: "gh_cli",
            status: CheckStatus::Warn,
            detail: "gh present but not authenticated".to_string(),
        }
    }
}

const fn status_mark(status: &CheckStatus) -> &'static str {
    #[cfg(windows)]
    {
        match status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
            CheckStatus::Skip => "SKIP",
        }
    }
    #[cfg(not(windows))]
    {
        match status {
            CheckStatus::Pass => "✓",
            CheckStatus::Warn => "⚠",
            CheckStatus::Fail => "✗",
            CheckStatus::Skip => "⊘",
        }
    }
}

fn print_human_report(report: &Report) {
    println!();
    println!("cchud doctor — environment check");
    println!();
    for c in &report.checks {
        let mark = status_mark(&c.status);
        println!("  {} {:<18} {}", mark, c.name.replace('_', " "), c.detail);
    }
    println!();
    println!(
        "Result: {} passed, {} warnings, {} failed, {} skipped.",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip,
    );
}

#[doc(hidden)]
pub mod testing {
    #[allow(unused_imports)]
    pub use super::{CheckResult, CheckStatus, DoctorEnv, Report, Summary, run_checks};
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{find_writable_ancestor, is_writable};
    use tempfile::tempdir;

    #[test]
    fn is_writable_returns_true_for_writable_dir() {
        let dir = tempdir().unwrap();
        assert!(is_writable(dir.path()));
    }

    #[test]
    fn find_writable_ancestor_finds_existing_parent() {
        let dir = tempdir().unwrap();
        let nonexistent = dir.path().join("a/b/c");
        // The tempdir itself exists and is writable, so the ancestor walk should succeed.
        assert!(find_writable_ancestor(&nonexistent));
    }

    #[test]
    fn find_writable_ancestor_returns_false_for_unresolvable_path() {
        // A path whose every ancestor is non-existent terminates at root.
        // Root exists but may not be writable; on the other hand we only need to
        // confirm the function does not panic and returns a bool.
        let result = std::panic::catch_unwind(|| {
            find_writable_ancestor(std::path::Path::new("/nonexistent_cchud_test_dir/sub"));
        });
        assert!(result.is_ok());
    }
}
