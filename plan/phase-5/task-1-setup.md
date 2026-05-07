# Task 1 — Setup: gix dep, GitInfo skeleton, RenderContext::git()

**Files:**
- Modify: `Cargo.toml` (добавить `gix = "=0.81.0"` runtime; `tempfile`/`assert_cmd` уже в dev-deps)
- Create: `src/git/mod.rs` (`GitInfo`, `Head`, `GitStatusCounts`, `RemoteInfo`, `discover()`)
- Create: `src/git/fixture.rs` (`#[cfg(test)]` helper — tempfile-репо через `git` CLI)
- Modify: `src/widgets/mod.rs` (добавить `pub git: OnceCell<Option<GitInfo>>` в `RenderContext`; метод `RenderContext::git()`)
- Modify: `src/lib.rs` (`pub mod git;`)
- Create: `docs/DECISIONS.md` запись (если не существует — добавить раздел; иначе append)

## Goal

Архитектурный скелет git-инфраструктуры — все Phase 5 виджеты будут *тонкими getter'ами* над одним lazy `GitInfo`. Никаких виджетов в этой задаче, но без неё T2–T7 невозможны.

Ключевые инварианты, которые лочит T1:

1. **Lazy один раз**: `RenderContext::git()` вызывает `discover()` максимум один раз за render. Если CWD не git-репо → `None`, виджеты возвращают `None`.
2. **Внутри `GitInfo` тоже lazy**: `head` и `remotes` парсятся при `discover`; `status_counts` и `diff_stat` — `OnceCell`, считаются только при первом запросе соответствующим виджетом.
3. **Стоимость отсутствия git**: payload без git-виджетов в строке → `discover()` вообще не вызывается. Это лочит производительность Phase 3 (нулевая регрессия).
4. **gix vs git2**: pure Rust (`gix`), без libgit2/cmake-system-deps. Pin на `=0.81.0` точная версия — minor breaking changes требуют отдельного PR.
5. **Фикстура**: тесты создают tempfile-репо через `git init` CLI (cross-platform, не требует gix внутри тестов; гейт T8 проверит производительность через `gix`).

Сознательно НЕ включаем (это T2+):
- Виджет-структуры (T2 head, T3 status, ...).
- Парсинг remote URL (T6, hand-parser).
- HTTP-клиент / GitPr (T7, отдельная dep `ureq`).
- `WorktreeInfo` через gix — Phase 3 уже покрыл worktree через payload.

## Inputs

- Phase 4 завершена, ветка main в стабильном состоянии.
- `cargo build --release --locked` зелёный.
- `git --version` возвращает что-нибудь (T1 тесты используют CLI).
- В `dev-dependencies` уже есть `tempfile = "3"`.

---

- [ ] **Step 1: Добавить `gix` в `Cargo.toml`**

Edit `Cargo.toml`:

- `old_string`:
  ```toml
  # Phase 3 — CustomCommand subprocess timeout
  wait-timeout = "0.2"

  # Lazy для виджетов (добавятся в фазах 3-7)
  # sonic-rs, gix, ureq, bincode, ratatui, crossterm — позже
  ```
