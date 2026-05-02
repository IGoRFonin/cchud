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

#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    let force = args.iter().any(|a| a == "--force");
    let no_relocate = args.iter().any(|a| a == "--no-relocate");

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: cannot determine executable path: {e}");
            return ExitCode::from(1);
        }
    };

    // Step 1: Self-relocate (если не --no-relocate).
    let final_exe = if no_relocate {
        current_exe
    } else {
        let target = match canonical_target_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cchud install: cannot resolve target path: {e}");
                return ExitCode::from(1);
            }
        };
        if same_file(&current_exe, &target).unwrap_or(false) {
            // Already at target — no-op.
        } else if let Err(e) = relocate_to(&current_exe, &target) {
            eprintln!("cchud install: relocation failed: {e}");
            return ExitCode::from(1);
        } else {
            println!("cchud: binary installed to {}", target.display());
        }
        target
    };

    // Step 2: Wire ~/.claude/settings.json.
    if let Err(e) = write_settings_with_exe(&final_exe, force) {
        eprintln!("cchud install: cannot write settings.json: {e}");
        return ExitCode::from(1);
    }
    println!("cchud: wired into Claude Code");

    // Step 3: PATH check (warning не валит exit).
    check_path_or_warn(&final_exe);

    ExitCode::SUCCESS
}

fn write_settings_with_exe(exe: &Path, force: bool) -> std::io::Result<()> {
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
            return Err(std::io::Error::other("statusLine occupied"));
        }
    }

    root["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "padding": 0,
    });

    write_atomic(&path, &root)
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
#[allow(dead_code)] // used by install::run in T3
pub fn canonical_target_path() -> io::Result<PathBuf> {
    if cfg!(target_os = "windows") {
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
#[allow(dead_code)] // used by install::run in T3
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
#[allow(dead_code)] // used by install::run in T3
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
#[allow(dead_code)] // used by install::run in T3
pub fn check_path_or_warn_capturing(
    target: &Path,
    path_var: &str,
    shell_name: &str,
) -> (String, String) {
    let Some(target_dir) = target.parent() else {
        return (String::new(), String::new());
    };
    let separator = if cfg!(windows) { ';' } else { ':' };
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
#[allow(dead_code)] // used by install::run in T3
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
