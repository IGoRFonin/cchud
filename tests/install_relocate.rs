//! Phase 9 Task 2 — relocation helpers tests.
//!
//! Все используют `tempfile::tempdir()`, не трогают `$HOME`.

#![allow(clippy::unwrap_used)]

use std::fs;

use cchud::commands::install::testing::{
    canonical_target_path, check_path_or_warn_capturing, relocate_to, same_file,
};
use tempfile::tempdir;

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
    assert!(same_file(&real, &link).unwrap(), "canonicalize should equate symlink and target");
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
