# Task 3 — Status cluster (GitStatus, GitChanges, GitStaged, GitUnstaged, GitUntracked, GitConflicts)

**Files:**
- Modify: `src/git/mod.rs` (impl `GitInfo::status_counts() -> Option<&GitStatusCounts>` через `OnceCell`)
- Create: `src/widgets/git_status.rs` (6 widget impl)
- Modify: `src/widgets/mod.rs` (`pub mod git_status;` + 6 match-arms)
- Modify: `src/types/config.rs` (6 enum-вариантов: `GitStatus`, `GitChanges`, `GitStaged`, `GitUnstaged`, `GitUntracked`, `GitConflicts`)

## Goal

Шесть виджетов делят **один** gix `status()` вызов через `OnceCell`. Это критично для производительности: `gix::status` на репе среднего размера ~1-2 ms, на монорепо до 10 ms — мы хотим заплатить эту цену максимум один раз.

| Widget | Render | Note |
|---|---|---|
| `GitStatus` | `"M{staged} ?{untracked} ✗{conflicts}"` или None если нет изменений | Summary-строка |
| `GitChanges` | `total()` — сумма всех 4 категорий, или None если 0 | Один счётчик |
| `GitStaged` | `staged` или None если 0 | |
| `GitUnstaged` | `unstaged` или None если 0 | |
| `GitUntracked` | `untracked` или None если 0 | |
| `GitConflicts` | `conflicts` или None если 0 | |

> **Naming**: `WidgetConfig::GitStatus` — виджет; `git::GitStatusCounts` — структура данных. Сознательно разные имена.

## Inputs

- T2 закрыт: `GitInfo::head` доступен; head-виджеты работают.
- `cargo test --locked widgets::git_head` зелёный.

---

- [ ] **Step 1: Расширить `WidgetConfig` 6 вариантами**

Edit `src/types/config.rs`:
- `old_string`:
  ```rust
      // Phase 5 — Task 2 (head cluster):
      GitBranch,
      GitSha,
      GitRootDir,
  }
  ```
