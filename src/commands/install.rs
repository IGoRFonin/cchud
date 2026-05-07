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
use std::{fs, io};

/// Outcome of `install_idempotent` — каждый вариант = другой UX/копирайт.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // used in T3 (event_loop), CLI not all variants
pub enum InstallStatus {
    /// settings.json не имел statusLine — записали свой.
    Installed,
    /// statusLine уже указывал на cchud — silent path-update.
    AlreadyConfigured,
    /// statusLine был занят сторонней командой и --force перезаписал.
    OverwroteForce,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // settings_path/binary_path consumed by T3
pub struct InstallReport {
    pub status: InstallStatus,
    pub settings_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub binary_path: PathBuf,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InstallError {
    /// statusLine занят чем-то ≠ cchud, --force не передан.
    OccupiedByOther { existing: String },
    Io(io::Error),
    Other(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OccupiedByOther { existing } => write!(f, "statusLine occupied by: {existing}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Other(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstallArgs {
    pub force: bool,
    pub no_relocate: bool,
}

#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    let install_args = InstallArgs {
        force: args.iter().any(|a| a == "--force"),
        no_relocate: args.iter().any(|a| a == "--no-relocate"),
    };

    match install_idempotent(&install_args) {
        Ok(report) => {
            let current = std::env::current_exe().unwrap_or_default();
            if !same_file(&report.binary_path, &current).unwrap_or(true) {
                println!("cchud: binary installed to {}", report.binary_path.display());
            }
            println!("cchud: wired into Claude Code");
            check_path_or_warn(&report.binary_path);
            ExitCode::SUCCESS
        }
        Err(InstallError::OccupiedByOther { existing }) => {
            eprintln!("cchud: statusLine already set to: {existing}");
            eprintln!("       use --force to overwrite, or remove it manually first.");
            ExitCode::from(1)
        }
        Err(InstallError::Io(e)) => {
            eprintln!("cchud install: cannot write settings.json: {e}");
            ExitCode::from(1)
        }
        Err(InstallError::Other(s)) => {
            eprintln!("cchud install: {s}");
            ExitCode::from(1)
        }
    }
}

/// TUI-friendly install: pure result, no stdout/stderr side effects (caller decides).
///
/// # Errors
/// Возвращает `InstallError::OccupiedByOther` если statusLine занят сторонней
/// командой без `--force`, `InstallError::Io` если IO упал, `InstallError::Other`
/// для прочих сбоев (не удалось определить путь к exe и т.д.).
pub fn install_idempotent(args: &InstallArgs) -> Result<InstallReport, InstallError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| InstallError::Other(format!("cannot determine executable path: {e}")))?;

    // Step 1: Self-relocate (если не --no-relocate).
    let final_exe = if args.no_relocate {
        current_exe
    } else {
        let target = canonical_target_path().map_err(InstallError::Io)?;
        if !same_file(&current_exe, &target).unwrap_or(false) {
            relocate_to(&current_exe, &target).map_err(InstallError::Io)?;
        }
        target
    };

    // Step 2: Determine status (pre-existing settings + statusLine).
    let path = settings_path();
    let pre_existed = path.exists();
    let pre_status_line = if pre_existed {
        read_or_empty(&path)
            .pointer("/statusLine/command")
            .and_then(serde_json::Value::as_str)
            .map(String::from)
    } else {
        None
    };

    if let Some(ref existing) = pre_status_line {
        if !existing.contains("cchud") && !args.force {
            return Err(InstallError::OccupiedByOther {
                existing: existing.clone(),
            });
        }
    }

    let status = match &pre_status_line {
        Some(s) if s.contains("cchud") => InstallStatus::AlreadyConfigured,
        Some(_) => InstallStatus::OverwroteForce,
        None => InstallStatus::Installed,
    };

    let backup_path = write_settings_with_exe_returning_backup(&final_exe, args.force)
        .map_err(InstallError::Io)?;

    Ok(InstallReport {
        status,
        settings_path: path,
        backup_path,
        binary_path: final_exe,
    })
}

fn write_settings_with_exe_returning_backup(
    exe: &Path,
    force: bool,
) -> std::io::Result<Option<PathBuf>> {
    let path = settings_path();
    let mut root = read_or_empty(&path);

    let backup_path = if path.exists() {
        let unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let pid = std::process::id();
        let mut bak = path.as_os_str().to_owned();
        bak.push(format!(".bak.{unix_ms}-{pid}"));
        let bak_path = PathBuf::from(bak);
        let copied = std::fs::copy(&path, &bak_path).is_ok();
        copied.then_some(bak_path)
    } else {
        None
    };

    if let Some(cmd) = root
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(serde_json::Value::as_str)
    {
        if !cmd.contains("cchud") && !force {
            return Err(std::io::Error::other("statusLine occupied"));
        }
    }

    root["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "padding": 0,
    });

    write_atomic(&path, &root)?;
    Ok(backup_path)
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

