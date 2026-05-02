//! Phase 9 Task 2 — relocation helpers tests.
//!
//! Все используют `tempfile::tempdir()`, не трогают `$HOME`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use cchud::commands::install::testing::{
    canonical_target_path, check_path_or_warn_capturing, relocate_to, same_file,
};
use tempfile::tempdir;

// Serialize tests that mutate process env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn same_file_distinguishes_different_paths() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    fs::write(&a, "alpha").unwrap();
    fs::write(&b, "beta").unwrap();
    assert!(!same_file(&a, &b).unwrap());
}

#[test]
fn same_file_same_path_returns_true() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("x");
    fs::write(&p, "data").unwrap();
    assert!(same_file(&p, &p).unwrap());
}

#[test]
fn same_file_handles_symlinks_correctly() {
    let dir = tempdir().unwrap();
    let real = dir.path().join("real");
    let link = dir.path().join("link");
    fs::write(&real, "data").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&real, &link).unwrap();
    assert!(
        same_file(&real, &link).unwrap(),
        "canonicalize should equate symlink and target"
    );
}

#[test]
fn same_file_returns_false_when_either_missing() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("missing");
    fs::write(&a, "x").unwrap();
    assert!(!same_file(&a, &b).unwrap());
}

#[test]
#[cfg(unix)]
fn canonical_target_path_unix_is_local_bin_cchud() {
    // Не зависит от env — функция работает с dirs::home_dir().
    let target = canonical_target_path().unwrap();
    let s = target.to_string_lossy();
    assert!(s.ends_with("/.local/bin/cchud"), "got: {s}");
}

#[test]
#[cfg(windows)]
fn canonical_target_path_windows_is_localappdata_cchud_exe() {
    let target = canonical_target_path().unwrap();
    let s = target.to_string_lossy();
    assert!(s.ends_with("\\cchud\\cchud.exe"), "got: {s}");
}

#[test]
fn relocate_to_copies_file_and_creates_parent() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src_bin");
    let dst = dir.path().join("nested/path/cchud");
    fs::write(&src, b"fake-binary").unwrap();
    relocate_to(&src, &dst).unwrap();
    assert!(dst.exists());
    assert_eq!(fs::read(&dst).unwrap(), b"fake-binary");
}

#[test]
#[cfg(unix)]
fn relocate_to_sets_mode_0755_on_unix() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    let dst = dir.path().join("dst");
    fs::write(&src, b"x").unwrap();
    relocate_to(&src, &dst).unwrap();
    let mode = fs::metadata(&dst).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o755, "expected 0o755, got {mode:o}");
}

#[test]
fn relocate_to_idempotent_overwrite() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    let dst = dir.path().join("dst");
    fs::write(&src, b"alpha").unwrap();
    relocate_to(&src, &dst).unwrap();
    fs::write(&src, b"beta").unwrap();
    relocate_to(&src, &dst).unwrap(); // overwrite OK
    assert_eq!(fs::read(&dst).unwrap(), b"beta");
}

#[test]
fn check_path_or_warn_in_path_emits_check() {
    let dir = tempdir().unwrap();
    let bin_dir = dir.path().to_path_buf();
    let target = bin_dir.join("cchud");
    fs::write(&target, b"x").unwrap();
    let path_var = bin_dir.to_string_lossy().to_string();
    let (stdout, stderr) = check_path_or_warn_capturing(&target, &path_var, "zsh");
    assert!(stdout.contains("is in PATH"), "stdout: {stdout}");
    assert!(stderr.is_empty(), "stderr: {stderr}");
}

#[test]
fn check_path_or_warn_not_in_path_emits_zsh_hint() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("subdir/cchud");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"x").unwrap();
    let (_, stderr) = check_path_or_warn_capturing(&target, "/usr/bin:/bin", "zsh");
    assert!(stderr.contains("not in PATH"), "stderr: {stderr}");
    assert!(stderr.contains(".zshrc"), "stderr: {stderr}");
}

#[test]
fn check_path_or_warn_not_in_path_emits_fish_hint() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("subdir/cchud");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"x").unwrap();
    let (_, stderr) = check_path_or_warn_capturing(&target, "/usr/bin:/bin", "fish");
    assert!(stderr.contains("fish_add_path"), "stderr: {stderr}");
}