- `new_string`:
  ```toml
  # Phase 3 — CustomCommand subprocess timeout
  wait-timeout = "0.2"

  # Phase 5 — git via gix (pure Rust, no libgit2 system dep).
  # Pinned to exact version: gix has frequent minor breaking changes;
  # bumping is a deliberate PR with smoke-тест.
  gix = { version = "=0.81.0", default-features = false, features = ["max-performance-safe"] }

  # Lazy для виджетов (добавятся в фазах 6-7)
  # sonic-rs, ureq, bincode, ratatui, crossterm — позже
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

> **Why `default-features = false`**: gix по умолчанию подтягивает `gitoxide` worktree/transport фичи, которые нам не нужны (Phase 5 только read repo state). `max-performance-safe` оставляет zlib/sha1/regex без C-bindings.

- [ ] **Step 2: Запустить `cargo build` — должен зелёным с новой dep'ой**

```bash
cargo build --release --locked 2>&1 | tail -5
```

Expected: успешная сборка (gix компилируется ~30-60 sec первый раз). Никаких deprecation warnings.

```bash
ls -lh target/release/cchud
```

Expected: размер < 6 MB (Phase 3 baseline ~5 MB + gix ~0.8-1.5 MB).

- [ ] **Step 3: Создать `src/git/fixture.rs` (test helper)**

Create `/Users/igor/mp/startup/cchud/src/git/fixture.rs`:

```rust
//! Test fixture helpers — Phase 5 Task 1.
//!
//! Создаёт tempfile-репозитории через `git` CLI (cross-platform, доступен
//! на macos/linux/windows CI runners). Использовать gix для setup'а
//! фикстур избыточно: тесты должны быть быстрыми и читаемыми, а gix init
//! API меняется между minor.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Lightweight git-репо в tempdir.
pub struct GitFixture {
    pub dir: TempDir,
}

impl GitFixture {
    /// Создать пустой git-репо с одним commit'ом и веткой `main`.
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        run(&dir.path(), &["init", "-b", "main"]);
        run(&dir.path(), &["config", "user.email", "test@cchud.dev"]);
        run(&dir.path(), &["config", "user.name", "cchud-test"]);
        // Один пустой commit, чтобы HEAD ссылался на коммит, а не unborn ref.
        run(&dir.path(), &["commit", "--allow-empty", "-m", "init"]);
        Self { dir }
    }

    /// Путь к рабочему дереву.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Написать файл и `git add`.
    pub fn write_file(&self, rel: &str, content: &str) {
        let abs = self.dir.path().join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&abs, content).unwrap();
    }

    /// Запустить произвольную git-команду в репо.
    pub fn git(&self, args: &[&str]) {
        run(&self.dir.path(), args);
    }

    /// Создать commit (требует предварительный `add`).
    pub fn commit(&self, msg: &str) {
        run(&self.dir.path(), &["commit", "-m", msg]);
    }

    /// Добавить remote.
    pub fn add_remote(&self, name: &str, url: &str) {
        run(&self.dir.path(), &["remote", "add", name, url]);
    }
}

