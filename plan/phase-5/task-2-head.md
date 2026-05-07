# Task 2 — Head cluster (GitBranch, GitSha, GitRootDir)

**Files:**
- Create: `src/widgets/git_head.rs` (3 widget impl)
- Modify: `src/widgets/mod.rs` (`pub mod git_head;` + 3 match-arms)
- Modify: `src/types/config.rs` (3 enum-варианта в `WidgetConfig`: `GitBranch`, `GitSha`, `GitRootDir`)

## Goal

Первый кластер git-виджетов — самый простой: данные уже распарсены в T1 (`GitInfo::head` и `GitInfo::root_dir`), виджеты — тонкие getter'ы.

| Widget | Render | Edge |
|---|---|---|
| `GitBranch` | `git.head.branch.clone()` | None если detached HEAD или вне репо |
| `GitSha` | `git.head.short_sha()` | None если HEAD не указывает на коммит (unborn) |
| `GitRootDir` | basename(`git.root_dir`) | None если рут — `/` (ломаный сценарий, но безопасно) |

Сознательно НЕ:
- Полный SHA — Phase 5 виджеты используют только short (7 chars). Полный SHA можно будет добавить позже опцией.
- Custom format strings — Phase 7.

## Inputs

- T1 закрыт: `GitInfo::discover` работает, `RenderContext::git()` доступен.
- `cargo test --locked git::` зелёный.

---

- [ ] **Step 1: Расширить `WidgetConfig` 3 вариантами**

Read `src/types/config.rs`. Найти `WidgetConfig` enum (после Phase 3 он содержит ~24 варианта).

Edit `src/types/config.rs`:
- `old_string` (выбрать последний вариант перед закрывающей `}`):
  ```rust
      CustomCommand { params: CustomCommandParams },
  }
  ```
- `new_string`:
  ```rust
      CustomCommand { params: CustomCommandParams },
      // Phase 5 — Task 2 (head cluster):
      GitBranch,
      GitSha,
      GitRootDir,
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

> Если `WidgetConfig` использует `#[serde(rename_all = "kebab-case")]` (Phase 3 retrofit) — `GitBranch` → `"git-branch"` автоматически.

- [ ] **Step 2: Добавить тест на парсинг новых вариантов**

В `src/types/config.rs` тестах (или там где `parses_phase3_widget_kinds`) добавить:

```rust
#[test]
fn parses_phase5_head_widgets() {
    let json = r#"[
        { "type": "git-branch" },
        { "type": "git-sha" },
        { "type": "git-root-dir" }
    ]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0], WidgetConfig::GitBranch));
    assert!(matches!(widgets[1], WidgetConfig::GitSha));
    assert!(matches!(widgets[2], WidgetConfig::GitRootDir));
}
```

Запустить:

```bash
cargo test --locked types::config::tests::parses_phase5_head_widgets
```

Expected: PASS.

- [ ] **Step 3: Создать `src/widgets/git_head.rs`**

Create `/Users/igor/mp/startup/cchud/src/widgets/git_head.rs`:

```rust
//! Git head cluster — Phase 5 Task 2.
//!
//! Тонкие getter'ы над `GitInfo::head` и `GitInfo::root_dir` (T1).
//! Виджеты возвращают None если cwd вне git-репо или HEAD detached/unborn.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct GitBranch;
pub struct GitSha;
pub struct GitRootDir;

impl Widget for GitBranch {
    fn id(&self) -> &'static str { "GitBranch" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        ctx.git()?.head.branch.clone()
    }
}

impl Widget for GitSha {
    fn id(&self) -> &'static str { "GitSha" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        ctx.git()?.head.short_sha()
    }
}

impl Widget for GitRootDir {
    fn id(&self) -> &'static str { "GitRootDir" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let root = &ctx.git()?.root_dir;
        root.file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::git::fixture::GitFixture;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_cwd(cwd: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: cwd.into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    #[test]
    fn git_branch_returns_main_in_fresh_repo() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx).as_deref(), Some("main"));
    }

    #[test]
    fn git_branch_returns_none_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload_with_cwd(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx), None);
    }

    #[test]
    fn git_branch_returns_none_for_detached_head() {
        let f = GitFixture::new();
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.git(&["checkout", "--detach", "HEAD"]);
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitBranch.render(&ctx), None);
    }

    #[test]
    fn git_sha_returns_seven_chars() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let sha = GitSha.render(&ctx).expect("must have sha");
        assert_eq!(sha.len(), 7);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn git_sha_returns_none_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        let p = payload_with_cwd(dir.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitSha.render(&ctx), None);
    }

    #[test]
    fn git_root_dir_returns_repo_basename() {
        let f = GitFixture::new();
        let p = payload_with_cwd(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        let root = GitRootDir.render(&ctx).expect("must have root");
        // tempfile создаёт директории вроде `.tmpAbcDef` — ненулевые.
        assert!(!root.is_empty());
        assert_eq!(root, f.path().file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn git_root_dir_works_from_subdirectory() {
        let f = GitFixture::new();
        f.write_file("nested/deep/file.txt", "x");
        let nested = f.path().join("nested/deep");
        let p = payload_with_cwd(nested.to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        // root_dir остаётся basename'ом репо, не CWD.
        assert_eq!(
            GitRootDir.render(&ctx).as_deref(),
            Some(f.path().file_name().unwrap().to_str().unwrap())
        );
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/widgets/git_head.rs`.

