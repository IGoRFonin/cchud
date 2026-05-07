# Task 4 — Diff stat (GitInsertions, GitDeletions)

**Files:**
- Modify: `src/git/mod.rs` (impl `GitInfo::diff_stat() -> Option<&DiffStat>` через `OnceCell`)
- Create: `src/widgets/git_diff.rs` (2 widget impl)
- Modify: `src/widgets/mod.rs` (`pub mod git_diff;` + 2 match-arms)
- Modify: `src/types/config.rs` (2 enum-варианта: `GitInsertions`, `GitDeletions`)

## Goal

Diff stat — самая дорогая git-операция (`gix::diff` через blame/diff API ~5-15 ms на крупных изменениях). Делаем её **ленивой**: `OnceCell` рассчитывается только если `GitInsertions` или `GitDeletions` есть в строке. Без этих виджетов — нулевая стоимость.

| Widget | Render | Note |
|---|---|---|
| `GitInsertions` | `+{insertions}` или None если 0 | Сумма + по unstaged + staged diff |
| `GitDeletions` | `-{deletions}` или None если 0 | Сумма − по unstaged + staged diff |

> **Что считаем**: суммарный insertions/deletions по diff между HEAD и working tree (unstaged + staged). Это ccstatusline upstream-поведение.

## Inputs

- T3 закрыт.
- `cargo test --locked widgets::git_status` зелёный.

---

- [ ] **Step 1: Расширить `WidgetConfig` 2 вариантами**

Edit `src/types/config.rs`:
- `old_string`:
  ```rust
      // Phase 5 — Task 3 (status cluster):
      GitStatus,
      GitChanges,
      GitStaged,
      GitUnstaged,
      GitUntracked,
      GitConflicts,
  }
  ```
- `new_string`:
  ```rust
      // Phase 5 — Task 3 (status cluster):
      GitStatus,
      GitChanges,
      GitStaged,
      GitUnstaged,
      GitUntracked,
      GitConflicts,
      // Phase 5 — Task 4 (diff stat):
      GitInsertions,
      GitDeletions,
  }
  ```

Добавить тест:
```rust
#[test]
fn parses_phase5_diff_widgets() {
    let json = r#"[
        { "type": "git-insertions" },
        { "type": "git-deletions" }
    ]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0], WidgetConfig::GitInsertions));
    assert!(matches!(widgets[1], WidgetConfig::GitDeletions));
}
```

- [ ] **Step 2: Реализовать `GitInfo::diff_stat()`**

Edit `src/git/mod.rs`. Добавить рядом с `status_counts()`:

```rust
impl GitInfo {
    /// Lazy: diff stat между HEAD-tree и working-tree. None если diff
    /// невозможен (unborn HEAD, gix error). Стоимость ~5-15 ms на изменения,
    /// поэтому вызывается только из виджетов GitInsertions/GitDeletions.
    pub fn diff_stat(&self) -> Option<&DiffStat> {
        self.diff_stat
            .get_or_init(|| compute_diff_stat(&self.repo))
            .as_ref()
    }
}

fn compute_diff_stat(repo: &gix::Repository) -> Option<DiffStat> {
    let head_tree = repo.head_tree_id().ok()?.object().ok()?.into_tree();
    // Сравниваем HEAD tree с index (staged) — это "что закоммичено vs готово к коммиту".
    // Plus: index vs working tree (unstaged) — добавляем сверху.
    let mut stat = DiffStat::default();

    // 1. HEAD vs index.
    if let Ok(idx) = repo.index() {
        if let Ok(diff) = head_tree.changes() {
            // gix 0.81 diff API — может быть platform-style.
            // Псевдокод; реальная реализация может использовать gix::diff::tree_to_index.
            let _ = diff;
            let _ = idx;
        }
    }

    // 2. Index vs working tree через status iterator с включённым diff stat.
    // Если gix 0.81 поддерживает inline stat в status iter — использовать его.
    // Иначе fallback на shell-out (см. Risks).

    // Заглушка: если diff API окажется тяжёлым — реализуем через shell-out
    // на `git diff --shortstat HEAD`. См. Step 3 fallback.

    if stat.insertions == 0 && stat.deletions == 0 {
        // Если gix-путь не дал результата — fallback shell-out.
        if let Some(s) = compute_diff_stat_shell(repo.work_dir()?) {
            stat = s;
        }
    }

    Some(stat)
}

/// Fallback: `git diff --shortstat HEAD` парсится из stdout.
/// Использовать только если gix-путь оказался неподъёмным.
fn compute_diff_stat_shell(cwd: &std::path::Path) -> Option<DiffStat> {
    use std::process::Command;
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["diff", "--shortstat", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);
    // Формат: " 2 files changed, 5 insertions(+), 3 deletions(-)"
    let mut stat = DiffStat::default();
    for part in s.split(',') {
        let part = part.trim();
        if let Some(n) = part.strip_suffix(" insertions(+)").or_else(|| part.strip_suffix(" insertion(+)")) {
            stat.insertions = n.trim().parse().unwrap_or(0);
        } else if let Some(n) = part.strip_suffix(" deletions(-)").or_else(|| part.strip_suffix(" deletion(-)")) {
            stat.deletions = n.trim().parse().unwrap_or(0);
        }
    }
    Some(stat)
}
```