// ---------- High-level integration tests for `commands::install::run` ----------
//
// Используем CCHUD_SETTINGS env var (existing escape hatch в settings_path())
// чтобы redirect ~/.claude/settings.json в tempdir.
// Используем --no-relocate чтобы skip копирование (избежать write в $HOME/.local/bin).

#[test]
fn run_no_relocate_writes_settings_and_returns_zero() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");

    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    let args = vec!["--no-relocate".to_string()];
    let exit = cchud::commands::install::run(&args);
    assert_eq!(
        format!("{exit:?}"),
        format!("{:?}", std::process::ExitCode::SUCCESS)
    );

    let body = fs::read_to_string(&settings_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let cmd = v
        .get("statusLine")
        .unwrap()
        .get("command")
        .unwrap()
        .as_str()
        .unwrap();
    // Under `cargo test`, current_exe() points to the test runner binary, not a cchud
    // binary, so the path may contain "test_runner" rather than "cchud".
    assert!(
        cmd.contains("cchud") || cmd.contains("test_runner"),
        "cmd: {cmd}"
    );
}

#[test]
fn run_idempotent_second_call_is_no_op_logically() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    let args = vec!["--no-relocate".to_string()];
    let _ = cchud::commands::install::run(&args);
    let first = fs::read_to_string(&settings_path).unwrap();
    let _ = cchud::commands::install::run(&args);
    let second = fs::read_to_string(&settings_path).unwrap();
    assert_eq!(
        first, second,
        "second run should produce identical settings.json"
    );
}

#[test]
fn run_force_overwrites_non_cchud_status_line() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    fs::write(
        &settings_path,
        r#"{"statusLine":{"type":"command","command":"ccstatusline"}}"#,
    )
    .unwrap();

    // Без --force должен fail.
    let exit_no_force = cchud::commands::install::run(&["--no-relocate".to_string()]);
    assert_eq!(
        format!("{exit_no_force:?}"),
        format!("{:?}", std::process::ExitCode::from(1))
    );

    // С --force должен пройти.
    let exit_force =
        cchud::commands::install::run(&["--force".to_string(), "--no-relocate".to_string()]);
    assert_eq!(
        format!("{exit_force:?}"),
        format!("{:?}", std::process::ExitCode::SUCCESS)
    );

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let cmd = v
        .get("statusLine")
        .unwrap()
        .get("command")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(!cmd.contains("ccstatusline"));
}

#[test]
fn run_creates_bak_file_when_settings_already_exists() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    fs::write(
        &settings_path,
        r#"{"statusLine":{"type":"command","command":"cchud-old"}}"#,
    )
    .unwrap();

    let exit = cchud::commands::install::run(&["--no-relocate".to_string()]);
    assert_eq!(
        format!("{exit:?}"),
        format!("{:?}", std::process::ExitCode::SUCCESS)
    );

    // Должен существовать хотя бы один файл с prefix settings.json.bak.
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("settings.json.bak."))
        .collect();
    assert!(
        !entries.is_empty(),
        "expected at least one backup, got: {entries:?}"
    );

    let bak_name = entries.first().unwrap();
    let bak_body = fs::read_to_string(dir.path().join(bak_name)).unwrap();
    assert!(
        bak_body.contains("cchud-old"),
        "backup should contain prev content; got: {bak_body}"
    );
}

#[test]
fn run_no_backup_when_settings_did_not_exist() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    let _ = cchud::commands::install::run(&["--no-relocate".to_string()]);

    let has_bak = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .any(|n| n.contains(".bak."));
    assert!(!has_bak, "no backup expected if file did not exist");
}

// Tiny RAII helper для env vars в tests.
struct EnvVarGuard {
    key: &'static str,
    prev: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set<P: AsRef<Path>>(key: &'static str, val: P) -> Self {
        let prev = std::env::var_os(key);
        // SAFETY: тесты запускаются в однопоточном контексте (cargo test --test …).
        unsafe { std::env::set_var(key, val.as_ref()) };
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            // SAFETY: тесты однопоточны.
            Some(v) => unsafe { std::env::set_var(self.key, v) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}
