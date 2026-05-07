//! Test fixture helpers.
//!
//! Создаёт tempfile-репозитории через `git` CLI (cross-platform, доступен
//! на macos/linux/windows CI runners). Использовать gix для setup'а
//! фикстур избыточно: тесты должны быть быстрыми и читаемыми, а gix init
//! API меняется между minor.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::missing_panics_doc)]

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Lightweight git-репо в tempdir.
pub struct GitFixture {
    pub dir: TempDir,
}

impl Default for GitFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl GitFixture {
    /// Создать пустой git-репо с одним commit'ом и веткой `main`.
    #[must_use]
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        run(dir.path(), &["init", "-b", "main"]);
        run(dir.path(), &["config", "user.email", "test@cchud.dev"]);
        run(dir.path(), &["config", "user.name", "cchud-test"]);
        // Один пустой commit, чтобы HEAD ссылался на коммит, а не unborn ref.
        run(dir.path(), &["commit", "--allow-empty", "-m", "init"]);
        Self { dir }
    }

    /// Путь к рабочему дереву.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Написать файл в рабочее дерево.
    pub fn write_file(&self, rel: &str, content: &str) {
        let abs = self.dir.path().join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&abs, content).unwrap();
    }

    /// Запустить произвольную git-команду в репо.
    pub fn git(&self, args: &[&str]) {
        run(self.dir.path(), args);
    }

    /// Создать commit (требует предварительный `add`).
    pub fn commit(&self, msg: &str) {
        run(self.dir.path(), &["commit", "-m", msg]);
    }

    /// Добавить remote.
    pub fn add_remote(&self, name: &str, url: &str) {
        run(self.dir.path(), &["remote", "add", name, url]);
    }
}

fn run(cwd: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: stdout={} stderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_creates_repo_with_initial_commit() {
        let f = GitFixture::new();
        assert!(f.path().join(".git").is_dir());
        let out = Command::new("git")
            .current_dir(f.path())
            .args(["log", "--oneline"])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("init"),
            "expected initial commit, got {stdout}"
        );
    }

    #[test]
    fn fixture_supports_dirty_working_tree() {
        let f = GitFixture::new();
        f.write_file("README.md", "# test");
        let out = Command::new("git")
            .current_dir(f.path())
            .args(["status", "--porcelain"])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("?? README.md"));
    }
}
