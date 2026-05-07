# Task 5 — Tracking (GitAheadBehind)

**Files:**
- Modify: `src/git/mod.rs` (impl `GitInfo::tracking() -> Option<&Tracking>` через `OnceCell`)
- Create: `src/widgets/git_tracking.rs` (1 widget impl)
- Modify: `src/widgets/mod.rs` (`pub mod git_tracking;` + 1 match-arm)
- Modify: `src/types/config.rs` (1 enum-вариант: `GitAheadBehind`)

## Goal

Один виджет `GitAheadBehind` показывает разницу между локальной веткой и upstream (origin/<branch>). Формат: `↑3↓1` (ahead/behind), пустые поля скрыты, обе нули → None.

| Widget | Render | Edge |
|---|---|---|
| `GitAheadBehind` | `↑{ahead}↓{behind}` | None если нет upstream, оба = 0, или detached HEAD |

## Inputs

- T4 закрыт.
- `cargo test --locked widgets::git_diff` зелёный.

---

- [ ] **Step 1: Расширить `WidgetConfig`**

Edit `src/types/config.rs`:
- `old_string`:
  ```rust
      // Phase 5 — Task 4 (diff stat):
      GitInsertions,
      GitDeletions,
  }
  ```
- `new_string`:
  ```rust
      // Phase 5 — Task 4 (diff stat):
      GitInsertions,
      GitDeletions,
      // Phase 5 — Task 5 (tracking):
      GitAheadBehind,
  }
  ```

Тест:
```rust
#[test]
fn parses_phase5_tracking_widget() {
    let json = r#"[{ "type": "git-ahead-behind" }]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0], WidgetConfig::GitAheadBehind));
}
```

- [ ] **Step 2: Реализовать `GitInfo::tracking()`**

Edit `src/git/mod.rs`:

```rust
impl GitInfo {
    pub fn tracking(&self) -> Option<&Tracking> {
        self.tracking
            .get_or_init(|| compute_tracking(&self.repo))
            .as_ref()
    }
}

fn compute_tracking(repo: &gix::Repository) -> Option<Tracking> {
    let head_ref = repo.head_ref().ok()??;
    let local_id = head_ref.target().try_id()?;
    // gix 0.81 даёт upstream через `Reference::remote_tracking_branch()`.
    // Если этот метод отличается — поправить локально.
    let upstream = head_ref.remote_tracking_branch().ok().flatten()?;
    let upstream_id = upstream.target().try_id()?;
    if local_id == upstream_id {
        return Some(Tracking { ahead: 0, behind: 0 });
    }
    // Подсчёт через graph::commit_count_diff (или ручной merge-base + revwalk).
    let (ahead, behind) = ahead_behind_count(repo, local_id, upstream_id)?;
    Some(Tracking { ahead, behind })
}

fn ahead_behind_count(
    repo: &gix::Repository,
    local: gix::ObjectId,
    upstream: gix::ObjectId,
) -> Option<(u32, u32)> {
    // Псевдокод. gix 0.81 — `repo.merge_base(local, upstream)` и revwalk.
    // Если оказывается, что API сложен — fallback на `git rev-list --left-right --count
    // upstream...HEAD`.
    let _ = (repo, local, upstream);
    None
}

/// Shell fallback — простой и надёжный.
fn ahead_behind_shell(cwd: &std::path::Path) -> Option<(u32, u32)> {
    use std::process::Command;
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
        .output()
        .ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 { return None; }
    let behind: u32 = parts[0].parse().ok()?;
    let ahead: u32 = parts[1].parse().ok()?;
    Some((ahead, behind))
}
```

> **Recommendation**: использовать **только** shell fallback в Phase 5 — gix tracking API нестабильно, а `git rev-list --left-right` стандартный и надёжный. Записать в DECISIONS если выбираем shell.

Упрощённая версия `compute_tracking`:

```rust
fn compute_tracking(repo: &gix::Repository) -> Option<Tracking> {
    let cwd = repo.work_dir()?;
    let (ahead, behind) = ahead_behind_shell(cwd)?;
    Some(Tracking { ahead, behind })
}
```

- [ ] **Step 3: Тесты для tracking**

В `src/git/mod.rs` `mod tests`:

```rust
#[test]
fn tracking_none_without_upstream() {
    let f = GitFixture::new();
    let info = GitInfo::discover(f.path()).unwrap();
    // Свежий репо — нет upstream.
    assert!(info.tracking().is_none());
}

#[test]
fn tracking_zero_when_in_sync() {
    let f = GitFixture::new();
    // Создать "удалённый" локально через bare-репо.
    let bare = tempfile::tempdir().unwrap();
    f.git(&["init", "--bare", bare.path().to_str().unwrap()]);
    f.add_remote("origin", bare.path().to_str().unwrap());
    f.git(&["push", "-u", "origin", "main"]);

    let info = GitInfo::discover(f.path()).unwrap();
    let t = info.tracking().expect("must have tracking");
    assert_eq!(t.ahead, 0);
    assert_eq!(t.behind, 0);
}