> **Pragmatism**: gix 0.81 diff API менее зрелый, чем status. Допустимо в Phase 5 использовать **только** shell-out fallback, оставив gix-путь как TODO для Phase 6+. Записать в DECISIONS.

> **Если выбираем shell-out only** (рекомендация): упростить `compute_diff_stat` до прямого вызова `compute_diff_stat_shell(repo.work_dir()?)`. Стоимость ~3-5 ms одного subprocess, приемлемо для 2-х виджетов (вызов один раз через OnceCell).

- [ ] **Step 3: Тесты для diff stat**

Добавить в `src/git/mod.rs` `mod tests`:

```rust
#[test]
fn diff_stat_zero_for_clean_repo() {
    let f = GitFixture::new();
    let info = GitInfo::discover(f.path()).unwrap();
    let d = info.diff_stat().expect("must compute");
    assert_eq!(d.insertions, 0);
    assert_eq!(d.deletions, 0);
}

#[test]
fn diff_stat_counts_unstaged_insertions() {
    let f = GitFixture::new();
    f.write_file("a.txt", "line1\nline2\n");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.write_file("a.txt", "line1\nline2\nline3\nline4\n");
    let info = GitInfo::discover(f.path()).unwrap();
    let d = info.diff_stat().unwrap();
    assert_eq!(d.insertions, 2, "expected +2 lines");
    assert_eq!(d.deletions, 0);
}

#[test]
fn diff_stat_counts_unstaged_deletions() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1\n2\n3\n");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.write_file("a.txt", "1\n");
    let info = GitInfo::discover(f.path()).unwrap();
    let d = info.diff_stat().unwrap();
    assert_eq!(d.deletions, 2);
    assert_eq!(d.insertions, 0);
}

#[test]
fn diff_stat_cached() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1\n");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.write_file("a.txt", "1\n2\n");
    let info = GitInfo::discover(f.path()).unwrap();
    let first = info.diff_stat().unwrap().insertions;
    f.write_file("a.txt", "1\n2\n3\n");
    let second = info.diff_stat().unwrap().insertions;
    assert_eq!(first, second);
}
```

- [ ] **Step 4: Создать `src/widgets/git_diff.rs`**

```rust
//! Git diff stat — Phase 5 Task 4.
//!
//! Lazy: GitInfo::diff_stat() считается только при первом запросе.
//! Без этих 2 виджетов в строке — нулевая стоимость.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitInsertions;
pub struct GitDeletions;

impl Widget for GitInsertions {
    fn id(&self) -> &'static str { "GitInsertions" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let n = ctx.git()?.diff_stat()?.insertions;
        if n == 0 { None } else { Some(format!("+{n}")) }
    }
}

impl Widget for GitDeletions {
    fn id(&self) -> &'static str { "GitDeletions" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let n = ctx.git()?.diff_stat()?.deletions;
        if n == 0 { None } else { Some(format!("-{n}")) }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::git::fixture::GitFixture;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload(cwd: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo { id: "m".into(), display_name: "M".into() },
            workspace: Workspace { current_dir: cwd.into(), project_dir: None, added_dirs: None },
            transcript_path: None, cwd: None, version: None, fast_mode: None,
            exceeds_200k_tokens: None, output_style: None, cost: None,
            context_window: None, worktree: None, vim: None,
            rate_limits: None, effort: None, thinking: None,
        }
    }

    #[test]
    fn insertions_renders_plus_prefix() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n2\n3\n");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx).as_deref(), Some("+2"));
        assert_eq!(GitDeletions.render(&ctx), None);
    }

    #[test]
    fn deletions_renders_minus_prefix() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1\n2\n3\n");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "1\n");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitDeletions.render(&ctx).as_deref(), Some("-2"));
        assert_eq!(GitInsertions.render(&ctx), None);
    }

    #[test]
    fn zero_diff_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx), None);
        assert_eq!(GitDeletions.render(&ctx), None);
    }

    #[test]
    fn outside_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitInsertions.render(&ctx), None);
        assert_eq!(GitDeletions.render(&ctx), None);
    }
}
```