- `new_string`:
  ```rust
      // Phase 5 — Task 2 (head cluster):
      GitBranch,
      GitSha,
      GitRootDir,
      // Phase 5 — Task 3 (status cluster):
      GitStatus,
      GitChanges,
      GitStaged,
      GitUnstaged,
      GitUntracked,
      GitConflicts,
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

Добавить тест:
```rust
#[test]
fn parses_phase5_status_widgets() {
    let json = r#"[
        { "type": "git-status" },
        { "type": "git-changes" },
        { "type": "git-staged" },
        { "type": "git-unstaged" },
        { "type": "git-untracked" },
        { "type": "git-conflicts" }
    ]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert_eq!(widgets.len(), 6);
    assert!(matches!(widgets[0], WidgetConfig::GitStatus));
    assert!(matches!(widgets[5], WidgetConfig::GitConflicts));
}
```

- [ ] **Step 2: Реализовать `GitInfo::status_counts()` через `OnceCell`**

Edit `src/git/mod.rs`:

Добавить метод после `discover`:

- `old_string`:
  ```rust
  fn parse_head(repo: &gix::Repository) -> Head {
  ```
- `new_string`:
  ```rust
  impl GitInfo {
      /// Lazy: считает status один раз, кэширует. None если gix вернул ошибку.
      pub fn status_counts(&self) -> Option<&GitStatusCounts> {
          self.status_counts
              .get_or_init(|| compute_status(&self.repo))
              .as_ref()
      }
  }

  fn compute_status(repo: &gix::Repository) -> Option<GitStatusCounts> {
      use gix::bstr::ByteSlice;

      let mut counts = GitStatusCounts::default();
      // gix 0.81 status iterator API: repo.status(progress::Discard).ok()?.into_iter(None).ok()?;
      // Точные имена методов могут отличаться — поправить локально.
      let status = repo.status(gix::progress::Discard).ok()?;
      let iter = status.into_index_worktree_iter(None).ok()?;

      for item in iter.flatten() {
          use gix::status::index_worktree::iter::Item;
          match item {
              Item::Modification { rewrite: Some(_), .. } => counts.staged += 1,
              Item::Modification { entry_mode_change, .. }
                  if entry_mode_change.is_some() => counts.staged += 1,
              Item::Modification { .. } => counts.unstaged += 1,
              Item::DirectoryContents { entry, .. }
                  if entry.status == gix::dir::entry::Status::Untracked =>
                  counts.untracked += 1,
              Item::DirectoryContents { .. } => {}
              Item::Rewrite { .. } => counts.staged += 1,
          }
      }

      // Conflicts отдельно: gix::index::entry::Stage != Unconflicted.
      if let Ok(idx) = repo.index() {
          for entry in idx.entries() {
              if entry.stage() != gix::index::entry::Stage::Unconflicted {
                  counts.conflicts += 1;
              }
          }
      }
      // Unique conflicts: каждый конфликтный файл имеет 3 stage-записи в индексе.
      counts.conflicts /= 3;

      Some(counts)
  }

  fn parse_head(repo: &gix::Repository) -> Head {
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/git/mod.rs`

> **Critical**: реальные имена методов в gix 0.81 (`status()`, `into_index_worktree_iter`, `Item` enum variants) — нужно сверить с актуальным API при первом `cargo build`. Контракт фиксирован тестами ниже; если gix даёт "stat staged/unstaged/untracked/conflicts" в одной структуре проще — использовать её. Допустимо упростить через `gix::status::Platform::iter()` если так короче.

> **Fallback strategy**: если gix-status API окажется слишком плотным/нестабильным, разрешено shell-out на `git status --porcelain=v1` через `Command` (cost ~3-5 ms — приемлемо для T8 budget 8 ms но без запаса). Записать как D-2026-04-XX в `docs/DECISIONS.md` если выберем этот fallback.

- [ ] **Step 3: Тесты для `compute_status`**

Добавить в `src/git/mod.rs` `mod tests`:

```rust
#[test]
fn status_counts_clean_repo() {
    let f = GitFixture::new();
    let info = GitInfo::discover(f.path()).unwrap();
    let s = info.status_counts().expect("must compute");
    assert_eq!(s.total(), 0);
}

#[test]
fn status_counts_untracked_file() {
    let f = GitFixture::new();
    f.write_file("new.txt", "x");
    let info = GitInfo::discover(f.path()).unwrap();
    let s = info.status_counts().unwrap();
    assert_eq!(s.untracked, 1);
    assert_eq!(s.total(), 1);
}

#[test]
fn status_counts_staged_file() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1");
    f.git(&["add", "a.txt"]);
    let info = GitInfo::discover(f.path()).unwrap();
    let s = info.status_counts().unwrap();
    assert_eq!(s.staged, 1);
    assert_eq!(s.untracked, 0);
}

#[test]
fn status_counts_unstaged_modification() {
    let f = GitFixture::new();
    f.write_file("a.txt", "1");
    f.git(&["add", "a.txt"]);
    f.commit("c2");
    f.write_file("a.txt", "2");
    let info = GitInfo::discover(f.path()).unwrap();
    let s = info.status_counts().unwrap();
    assert_eq!(s.unstaged, 1);
    assert_eq!(s.staged, 0);
}

#[test]
fn status_counts_cached_after_first_call() {
    let f = GitFixture::new();
    f.write_file("new.txt", "x");
    let info = GitInfo::discover(f.path()).unwrap();
    let first = info.status_counts().unwrap().total();
    // Изменим файл — но кэш OnceCell должен вернуть старое значение.
    f.write_file("new2.txt", "y");
    let second = info.status_counts().unwrap().total();
    assert_eq!(first, second, "OnceCell must NOT re-compute");
}
```

- [ ] **Step 4: Создать `src/widgets/git_status.rs`**

Create `/Users/igor/mp/startup/cchud/src/widgets/git_status.rs`:

```rust
//! Git status cluster — Phase 5 Task 3.
//!
//! 6 виджетов делят один gix-status вызов через `GitInfo::status_counts`
//! (`OnceCell`). Стоимость status'а — ~1-2 ms на репе среднего размера,
//! платится максимум один раз за render.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitStatus;
pub struct GitChanges;
pub struct GitStaged;
pub struct GitUnstaged;
pub struct GitUntracked;
pub struct GitConflicts;

fn nonzero(n: u32) -> Option<String> {
    if n == 0 { None } else { Some(n.to_string()) }
}

impl Widget for GitStatus {
    fn id(&self) -> &'static str { "GitStatus" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let s = ctx.git()?.status_counts()?;
        if s.total() == 0 {
            return None;
        }
        let mut parts: Vec<String> = Vec::with_capacity(4);
        if s.staged > 0 { parts.push(format!("M{}", s.staged)); }
        if s.unstaged > 0 { parts.push(format!("~{}", s.unstaged)); }
        if s.untracked > 0 { parts.push(format!("?{}", s.untracked)); }
        if s.conflicts > 0 { parts.push(format!("✗{}", s.conflicts)); }
        Some(parts.join(" "))
    }
}