#[test]
fn tracking_ahead_after_local_commit() {
    let f = GitFixture::new();
    let bare = tempfile::tempdir().unwrap();
    let bare_path = bare.path().to_str().unwrap();
    f.git(&["init", "--bare", bare_path]);
    f.add_remote("origin", bare_path);
    f.git(&["push", "-u", "origin", "main"]);

    f.write_file("a.txt", "x");
    f.git(&["add", "a.txt"]);
    f.commit("c2");

    let info = GitInfo::discover(f.path()).unwrap();
    let t = info.tracking().unwrap();
    assert_eq!(t.ahead, 1);
    assert_eq!(t.behind, 0);
}
```

- [ ] **Step 4: Создать `src/widgets/git_tracking.rs`**

```rust
//! Git ahead/behind tracking — Phase 5 Task 5.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitAheadBehind;

impl Widget for GitAheadBehind {
    fn id(&self) -> &'static str { "GitAheadBehind" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.git()?.tracking()?;
        if t.ahead == 0 && t.behind == 0 {
            return None;
        }
        let mut s = String::with_capacity(8);
        if t.ahead > 0 {
            s.push('↑');
            s.push_str(&t.ahead.to_string());
        }
        if t.behind > 0 {
            s.push('↓');
            s.push_str(&t.behind.to_string());
        }
        Some(s)
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
    fn no_upstream_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx), None);
    }

    #[test]
    fn ahead_only_renders_arrow_up() {
        let f = GitFixture::new();
        let bare = tempfile::tempdir().unwrap();
        let bare_path = bare.path().to_str().unwrap();
        f.git(&["init", "--bare", bare_path]);
        f.add_remote("origin", bare_path);
        f.git(&["push", "-u", "origin", "main"]);
        f.write_file("a.txt", "x");
        f.git(&["add", "a.txt"]);
        f.commit("c2");

        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx).as_deref(), Some("↑1"));
    }

    #[test]
    fn outside_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitAheadBehind.render(&ctx), None);
    }
}
```

- [ ] **Step 5: Регистрация**

Edit `src/widgets/mod.rs`:

Edit 1: `pub mod git_tracking;` после `pub mod git_diff;`.

Edit 2:
- `old_string`:
  ```rust
          WidgetConfig::GitDeletions => Box::new(git_diff::GitDeletions),
      }
  ```
- `new_string`:
  ```rust
          WidgetConfig::GitDeletions => Box::new(git_diff::GitDeletions),
          // Phase 5 — Task 5 (tracking):
          WidgetConfig::GitAheadBehind => Box::new(git_tracking::GitAheadBehind),
      }
  ```

- [ ] **Step 6: Тесты + standard gate**

```bash
cargo test --locked --lib widgets::git_tracking
cargo test --locked --lib git::tests::tracking
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 7: Commit**

```bash
git add src/git/mod.rs src/widgets/git_tracking.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T5 tracking — GitAheadBehind

Reads ahead/behind via 'git rev-list --left-right --count
@{upstream}...HEAD' (shell fallback, ~3-5 ms via OnceCell).

Render: '↑3↓1' / '↑3' / '↓1' / None when in sync, no upstream,
or outside repo.

Task 5/9 of Phase 5. Cumulative: 12/20.
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

- [ ] `GitInfo::tracking() -> Option<&Tracking>` через `OnceCell`
- [ ] `src/widgets/git_tracking.rs` с `GitAheadBehind`
- [ ] Enum `GitAheadBehind` парсится как `git-ahead-behind`
- [ ] Без upstream → None
- [ ] In-sync → None (оба нули)
- [ ] Ahead 1 → `↑1`
- [ ] ≥3 теста в `git::tests::tracking*` + 3 в `widgets::git_tracking::tests`
- [ ] Один commit `feat(phase-5): T5 tracking ...`

## Files touched

- `src/git/mod.rs` (modified)
- `src/widgets/git_tracking.rs` (created)
- `src/widgets/mod.rs` (modified)
- `src/types/config.rs` (modified)

## Risks & rollback

- **`git rev-list` без upstream падает**: `status.success()` ложь → None. Тест `no_upstream_returns_none` лочит контракт.
- **`@{upstream}` синтаксис в Windows shells**: команда выполняется через `Command::new("git")`, не через shell — `@{upstream}` парсится самим git'ом, кроссплатформенно.
- **Stale upstream после `git fetch`**: tracking считается против локального ref'а `refs/remotes/origin/main`, который обновляется только при fetch. Это поведение upstream ccstatusline; документировать в README.
- **Rollback**: `git revert HEAD`.
