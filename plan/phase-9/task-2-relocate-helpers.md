# Task 2 — Relocate helpers + tests (TDD)

**Цель:** Реализовать pure helper'ы для self-relocation в `src/commands/install.rs`: `canonical_target_path`, `same_file`, `relocate_to`, `check_path_or_warn`. Все pure (используют `tempfile::tempdir()`, не трогают `$HOME`). TDD: ≥8 unit-тестов.

**Files:**
- Modify: `src/commands/install.rs` — добавить helper'ы (но `pub fn run` пока не трогаем — Task 3).
- Create: `tests/install_relocate.rs` — integration tests (≥8 кейсов).

---

- [ ] **Step 1: Написать failing tests первыми (TDD)**

Создать файл `tests/install_relocate.rs`:

```rust
//! Phase 9 Task 2 — relocation helpers tests.
//!
//! Все используют `tempfile::tempdir()`, не трогают `$HOME`.

#![allow(clippy::unwrap_used)]

// Делаем helper'ы доступными через `pub(crate)` из crate root.
// install.rs регистрирует test-only re-export.

use std::fs;
use std::path::Path;

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
```

Сохранить файл.

- [ ] **Step 2: Run tests — confirm они FAIL**

```bash
cargo test --locked --test install_relocate
```

Expected: build error — `cchud::commands::install::testing` не существует, helper'ы не реализованы.

- [ ] **Step 3: Реализовать helpers в `src/commands/install.rs`**

В конец файла (после существующего `write_atomic`) добавить:

```rust
use std::path::PathBuf;
use std::{fs, io};

/// Returns the canonical target path где должен жить `cchud` после `cchud install`.
/// Unix: `$HOME/.local/bin/cchud`. Windows: `%LOCALAPPDATA%\cchud\cchud.exe`.
pub(crate) fn canonical_target_path() -> io::Result<PathBuf> {
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
pub(crate) fn same_file(a: &Path, b: &Path) -> io::Result<bool> {
    if !a.exists() || !b.exists() {
        return Ok(false);
    }
    Ok(fs::canonicalize(a)? == fs::canonicalize(b)?)
}

/// Copies `src` → `dst`, creating parent dir, setting mode 0755 on Unix.
/// Idempotent: works if `dst` already exists.
pub(crate) fn relocate_to(src: &Path, dst: &Path) -> io::Result<()> {
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
/// Pure: не делает println!/eprintln!. Real entry point делает обёртку.
pub(crate) fn check_path_or_warn_capturing(
    target: &Path,
    path_var: &str,
    shell_name: &str,
) -> (String, String) {
    let target_dir = match target.parent() {
        Some(d) => d,
        None => return (String::new(), String::new()),
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
            "zsh" => stderr.push_str(
                "  Run: echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.zshrc\n",
            ),
            "bash" => stderr.push_str(
                "  Run: echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.bashrc\n",
            ),
            "fish" => stderr.push_str("  Run: fish_add_path -U $HOME/.local/bin\n"),
            _ => stderr.push_str("  Add $HOME/.local/bin to your PATH manually\n"),
        }
        (String::new(), stderr)
    }
}

/// Real-side обёртка: печатает в stdout/stderr.
pub(crate) fn check_path_or_warn(target: &Path) {
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
pub mod testing {
    pub use super::{
        canonical_target_path, check_path_or_warn_capturing, relocate_to, same_file,
    };
}
```

- [ ] **Step 4: Прокинуть `lib.rs` чтобы tests могли import'ить `cchud::commands::install`**

Прочитать `src/lib.rs`. Если `pub mod commands;` отсутствует или ограничено feature-flag — добавить безусловно (чтобы integration tests видели `install`):

```rust
pub mod commands;
```

(Если `commands::install` уже public через lib — пропустить. Если `mod.rs` объявляет всё `pub mod`, всё ОК.)

- [ ] **Step 5: Run tests — confirm они PASS**

```bash
cargo test --locked --test install_relocate
```

Expected: 12+ tests PASS.

- [ ] **Step 6: Run полный test suite чтобы убедиться, что ничего не сломалось**

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Expected: оба зелёные. Никаких warnings.

- [ ] **Step 7: Commit**

```bash
git add src/commands/install.rs src/lib.rs tests/install_relocate.rs
git commit -m "$(cat <<'EOF'
feat(phase-9): T2 relocate helpers — canonical_target_path/same_file/relocate_to/check_path_or_warn

Pure helpers for cchud install self-relocation (used by T3).
12 unit tests covering symlinks, idempotent overwrite, mode 0755,
shell-specific PATH hints (zsh/bash/fish).
EOF
)"
```