impl Widget for GitChanges {
    fn id(&self) -> &'static str { "GitChanges" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.total())
    }
}

impl Widget for GitStaged {
    fn id(&self) -> &'static str { "GitStaged" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.staged)
    }
}

impl Widget for GitUnstaged {
    fn id(&self) -> &'static str { "GitUnstaged" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.unstaged)
    }
}

impl Widget for GitUntracked {
    fn id(&self) -> &'static str { "GitUntracked" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.untracked)
    }
}

impl Widget for GitConflicts {
    fn id(&self) -> &'static str { "GitConflicts" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        nonzero(ctx.git()?.status_counts()?.conflicts)
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
    fn clean_repo_returns_none_for_all_status_widgets() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitStatus.render(&ctx), None);
        assert_eq!(GitChanges.render(&ctx), None);
        assert_eq!(GitStaged.render(&ctx), None);
        assert_eq!(GitUnstaged.render(&ctx), None);
        assert_eq!(GitUntracked.render(&ctx), None);
        assert_eq!(GitConflicts.render(&ctx), None);
    }

    #[test]
    fn untracked_file_lights_up_untracked_and_changes_only() {
        let f = GitFixture::new();
        f.write_file("new.txt", "x");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitUntracked.render(&ctx).as_deref(), Some("1"));
        assert_eq!(GitChanges.render(&ctx).as_deref(), Some("1"));
        assert_eq!(GitStaged.render(&ctx), None);
        assert_eq!(GitUnstaged.render(&ctx), None);
        assert_eq!(GitConflicts.render(&ctx), None);
        assert_eq!(GitStatus.render(&ctx).as_deref(), Some("?1"));
    }

    #[test]
    fn staged_and_unstaged_combine_in_summary() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.write_file("a.txt", "2");
        f.git(&["add", "a.txt"]);
        f.write_file("a.txt", "3");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let summary = GitStatus.render(&ctx).expect("must render");
        // M1 ~1 (1 staged + 1 unstaged для одного файла).
        assert!(summary.contains("M1"));
        assert!(summary.contains("~1"));
    }

    #[test]
    fn outside_repo_all_status_widgets_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitStatus.render(&ctx), None);
        assert_eq!(GitChanges.render(&ctx), None);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/widgets/git_status.rs`.

- [ ] **Step 5: Подключить и зарегистрировать**

Edit `src/widgets/mod.rs`:

Edit 1:
- `old_string`: `pub mod git_head;`
- `new_string`: `pub mod git_head;\npub mod git_status;`

Edit 2 (добавить 6 arm'ов после head cluster):
- `old_string`:
  ```rust
          // Phase 5 — Task 2 (head cluster):
          WidgetConfig::GitBranch => Box::new(git_head::GitBranch),
          WidgetConfig::GitSha => Box::new(git_head::GitSha),
          WidgetConfig::GitRootDir => Box::new(git_head::GitRootDir),
      }
  ```
- `new_string`:
  ```rust
          // Phase 5 — Task 2 (head cluster):
          WidgetConfig::GitBranch => Box::new(git_head::GitBranch),
          WidgetConfig::GitSha => Box::new(git_head::GitSha),
          WidgetConfig::GitRootDir => Box::new(git_head::GitRootDir),
          // Phase 5 — Task 3 (status cluster):
          WidgetConfig::GitStatus => Box::new(git_status::GitStatus),
          WidgetConfig::GitChanges => Box::new(git_status::GitChanges),
          WidgetConfig::GitStaged => Box::new(git_status::GitStaged),
          WidgetConfig::GitUnstaged => Box::new(git_status::GitUnstaged),
          WidgetConfig::GitUntracked => Box::new(git_status::GitUntracked),
          WidgetConfig::GitConflicts => Box::new(git_status::GitConflicts),
      }
  ```

- [ ] **Step 6: Запустить тесты**

```bash
cargo test --locked --lib widgets::git_status
cargo test --locked --lib git::tests
```

Expected: все зелёные. ≥4 status тестов в `git::tests` + 4 в `widgets::git_status`.

- [ ] **Step 7: Проверить, что status считается ровно 1 раз**

Не автоматизированный тест, но видно по `OnceCell`-контракту: тест `status_counts_cached_after_first_call` (Step 3) лочит идемпотентность.

Дополнительно — добавить debug-инстументацию (только в dev) если нужно: `#[cfg(debug_assertions)]` счётчик в `compute_status`. Опционально, можно пропустить.