/// Returns the canonical target path где должен жить `cchud` после `cchud install`.
/// Unix: `$HOME/.local/bin/cchud`. Windows: `%LOCALAPPDATA%\cchud\cchud.exe`.
///
/// # Errors
/// Returns an error if `$HOME` / `%LOCALAPPDATA%` cannot be resolved.
pub fn canonical_target_path() -> io::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        // Prefer %LOCALAPPDATA% directly; fall back to dirs::data_local_dir() if the
        // env var is missing (e.g. in restricted/service accounts). The two may diverge
        // in non-standard configurations — documented in DECISIONS.md as accepted risk.
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(dirs::data_local_dir)
            .ok_or_else(|| io::Error::other("cannot resolve %LOCALAPPDATA%"))?;
        Ok(base.join("cchud").join("cchud.exe"))
    } else {
        let home = dirs::home_dir().ok_or_else(|| io::Error::other("no $HOME"))?;
        Ok(home.join(".local").join("bin").join("cchud"))
    }
}

/// Returns true iff `a` and `b` resolve to the same canonical path.
/// Returns false if either path does not exist.
///
/// # Errors
/// Returns an error if `fs::canonicalize` fails for an existing path.
pub fn same_file(a: &Path, b: &Path) -> io::Result<bool> {
    if !a.exists() || !b.exists() {
        return Ok(false);
    }
    Ok(fs::canonicalize(a)? == fs::canonicalize(b)?)
}

/// Copies `src` → `dst`, creating parent dir, setting mode 0755 on Unix.
/// Idempotent: works if `dst` already exists.
///
/// # Errors
/// Returns an error if directory creation, file copy, or permission setting fails.
pub fn relocate_to(src: &Path, dst: &Path) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dst)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dst, perms)?;
    }
    Ok(())
}

/// Returns `(stdout_string, stderr_string)`. Caller decides куда писать.
/// Pure: не делает println!/eprintln!.
#[must_use]
pub fn check_path_or_warn_capturing(
    target: &Path,
    path_var: &str,
    shell_name: &str,
) -> (String, String) {
    let Some(target_dir) = target.parent() else {
        return (String::new(), String::new());
    };
    let separator = if cfg!(windows) { ';' } else { ':' };
    // canonicalize errors on non-existent PATH entries are silently dropped (filter_map).
    // If target_dir itself does not exist, canonicalize fails and we compare against its
    // raw path — which won't match any canonical PATH entry, yielding a "not in PATH"
    // warning. This is acceptable: if the directory doesn't exist it truly isn't useful.
    let in_path = path_var
        .split(separator)
        .filter_map(|p| Path::new(p).canonicalize().ok())
        .any(|p| p == fs::canonicalize(target_dir).unwrap_or_else(|_| target_dir.to_path_buf()));

    if in_path {
        (
            format!("✓ {} is in PATH\n", target_dir.display()),
            String::new(),
        )
    } else {
        let mut stderr = format!("⚠ {} is not in PATH\n", target_dir.display());
        match shell_name {
            "zsh" => stderr
                .push_str("  Run: echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.zshrc\n"),
            "bash" => stderr
                .push_str("  Run: echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.bashrc\n"),
            "fish" => stderr.push_str("  Run: fish_add_path -U $HOME/.local/bin\n"),
            _ => stderr.push_str("  Add $HOME/.local/bin to your PATH manually\n"),
        }
        (String::new(), stderr)
    }
}

/// Real-side обёртка: печатает в stdout/stderr.
pub fn check_path_or_warn(target: &Path) {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_name = shell.split('/').next_back().unwrap_or("");
    let (out, err) = check_path_or_warn_capturing(target, &path_var, shell_name);
    if !out.is_empty() {
        print!("{out}");
    }
    if !err.is_empty() {
        eprint!("{err}");
    }
}

/// Test-only re-exports для integration tests.
#[doc(hidden)]
#[allow(unused_imports)]
pub mod testing {
    pub use super::{canonical_target_path, check_path_or_warn_capturing, relocate_to, same_file};
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests_idempotent {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn override_settings(path: &Path) {
        // SAFETY: serial_test gates these tests against parallel access to env.
        unsafe { std::env::set_var("CCHUD_SETTINGS", path); }
    }

    #[test]
    #[serial]
    fn returns_installed_when_settings_missing() {
        let tmp = TempDir::new().unwrap();
        override_settings(&tmp.path().join("settings.json"));
        let args = InstallArgs { force: false, no_relocate: true };
        let report = install_idempotent(&args).expect("install");
        assert_eq!(report.status, InstallStatus::Installed);
        assert!(report.backup_path.is_none());
    }

    #[test]
    #[serial]
    fn returns_already_configured_when_pointed_at_cchud() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        override_settings(&path);
        // Pre-write settings already wired to cchud-like path.
        std::fs::write(
            &path,
            r#"{"statusLine":{"type":"command","command":"/some/cchud/binary"}}"#,
        )
        .unwrap();
        let report =
            install_idempotent(&InstallArgs { no_relocate: true, ..Default::default() }).unwrap();
        assert_eq!(report.status, InstallStatus::AlreadyConfigured);
    }

    #[test]
    #[serial]
    fn returns_occupied_when_pointed_at_other_without_force() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        override_settings(&path);
        std::fs::write(
            &path,
            r#"{"statusLine":{"type":"command","command":"some/other/tool"}}"#,
        )
        .unwrap();

        let err =
            install_idempotent(&InstallArgs { no_relocate: true, ..Default::default() }).unwrap_err();
        match err {
            InstallError::OccupiedByOther { existing } => {
                assert!(existing.contains("some/other/tool"));
            }
            other => panic!("expected OccupiedByOther, got {other:?}"),
        }
    }
}
