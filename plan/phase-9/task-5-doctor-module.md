# Task 5 — `cchud doctor` module + 9 проверок (TDD)

**Цель:** Реализовать `src/commands/doctor.rs` (~150 LOC) с 9 проверками: version, binary path, platform target, color level, hyperlinks, cache dir, claude settings, cchud config, gh CLI. Exit codes 0 (all pass/skip), 1 (fail), 2 (warn без fail). ≥6 unit tests.

**Files:**
- Modify: `src/commands/doctor.rs` (replace stub).
- Create: `tests/doctor.rs` — integration tests.

---

- [ ] **Step 1: Failing tests первыми**

Создать `tests/doctor.rs`:

```rust
//! Phase 9 Task 5 — doctor command tests.
#![allow(clippy::unwrap_used)]

use cchud::commands::doctor::testing::{run_checks, CheckResult, CheckStatus, DoctorEnv};
use std::fs;
use tempfile::tempdir;

#[test]
fn all_checks_pass_returns_exit_zero() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: Some(dir.path().join("cchud.json")),
        cache_dir: Some(dir.path().join("cache")),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::create_dir_all(env.cache_dir.as_ref().unwrap()).unwrap();
    fs::write(
        env.claude_settings.as_ref().unwrap(),
        r#"{"statusLine":{"type":"command","command":"cchud"}}"#,
    )
    .unwrap();
    fs::write(
        env.cchud_settings.as_ref().unwrap(),
        r#"{"version":1,"lines":[],"theme":{}}"#,
    )
    .unwrap();

    let report = run_checks(&env);
    assert_eq!(report.exit_code, 0, "checks: {:?}", report.checks);
    assert!(report.summary.fail == 0);
}

#[test]
fn invalid_claude_settings_yields_warn() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::write(env.claude_settings.as_ref().unwrap(), "not-json").unwrap();
    let report = run_checks(&env);
    let claude = report.checks.iter().find(|c| c.name == "claude_settings").unwrap();
    assert!(matches!(claude.status, CheckStatus::Warn));
}

#[test]
fn missing_binary_yields_fail() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: Some(dir.path().join("nonexistent_binary")),
        gh_required: false,
    };
    let report = run_checks(&env);
    let bin = report.checks.iter().find(|c| c.name == "binary_path").unwrap();
    assert!(matches!(bin.status, CheckStatus::Fail));
    assert_eq!(report.exit_code, 1);
}

#[test]
fn no_git_pr_widget_skips_gh_check() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let gh = report.checks.iter().find(|c| c.name == "gh_cli").unwrap();
    assert!(matches!(gh.status, CheckStatus::Skip), "got: {:?}", gh.status);
}

#[test]
fn warn_without_fail_yields_exit_two() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: Some(dir.path().join("claude.json")),
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    fs::write(env.claude_settings.as_ref().unwrap(), "not-json").unwrap();
    let report = run_checks(&env);
    assert!(report.summary.fail == 0);
    assert!(report.summary.warn >= 1);
    assert_eq!(report.exit_code, 2);
}

#[test]
fn missing_cache_dir_when_parent_writable_yields_warn() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().join("subdir/cchud_cache")),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let cache = report.checks.iter().find(|c| c.name == "cache_dir").unwrap();
    assert!(matches!(cache.status, CheckStatus::Warn));
}

#[test]
fn json_output_serializes_summary() {
    let dir = tempdir().unwrap();
    let env = DoctorEnv {
        claude_settings: None,
        cchud_settings: None,
        cache_dir: Some(dir.path().to_path_buf()),
        binary_path: std::env::current_exe().ok(),
        gh_required: false,
    };
    let report = run_checks(&env);
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("summary").is_some());
    assert!(json.get("checks").unwrap().is_array());
    assert!(json.get("exit_code").is_some());
}
```

- [ ] **Step 2: Run tests — confirm они FAIL (build error)**

```bash
cargo test --locked --test doctor
```

Expected: build error — модуль `testing` ещё не существует.

- [ ] **Step 3: Реализовать `src/commands/doctor.rs`**

Заменить stub полным телом:

```rust
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
    ExitCode::from(report.exit_code as u8)
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
        Some(p) if p.exists() => {
            let detail = p.display().to_string();
            // WARN если не в ~/.local/bin/ и не в %LOCALAPPDATA%\cchud (Phase 9 expectation).
            let in_canonical = if cfg!(windows) {
                detail.to_ascii_lowercase().contains("\\cchud\\")
            } else {
                detail.contains("/.local/bin/")
            };
            if in_canonical {
                CheckResult { name: "binary_path", status: CheckStatus::Pass, detail }
            } else {
                CheckResult {
                    name: "binary_path",
                    status: CheckStatus::Warn,
                    detail: format!("{detail} (not in canonical install location)"),
                }
            }
        }
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
        CheckResult { name: "color_level", status: CheckStatus::Pass, detail: "truecolor".into() }
    } else if std::env::var("TERM").unwrap_or_default().contains("256color") {
        CheckResult { name: "color_level", status: CheckStatus::Pass, detail: "256color".into() }
    } else {
        CheckResult { name: "color_level", status: CheckStatus::Warn, detail: "basic".into() }
    }
}

fn check_hyperlinks() -> CheckResult {
    // Best-effort: TERM_PROGRAM hint.
    let tp = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let supports = matches!(
        tp.as_str(),
        "iTerm.app" | "WezTerm" | "Alacritty" | "kitty" | "vscode"
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
            detail: format!("unknown terminal: {tp}"),
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
        // Parent writable check.
        let parent = dir.parent().unwrap_or(dir);
        if is_writable(parent) {
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

fn is_writable(p: &Path) -> bool {
    let probe = p.join(".cchud_doctor_probe");
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
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);
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
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);
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

fn print_human_report(report: &Report) {
    println!();
    println!("cchud doctor — environment check");
    println!();
    for c in &report.checks {
        let mark = match c.status {
            CheckStatus::Pass => "✓",
            CheckStatus::Warn => "⚠",
            CheckStatus::Fail => "✗",
            CheckStatus::Skip => "⊘",
        };
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
    pub use super::{run_checks, CheckResult, CheckStatus, DoctorEnv, Report, Summary};
}
```

- [ ] **Step 4: Add `serde::Serialize` derive — verify уже есть в deps**

`Cargo.toml` уже имеет `serde = { features = ["derive"] }` (используется типами Settings). Проверить:

```bash
rtk grep -n 'serde =' /Users/igor/mp/startup/cchud/Cargo.toml
```

Expected: видим `serde = { version = "1", features = ["derive"] }`. Если нет `derive` — добавить.

- [ ] **Step 5: Run tests — confirm они PASS**

```bash
cargo test --locked --test doctor
```

Expected: 7 tests PASS.

- [ ] **Step 6: Run полный suite**

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Expected: всё зелёное.

- [ ] **Step 7: Commit**

```bash
git add src/commands/doctor.rs tests/doctor.rs Cargo.toml
git commit -m "$(cat <<'EOF'
feat(phase-9): T5 cchud doctor — 9-check environment report

Реализует src/commands/doctor.rs (~250 LOC):
- 9 проверок: version, binary path, platform, color, hyperlinks, cache,
  claude settings, cchud config, gh CLI (conditional на git-pr widget)
- Exit codes 0 (all pass/skip), 1 (fail), 2 (warn без fail)
- --json flag — машино-читаемый Report{ version, checks, summary, exit_code }
- DoctorEnv injectable для tests (path overrides)
- 7 integration tests
EOF
)"
```