- [ ] **Step 8: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 9: Verification**

```bash
grep -c 'pub struct GitStatus\b' src/widgets/git_status.rs
grep -c 'pub struct GitChanges' src/widgets/git_status.rs
grep -c 'pub struct GitStaged' src/widgets/git_status.rs
grep -c 'pub struct GitUnstaged' src/widgets/git_status.rs
grep -c 'pub struct GitUntracked' src/widgets/git_status.rs
grep -c 'pub struct GitConflicts' src/widgets/git_status.rs
grep -c 'pub fn status_counts' src/git/mod.rs
grep -c 'OnceCell' src/git/mod.rs
```

Expected: каждый ≥1.

- [ ] **Step 10: Commit**

```bash
git add src/git/mod.rs src/widgets/git_status.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T3 status cluster — 6 widgets sharing one gix status

GitInfo::status_counts() is OnceCell-cached: 6 widgets read the same
GitStatusCounts struct, gix::status() runs at most once per render.

Cluster: GitStatus (summary 'M1 ~2 ?3 ✗4'), GitChanges (sum), and
4 individual counters. Zero counters render as None (filtered out).

Conflicts counted via index stage entries (divided by 3 — each
conflict file has 3 stage records).

Task 3/9 of Phase 5. Cumulative: 9/20.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/widgets/git_status.rs` создан с 6 widget структурами
- [ ] `GitInfo::status_counts() -> Option<&GitStatusCounts>` реализован через `OnceCell`
- [ ] Status считается **ровно один раз** на render (тест `status_counts_cached_after_first_call`)
- [ ] 6 enum-вариантов в `WidgetConfig`, парсятся как `git-status`/`git-changes`/...
- [ ] Чистый репо → все 6 виджетов = None
- [ ] Untracked file → `GitUntracked`, `GitChanges`, `GitStatus` рендерят; остальные = None
- [ ] Conflicts корректно делятся на 3 (каждый конфликтный файл в индексе 3 раза)
- [ ] ≥9 unit-тестов суммарно (4 в `git::tests`, 4+ в `widgets::git_status::tests`)
- [ ] Один commit `feat(phase-5): T3 status cluster ...`

## Files touched

- `src/git/mod.rs` (modified — `compute_status`, `status_counts()`)
- `src/widgets/git_status.rs` (created)
- `src/widgets/mod.rs` (modified — pub mod + 6 arms)
- `src/types/config.rs` (modified — 6 enum variants)

## Risks & rollback

- **gix status API mismatch** — самый высокий риск задачи. Если `gix::status::Platform::into_index_worktree_iter` имеет другую сигнатуру в 0.81 — потратить ≤30 минут на адаптацию. Если API окажется болезненным — fallback на `git status --porcelain=v1` через `Command` (записать решение в DECISIONS).
- **Conflicts /=3 бьёт по точности при rename'ах**: rename в индексе создаёт 4 записи. Известный edge case — 1 widget на это не реагирует (порядок ошибки 1 файл из 1000), документируется в README.
- **Performance на монорепо**: status может быть 10-15 ms — превышает Phase 5 budget 8 ms. Документируем в README phase 9 как "тяжёлая опция, отключайте status-виджеты на монорепо".
- **`OnceCell` race в multi-thread**: cchud single-threaded; если Phase 8 TUI live-update — менять на `OnceLock`.
- **Rollback**: `git revert HEAD` — снимает status кластер; T2 head остаётся.