fn run(cwd: &Path, args: &[&str]) {
    let out = Command::new("git").current_dir(cwd).args(args).output().unwrap();
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
        assert!(stdout.contains("init"), "expected initial commit, got {stdout}");
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
```

Path: `/Users/igor/mp/startup/cchud/src/git/fixture.rs`.

- [ ] **Step 4: Создать `src/git/mod.rs` с `GitInfo` skeleton**

Create `/Users/igor/mp/startup/cchud/src/git/mod.rs`:

```rust
//! Git domain — lazy `GitInfo` через gix; виджеты Phase 5 — тонкие
//! getter'ы над этой структурой.
//!
//! Контракт:
//! - `discover(cwd)` пытается найти репо начиная с `cwd` вверх. None если
//!   за границу маунта или до `/`.
//! - `head` и `remotes` парсятся eagerly при `discover()` (стоимость ~50 µs).
//! - `status_counts` и `diff_stat` — `OnceCell`, считаются только при
//!   запросе соответствующим виджетом (T3, T4).
//! - `RenderContext::git()` обеспечивает однократный `discover()` за render.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub mod fixture;

/// Главная git-структура, переиспользуемая всеми Phase 5 виджетами.
pub struct GitInfo {
    /// `gix::Repository` — переиспользуется внутренними методами для status/diff.
    pub(crate) repo: gix::Repository,
    pub root_dir: PathBuf,
    pub head: Head,
    pub remotes: HashMap<String, RemoteInfo>,
    /// T3: lazy. None если ещё не считали.
    pub(crate) status_counts: OnceCell<Option<GitStatusCounts>>,
    /// T4: lazy. None если ещё не считали.
    pub(crate) diff_stat: OnceCell<Option<DiffStat>>,
    /// T5: lazy ahead/behind.
    pub(crate) tracking: OnceCell<Option<Tracking>>,
}

#[derive(Debug, Clone, Default)]
pub struct Head {
    /// Имя ветки или None если detached HEAD.
    pub branch: Option<String>,
    /// Полный SHA-1 (40 hex).
    pub sha: Option<String>,
}

impl Head {
    /// Короткий SHA — первые 7 символов (upstream ccstatusline дефолт).
    #[must_use]
    pub fn short_sha(&self) -> Option<String> {
        self.sha.as_ref().map(|s| s.chars().take(7).collect())
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitStatusCounts {
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicts: u32,
}

impl GitStatusCounts {
    /// Сумма для виджета `GitChanges`: staged + unstaged + untracked + conflicts.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.staged + self.unstaged + self.untracked + self.conflicts
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiffStat {
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Tracking {
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub url: String,
    /// Заполняется в T6 (`parse_url`). В T1 — None.
    pub owner: Option<String>,
    pub repo: Option<String>,
}

impl GitInfo {
    /// Поиск репо начиная с `cwd`, вверх до `/` или mount-границы.
    /// Возвращает None если репо не найдено или gix не смог открыть.
    #[must_use]
    pub fn discover(cwd: &Path) -> Option<Self> {
        let repo = gix::discover(cwd).ok()?;
        let root_dir = repo
            .work_dir()
            .or_else(|| Some(repo.path()))
            .map(Path::to_path_buf)?;

        let head = parse_head(&repo);
        let remotes = parse_remotes(&repo);

        Some(Self {
            repo,
            root_dir,
            head,
            remotes,
            status_counts: OnceCell::new(),
            diff_stat: OnceCell::new(),
            tracking: OnceCell::new(),
        })
    }
}

fn parse_head(repo: &gix::Repository) -> Head {
    let Ok(head) = repo.head() else {
        return Head::default();
    };
    let branch = head.referent_name().map(|n| {
        let full = n.as_bstr().to_string();
        full.strip_prefix("refs/heads/").map_or(full.clone(), String::from)
    });
    let sha = head.id().map(|id| id.to_string());
    Head { branch, sha }
}

fn parse_remotes(repo: &gix::Repository) -> HashMap<String, RemoteInfo> {
    let mut out = HashMap::new();
    let Ok(names) = repo.remote_names() else {
        return out;
    };
    for name in names {
        let Ok(remote) = repo.find_remote(name.as_ref()) else {
            continue;
        };
        let Some(url) = remote.url(gix::remote::Direction::Fetch) else {
            continue;
        };
        let url_str = url.to_bstring().to_string();
        out.insert(
            name.to_string(),
            RemoteInfo { url: url_str, owner: None, repo: None },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::git::fixture::GitFixture;

    #[test]
    fn discover_returns_some_for_git_repo() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).expect("must discover");
        assert_eq!(info.root_dir.canonicalize().unwrap(), f.path().canonicalize().unwrap());
    }

    #[test]
    fn discover_returns_none_for_non_repo() {
        let dir = tempfile::tempdir().unwrap();
        assert!(GitInfo::discover(dir.path()).is_none());
    }

    #[test]
    fn head_branch_is_main_after_init() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.head.branch.as_deref(), Some("main"));
        assert!(info.head.sha.is_some());
        assert_eq!(info.head.sha.as_ref().unwrap().len(), 40);
    }

    #[test]
    fn short_sha_is_seven_chars() {
        let f = GitFixture::new();
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.head.short_sha().unwrap().len(), 7);
    }

    #[test]
    fn detached_head_branch_is_none() {
        let f = GitFixture::new();
        // Создать второй коммит, чтобы было куда отсоединиться.
        f.write_file("a.txt", "1");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        // Detach.
        f.git(&["checkout", "--detach", "HEAD"]);
        let info = GitInfo::discover(f.path()).unwrap();
        assert!(info.head.branch.is_none(), "detached HEAD must yield None branch");
        assert!(info.head.sha.is_some());
    }

    #[test]
    fn remotes_parsed_from_config() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        f.add_remote("upstream", "https://github.com/baz/bar.git");
        let info = GitInfo::discover(f.path()).unwrap();
        assert_eq!(info.remotes.len(), 2);
        assert_eq!(
            info.remotes.get("origin").unwrap().url,
            "git@github.com:foo/bar.git"
        );
        assert_eq!(
            info.remotes.get("upstream").unwrap().url,
            "https://github.com/baz/bar.git"
        );
        // T6 заполнит owner/repo.
        assert!(info.remotes.get("origin").unwrap().owner.is_none());
    }

    #[test]
    fn status_counts_total_sums_fields() {
        let s = GitStatusCounts { staged: 1, unstaged: 2, untracked: 3, conflicts: 4 };
        assert_eq!(s.total(), 10);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/git/mod.rs`.

> **Note про `gix::head::Head` API**: gix 0.81 предоставляет `Head::referent_name()` для получения ref-имени и `Head::id()` для object id. Если в фактическом 0.81 имена методов другие — поправить точечно при первом `cargo build`. Контракт фиксирован тестами.

- [ ] **Step 5: Подключить `pub mod git;` в `src/lib.rs`**

Read `src/lib.rs`. Если содержимое:

```rust
// (current content)
```

— дописать `pub mod git;` рядом с другими `pub mod`.

Edit `src/lib.rs`:
- `old_string`: текущая последняя строка (например `pub mod widgets;` или похожее)
- `new_string`: добавить ниже `pub mod git;`

Если `lib.rs` пустой / только декларация — добавить:

```rust
pub mod git;
```

`file_path`: `/Users/igor/mp/startup/cchud/src/lib.rs`.

- [ ] **Step 6: Расширить `RenderContext` lazy `git()` методом**

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
  pub struct RenderContext<'a> {
      pub payload: &'a StatusPayload,
      #[allow(dead_code)]
      pub settings: &'a Settings,
  }

  impl<'a> RenderContext<'a> {
      #[must_use]
      pub const fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
          Self { payload, settings }
      }
  }
  ```
- `new_string`:
  ```rust
  pub struct RenderContext<'a> {
      pub payload: &'a StatusPayload,
      #[allow(dead_code)]
      pub settings: &'a Settings,
      /// Phase 5: lazy git discover. None если cwd не git-репо.
      git: std::cell::OnceCell<Option<crate::git::GitInfo>>,
  }

  impl<'a> RenderContext<'a> {
      #[must_use]
      pub fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
          Self {
              payload,
              settings,
              git: std::cell::OnceCell::new(),
          }
      }

      /// Lazy: вызывает `gix::discover(cwd)` максимум один раз. None если
      /// payload без cwd или cwd вне git-репо.
      pub fn git(&self) -> Option<&crate::git::GitInfo> {
          self.git
              .get_or_init(|| {
                  let cwd = self.payload.workspace.current_dir.as_str();
                  crate::git::GitInfo::discover(std::path::Path::new(cwd))
              })
              .as_ref()
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

> **`const fn new` → `fn new`**: const больше нельзя, поскольку `OnceCell::new()` не const на edition 2024 в stable (оно const с 1.70+, ОК — оставить `const` если компилируется). Если clippy ругается — `const` снимем.

- [ ] **Step 7: Обновить тестовые helper'ы виджетов (контекст-фабрики)**

В файлах `src/widgets/context.rs`, `src/widgets/custom_command.rs`, и любых других тестах, использующих `RenderContext::new`, проверить, что `OnceCell::new()` совместим с `const fn` или удалить `const`.

```bash
grep -rn 'RenderContext::new' src/ tests/ 2>/dev/null
```

Запустить:

```bash
cargo build --locked 2>&1 | tail -20
```

Expected: успешная сборка. Если ошибка `cannot call non-const fn OnceCell::new in const context` — снять `const` с `RenderContext::new` (Step 6 уже это делает, но если есть остаточные `const fn` в тестах — поправить).

- [ ] **Step 8: Запустить все тесты — Phase 3 не должна регрессировать**

```bash
cargo test --locked 2>&1 | tail -10
```

Expected: все Phase 2/3 тесты зелёные. Новые тесты в `git::tests` и `git::fixture::tests` тоже зелёные.

```bash
cargo test --locked git:: 2>&1 | tail -15
```

Expected: ≥9 тестов passed (3 в `fixture::tests`, 6+ в `git::tests`).

- [ ] **Step 9: Записать decision в `docs/DECISIONS.md`**

Read `docs/DECISIONS.md`. Найти конец файла. Append:

```markdown

---

## D-2026-04-27 — gix vs git2 для Phase 5

**Контекст:** Phase 5 требует читать git state (HEAD, status, diff, remotes, tracking). Два кандидата:
- `git2` (libgit2 bindings) — зрелый, но требует libgit2 + cmake systemstack, сложности на Windows и в `cargo install`-сценарии.
- `gix` (pure Rust) — без C зависимостей, быстрый, но API менее стабилен (breaking changes между minor).

**Решение:** `gix = "=0.81.0"`, `default-features = false`, `max-performance-safe`.

**Обоснование:**
1. **Дистрибуция через `cargo install` / npm-loader**: pure Rust → бинарь без рантайм-зависимостей. libgit2 в WASM/musl-сборках доставляет проблем.
2. **Cold-start budget < 8 ms p95**: gix `discover()` ~100 µs, status ~1-2 ms на репе среднего размера. git2 сопоставим, но имеет startup overhead на загрузке libgit2.so.
3. **Pin на точную версию**: gix меняет `gix::head::Head` API между minor; обновление gix → отдельный PR со smoke-тестом фикстур из `git::fixture`.
4. **Фичи**: `default-features = false` исключает worktree/transport/protocol — мы только читаем локальное состояние.

**Trade-offs приняты:**
- Зависимость от ручного bump'а gix. Митигация: CI-бенч на каждом обновлении.
- Если gix окажется неподходящим (например, регрессии в diff API) — fallback на `git2` остаётся опцией; интерфейс `GitInfo` спрятан за `pub(crate)` `repo: gix::Repository`.

**Альтернативы рассмотрены:**
- shell-out на `git` CLI: ~2-5 ms на каждый вызов из-за subprocess fork; неприемлемо для 6+ status-виджетов.
- кастомный read-only git parser: переизобретение, не оправдано.

**Owner:** Igor Fonin
```

Если файла нет — создать с этой записью + краткий header.

- [ ] **Step 10: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

Возможные clippy-замечания:
- `pedantic::missing_const_for_fn` на `RenderContext::new` — игнорируем (`OnceCell::new()` не делает функцию некостной, но clippy может наоборот требовать `const` — добавить `#[allow(clippy::missing_const_for_fn)]` если так).
- `pedantic::module_name_repetitions` на `GitInfo`/`GitStatusCounts` — игнорируем; namespace-collision с `WidgetConfig::GitStatus` будет в T3, имена сознательно различны (`GitStatusCounts` для модели, `GitStatus` для виджета).

- [ ] **Step 11: Verification — task-specific gate**

```bash
grep -c 'pub struct GitInfo' src/git/mod.rs
grep -c 'pub fn discover' src/git/mod.rs
grep -c 'pub fn git' src/widgets/mod.rs
grep -c 'OnceCell' src/widgets/mod.rs
grep -c '"=0.81' Cargo.toml
ls -lh target/release/cchud | awk '{print $5}'
```

Expected:
```
1                      (struct GitInfo)
1                      (fn discover)
1                      (fn git on RenderContext)
≥1                     (OnceCell в RenderContext)
1                      (gix pinned)
< 6.5M                 (binary size)
```

- [ ] **Step 12: Phase 3 snapshot regression check**

```bash
cargo test --locked snapshots 2>&1 | tail -10
cargo insta pending-snapshots 2>&1 | head -5
```

Expected: snapshot'ы зелёные, pending пусто.

- [ ] **Step 13: Commit**

```bash
git add Cargo.toml Cargo.lock src/git/ src/widgets/mod.rs src/lib.rs docs/DECISIONS.md
git commit -m "feat(phase-5): T1 setup — gix dep, GitInfo skeleton, RenderContext::git()

Phase 5 foundation. Adds gix=0.81 (pure Rust, pinned exact). New
src/git/mod.rs with GitInfo (lazy via OnceCell): head + remotes
parsed eagerly on discover(); status_counts, diff_stat, tracking
deferred to T3/T4/T5.

RenderContext gets pub fn git() — single discover per render via
OnceCell. None if cwd not in repo; widgets handle gracefully.

Test fixture (src/git/fixture.rs, cfg(test)) creates tempdir repos
via 'git' CLI for cross-platform reproducibility.

DECISIONS: gix-vs-git2 rationale recorded.

Task 1/9 of Phase 5. No widgets yet — T2 adds the head cluster.
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

- [ ] `Cargo.toml` содержит `gix = "=0.81.0"` с `default-features = false`
- [ ] `cargo build --release --locked` зелёный, binary < 6.5 MB
- [ ] `src/git/mod.rs` создан с `GitInfo`, `Head`, `GitStatusCounts`, `DiffStat`, `Tracking`, `RemoteInfo`
- [ ] `GitInfo::discover()` корректно возвращает `Some` в репо, `None` вне
- [ ] `Head::short_sha()` возвращает 7 символов
- [ ] `parse_remotes` собирает remote.url; owner/repo остаются `None` (T6 заполнит)
- [ ] `src/git/fixture.rs` (`#[cfg(test)]`) предоставляет `GitFixture::{new, write_file, git, commit, add_remote}`
- [ ] `RenderContext` получил `git: OnceCell<Option<GitInfo>>` и метод `git() -> Option<&GitInfo>`
- [ ] `git()` вызывает `discover()` максимум один раз за context-lifetime
- [ ] Phase 3 тесты не регрессируют (`cargo test --locked` зелёный)
- [ ] `docs/DECISIONS.md` содержит запись D-2026-04-27 (gix vs git2)
- [ ] Один commit `feat(phase-5): T1 setup ...`

## Files touched

- `Cargo.toml` (modified)
- `Cargo.lock` (auto)
- `src/git/mod.rs` (created)
- `src/git/fixture.rs` (created)
- `src/widgets/mod.rs` (modified — `RenderContext.git()`)
- `src/lib.rs` (modified — `pub mod git;`)
- `docs/DECISIONS.md` (appended)

## Risks & rollback

- **gix 0.81 API name mismatch**: реальные имена методов `Head::referent_name()`/`id()` могут отличаться. Действие: при первом `cargo build` поправить локально, фиксированный контракт в тестах не страдает.
- **Cold-start regression**: если binary > 7 MB или Phase 3 hyperfine p95 > 6 ms (Phase 3 budget = 5ms, добавили 1ms запас) — откат gix-dep, рассмотреть git2 / shell-out.
- **`OnceCell` thread-safety**: cchud rendering single-threaded, `OnceCell` (не `Mutex`/`RwLock`) — корректно. Если Phase 8 TUI потребует multi-thread — менять на `OnceLock`.
- **Rollback**: `git revert HEAD` — снимает gix dep и git module; Phase 3 продолжает работать.
- **Windows фикстуры**: `git init -b main` требует git ≥ 2.28. CI matrix windows-2022 включает 2.40+, проблем нет.