- [ ] **Step 4: Подключить модуль и заменить match-arms**

Edit `src/widgets/mod.rs`:

Edit 1:
- `old_string`: `pub mod custom_command;`
- `new_string`: `pub mod custom_command;\npub mod git_head;`
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 2 (добавить 3 arm'а в `build_one`, перед закрывающей `}` функции):
- `old_string`:
  ```rust
          // Phase 3 — Task 7 (custom-command):
          WidgetConfig::CustomCommand { params } => Box::new(custom_command::CustomCommand {
              params: params.clone(),
          }),
      }
  }
  ```
- `new_string`:
  ```rust
          // Phase 3 — Task 7 (custom-command):
          WidgetConfig::CustomCommand { params } => Box::new(custom_command::CustomCommand {
              params: params.clone(),
          }),

          // Phase 5 — Task 2 (head cluster):
          WidgetConfig::GitBranch => Box::new(git_head::GitBranch),
          WidgetConfig::GitSha => Box::new(git_head::GitSha),
          WidgetConfig::GitRootDir => Box::new(git_head::GitRootDir),
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 5: Запустить тесты**

```bash
cargo test --locked --lib widgets::git_head
```

Expected: 7 тестов passed.

- [ ] **Step 6: End-to-end smoke**

```bash
cd /tmp && rm -rf cchud-smoke && mkdir cchud-smoke && cd cchud-smoke && git init -b main -q && git -c user.email=a@b -c user.name=a commit --allow-empty -q -m init
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}' \
  | /Users/igor/mp/startup/cchud/target/release/cchud
```

Expected: вывод включает `M` (default-line с одним только Model). Чтобы проверить git-виджет — добавить настройку через файл с `git-branch`. Optional manual test, можно пропустить.

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 8: Verification — task-specific gate**

```bash
grep -c 'pub struct GitBranch' src/widgets/git_head.rs
grep -c 'pub struct GitSha' src/widgets/git_head.rs
grep -c 'pub struct GitRootDir' src/widgets/git_head.rs
grep -c 'git_head::GitBranch' src/widgets/mod.rs
grep -c 'git_head::GitSha' src/widgets/mod.rs
grep -c 'git_head::GitRootDir' src/widgets/mod.rs
```

Expected:
```
1
1
1
1
1
1
```

- [ ] **Step 9: Commit**

```bash
git add src/widgets/git_head.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T2 head cluster — GitBranch, GitSha, GitRootDir

Three thin getters over GitInfo::{head, root_dir} (built in T1).

GitBranch: head.branch (None if detached/unborn).
GitSha: head.short_sha() — first 7 hex chars.
GitRootDir: basename of work_dir (works from any subdir of repo).

7 unit tests with GitFixture: fresh repo / detached HEAD / outside
repo / nested cwd. All exit gracefully via Option chain.

Task 2/9 of Phase 5. 3 widgets added; cumulative Phase 5 = 3/20.
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

- [ ] `src/widgets/git_head.rs` создан с `GitBranch`, `GitSha`, `GitRootDir`
- [ ] Каждый виджет реализует `Widget` trait с уникальным `id()`
- [ ] 3 enum-варианта добавлены в `WidgetConfig`; парсятся как `git-branch`/`git-sha`/`git-root-dir`
- [ ] 7 unit-тестов в `widgets::git_head::tests` зелёные
- [ ] `widgets::mod` инстанциирует 3 widget'а в `build_one`
- [ ] Detached HEAD → `GitBranch::render` = None
- [ ] Cwd вне репо → все 3 виджета = None
- [ ] Один commit `feat(phase-5): T2 head cluster ...`

## Files touched

- `src/widgets/git_head.rs` (created)
- `src/widgets/mod.rs` (modified)
- `src/types/config.rs` (modified — 3 new enum variants + 1 new test)

## Risks & rollback

- **gix `referent_name()` рейзит на unborn HEAD**: T1 контракт говорит "None если HEAD не разрешается". Если gix паникует на пустом репо без коммитов — обернуть `head()` в catch_unwind или проверить `repo.is_empty()` сначала. Тесты T1 уже создают initial commit, обходим.
- **`root_dir.file_name()` для bare repo**: bare-репо имеет `path()` без `work_dir`. T1 `discover` возвращает `repo.path()` в таком случае — `.file_name()` даст `.git` или последний компонент. Документировать как edge case в T9 README.
- **Performance**: `ctx.git()` lazy — первый виджет триггерит discover, остальные читают мгновенно. Нет N+1.
- **Rollback**: `git revert HEAD` — снимает 3 виджета и enum-варианты; Phase 5 T1 остаётся.