- [ ] **Step 5: Регистрация**

Edit `src/widgets/mod.rs`:

Edit 1: добавить `pub mod git_diff;` после `pub mod git_status;`.

Edit 2:
- `old_string`:
  ```rust
          WidgetConfig::GitConflicts => Box::new(git_status::GitConflicts),
      }
  ```
- `new_string`:
  ```rust
          WidgetConfig::GitConflicts => Box::new(git_status::GitConflicts),
          // Phase 5 — Task 4 (diff stat):
          WidgetConfig::GitInsertions => Box::new(git_diff::GitInsertions),
          WidgetConfig::GitDeletions => Box::new(git_diff::GitDeletions),
      }
  ```

- [ ] **Step 6: Тесты + standard gate**

```bash
cargo test --locked --lib widgets::git_diff
cargo test --locked --lib git::tests::diff_stat
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 7: Verification**

```bash
grep -c 'pub fn diff_stat' src/git/mod.rs
grep -c 'GitInsertions' src/widgets/git_diff.rs
grep -c 'GitDeletions' src/widgets/git_diff.rs
grep -c 'git_diff::GitInsertions' src/widgets/mod.rs
```

Expected: каждый ≥1.

- [ ] **Step 8: Commit**

```bash
git add src/git/mod.rs src/widgets/git_diff.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T4 diff stat — GitInsertions, GitDeletions

GitInfo::diff_stat() is OnceCell-cached, lazy. Computed only when
GitInsertions or GitDeletions is in the line — zero cost otherwise.

Implementation falls back to 'git diff --shortstat HEAD' shell-out
because gix 0.81 diff API is less mature than status (~5 ms cost,
acceptable for one-off cache-fill).

Render format: '+N' / '-N' (matches ccstatusline upstream). Zero
counts render as None.

Task 4/9 of Phase 5. Cumulative: 11/20.
"
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `GitInfo::diff_stat()` через `OnceCell`, lazy
- [ ] `src/widgets/git_diff.rs` с `GitInsertions` и `GitDeletions`
- [ ] 2 enum-варианта парсятся как `git-insertions`/`git-deletions`
- [ ] Чистый репо → оба = None
- [ ] +2 строки в файле → `GitInsertions = "+2"`, `GitDeletions = None`
- [ ] −2 строки → `GitDeletions = "-2"`, `GitInsertions = None`
- [ ] Тест `diff_stat_cached` подтверждает идемпотентность
- [ ] ≥4 unit-теста в `git::tests::diff_stat*` + 4 в `widgets::git_diff::tests`
- [ ] Один commit `feat(phase-5): T4 diff stat ...`

## Files touched

- `src/git/mod.rs` (modified — `compute_diff_stat`, `diff_stat()`)
- `src/widgets/git_diff.rs` (created)
- `src/widgets/mod.rs` (modified)
- `src/types/config.rs` (modified — 2 enum variants)

## Risks & rollback

- **Shell-out на `git diff` через subprocess** — лишний fork (~3-5 ms). Митигация: вызывается ровно один раз (OnceCell), только если виджеты в строке. T8 hyperfine проверит, что budget 8 ms держится.
- **`git diff --shortstat HEAD` парсинг** — формат стабильный, но локализация может сломать ("ins. einf"). Митигация: процесс получает `LANG=C` env (TODO добавить в Step 2 если в CI ломается).
- **Unborn HEAD**: `git diff HEAD` упадёт. `compute_diff_stat_shell` возвращает None через `status.success()` check — ОК.
- **gix-путь оживить позже**: TODO в комментарии `compute_diff_stat` — если в Phase 7+ gix 0.85+ упростит diff API, заменим shell-out.
- **Rollback**: `git revert HEAD` — снимает 2 виджета.
