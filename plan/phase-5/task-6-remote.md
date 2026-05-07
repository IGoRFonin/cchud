# Task 6 — Remote URL parsing + remote widgets (7)

**Files:**
- Create: `src/git/remote.rs` (`parse_url` hand-parser; 4 формата)
- Modify: `src/git/mod.rs` (вызов `parse_url` в `parse_remotes`; добавить `pub use remote::*;`)
- Create: `src/widgets/git_remote.rs` (7 widget impl)
- Modify: `src/widgets/mod.rs` (`pub mod git_remote;` + 7 match-arms)
- Modify: `src/types/config.rs` (7 enum-варианта)

## Goal

Hand-parser для git remote URL — без зависимости `regex` (экономия ~300 КБ binary). Покрываем 4 формата:

```
git@github.com:owner/repo.git           (SSH short)
ssh://git@gitlab.com/owner/repo.git     (SSH explicit)
https://github.com/owner/repo.git       (HTTPS)
https://github.com/owner/repo            (HTTPS no .git)
```

Затем — 7 виджетов, читающих `info.remotes["origin"]` и `info.remotes["upstream"]`.

| Widget | Render | Note |
|---|---|---|
| `GitOriginOwner` | `origin.owner` | None если нет origin или не парсится |
| `GitOriginRepo` | `origin.repo` | |
| `GitOriginOwnerRepo` | `{owner}/{repo}` | |
| `GitUpstreamOwner` | `upstream.owner` | |
| `GitUpstreamRepo` | `upstream.repo` | |
| `GitUpstreamOwnerRepo` | `{owner}/{repo}` | |
| `GitIsFork` | `"fork"` если origin.owner ≠ upstream.owner | None если один remote или owners совпадают |

## Inputs

- T5 закрыт.
- `cargo test --locked widgets::git_tracking` зелёный.

---

- [ ] **Step 1: Создать `src/git/remote.rs` с hand-parser'ом**

```rust
//! Remote URL parser — Phase 5 Task 6.
//!
//! Hand-parser без зависимости `regex` (экономия ~300 КБ).
//! Покрывает 4 канонических формата git remote URL.

#![deny(clippy::unwrap_used, clippy::expect_used)]

/// Парсит remote URL в `(owner, repo)`. None если формат не распознан.
///
/// Поддерживаемые форматы:
/// - `git@host:owner/repo.git` (SSH short)
/// - `ssh://git@host/owner/repo.git` (SSH explicit)
/// - `https://host/owner/repo.git` (HTTPS)
/// - `https://host/owner/repo` (HTTPS no .git)
#[must_use]
pub fn parse_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // 1. SSH explicit: ssh://[user@]host/path
    if let Some(rest) = url.strip_prefix("ssh://") {
        let after_host = strip_user_at_host_slash(rest)?;
        return parse_owner_repo(after_host);
    }

    // 2. HTTPS / HTTP
    for prefix in ["https://", "http://"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            // rest = host/path...
            let (_, path) = rest.split_once('/')?;
            return parse_owner_repo(path);
        }
    }

    // 3. SSH short: user@host:path
    if let Some((_, path)) = url.split_once(':') {
        // Защита от случая 'C:\path' на Windows: путь после ':' должен начинаться
        // с буквы и содержать '/'.
        if !path.is_empty() && !path.starts_with('/') && path.contains('/') {
            return parse_owner_repo(path);
        }
    }

    None
}

fn strip_user_at_host_slash(s: &str) -> Option<&str> {
    // ssh://...  rest = [user@]host/path
    let after_at = s.split_once('@').map_or(s, |(_, r)| r);
    after_at.split_once('/').map(|(_, path)| path)
}

fn parse_owner_repo(path: &str) -> Option<(String, String)> {
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.rsplitn(2, '/');
    let repo = parts.next()?;
    let owner = parts.next()?;
    if repo.is_empty() || owner.is_empty() {
        return None;
    }
    // Group в gitlab может быть `group/subgroup/repo`. Берём *последние* два сегмента.
    // owner = последний сегмент перед repo (не вся group-цепочка, чтобы соответствовать
    // ccstatusline upstream поведению — он берёт `group/repo` или `subgroup/repo`).
    let owner = owner.rsplit('/').next().unwrap_or(owner);
    Some((owner.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn parses_ssh_short() {
        assert_eq!(
            parse_url("git@github.com:foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_ssh_explicit() {
        assert_eq!(
            parse_url("ssh://git@gitlab.com/foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_with_dot_git() {
        assert_eq!(
            parse_url("https://github.com/foo/bar.git"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_without_dot_git() {
        assert_eq!(
            parse_url("https://github.com/foo/bar"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_https_with_trailing_slash() {
        assert_eq!(
            parse_url("https://github.com/foo/bar/"),
            Some(("foo".into(), "bar".into()))
        );
    }

    #[test]
    fn parses_gitlab_subgroup_takes_immediate_owner() {
        // `group/subgroup/repo` — owner = `subgroup`, repo = `repo`
        assert_eq!(
            parse_url("https://gitlab.com/group/subgroup/repo.git"),
            Some(("subgroup".into(), "repo".into()))
        );
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse_url(""), None);
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_url("not a url"), None);
        assert_eq!(parse_url("ftp://example.com/foo/bar"), None);
    }

    #[test]
    fn rejects_windows_path_lookalike() {
        // 'C:\foo\bar' — после ':' нет '/', не воспримем как SSH.
        assert_eq!(parse_url("C:foo\\bar"), None);
    }

    #[test]
    fn handles_dashes_and_dots_in_repo_name() {
        assert_eq!(
            parse_url("git@github.com:foo-bar/repo.with.dots.git"),
            Some(("foo-bar".into(), "repo.with.dots".into()))
        );
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/git/remote.rs`.

- [ ] **Step 2: Подключить `parse_url` в `parse_remotes`**

Edit `src/git/mod.rs`:

Edit 1 (объявить mod):
- `old_string`: `#[cfg(test)]\npub mod fixture;`
- `new_string`: `#[cfg(test)]\npub mod fixture;\npub mod remote;`

Edit 2 (вызвать `parse_url` при заполнении `RemoteInfo`):
- `old_string`:
  ```rust
          out.insert(
              name.to_string(),
              RemoteInfo { url: url_str, owner: None, repo: None },
          );
  ```
- `new_string`:
  ```rust
          let (owner, repo) = remote::parse_url(&url_str).map_or((None, None), |(o, r)| (Some(o), Some(r)));
          out.insert(
              name.to_string(),
              RemoteInfo { url: url_str, owner, repo },
          );
  ```

- [ ] **Step 3: Расширить `WidgetConfig` 7 вариантами**

Edit `src/types/config.rs`:
- `old_string`:
  ```rust
      // Phase 5 — Task 5 (tracking):
      GitAheadBehind,
  }
  ```
- `new_string`:
  ```rust
      // Phase 5 — Task 5 (tracking):
      GitAheadBehind,
      // Phase 5 — Task 6 (remote):
      GitOriginOwner,
      GitOriginRepo,
      GitOriginOwnerRepo,
      GitUpstreamOwner,
      GitUpstreamRepo,
      GitUpstreamOwnerRepo,
      GitIsFork,
  }
  ```

Тест:
```rust
#[test]
fn parses_phase5_remote_widgets() {
    let json = r#"[
        { "type": "git-origin-owner" },
        { "type": "git-origin-repo" },
        { "type": "git-origin-owner-repo" },
        { "type": "git-upstream-owner" },
        { "type": "git-upstream-repo" },
        { "type": "git-upstream-owner-repo" },
        { "type": "git-is-fork" }
    ]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert_eq!(widgets.len(), 7);
    assert!(matches!(widgets[6], WidgetConfig::GitIsFork));
}
```

- [ ] **Step 4: Создать `src/widgets/git_remote.rs`**

```rust
//! Git remote widgets — Phase 5 Task 6.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::git::GitInfo;
use crate::widgets::{RenderContext, Widget};

pub struct GitOriginOwner;
pub struct GitOriginRepo;
pub struct GitOriginOwnerRepo;
pub struct GitUpstreamOwner;
pub struct GitUpstreamRepo;
pub struct GitUpstreamOwnerRepo;
pub struct GitIsFork;

fn remote_owner(g: &GitInfo, name: &str) -> Option<String> {
    g.remotes.get(name)?.owner.clone()
}

fn remote_repo(g: &GitInfo, name: &str) -> Option<String> {
    g.remotes.get(name)?.repo.clone()
}

fn remote_owner_repo(g: &GitInfo, name: &str) -> Option<String> {
    let r = g.remotes.get(name)?;
    let owner = r.owner.as_deref()?;
    let repo = r.repo.as_deref()?;
    Some(format!("{owner}/{repo}"))
}

impl Widget for GitOriginOwner {
    fn id(&self) -> &'static str { "GitOriginOwner" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner(ctx.git()?, "origin")
    }
}
impl Widget for GitOriginRepo {
    fn id(&self) -> &'static str { "GitOriginRepo" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_repo(ctx.git()?, "origin")
    }
}
impl Widget for GitOriginOwnerRepo {
    fn id(&self) -> &'static str { "GitOriginOwnerRepo" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner_repo(ctx.git()?, "origin")
    }
}
impl Widget for GitUpstreamOwner {
    fn id(&self) -> &'static str { "GitUpstreamOwner" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner(ctx.git()?, "upstream")
    }
}
impl Widget for GitUpstreamRepo {
    fn id(&self) -> &'static str { "GitUpstreamRepo" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_repo(ctx.git()?, "upstream")
    }
}
impl Widget for GitUpstreamOwnerRepo {
    fn id(&self) -> &'static str { "GitUpstreamOwnerRepo" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        remote_owner_repo(ctx.git()?, "upstream")
    }
}
impl Widget for GitIsFork {
    fn id(&self) -> &'static str { "GitIsFork" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let git = ctx.git()?;
        let origin_owner = git.remotes.get("origin")?.owner.as_deref()?;
        let upstream_owner = git.remotes.get("upstream")?.owner.as_deref()?;
        if origin_owner != upstream_owner {
            Some("fork".into())
        } else {
            None
        }
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
    fn origin_owner_repo_from_ssh_url() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitOriginOwner.render(&ctx).as_deref(), Some("foo"));
        assert_eq!(GitOriginRepo.render(&ctx).as_deref(), Some("bar"));
        assert_eq!(GitOriginOwnerRepo.render(&ctx).as_deref(), Some("foo/bar"));
    }

    #[test]
    fn upstream_owner_from_https_url() {
        let f = GitFixture::new();
        f.add_remote("upstream", "https://github.com/baz/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitUpstreamOwner.render(&ctx).as_deref(), Some("baz"));
        assert_eq!(GitUpstreamRepo.render(&ctx).as_deref(), Some("bar"));
        assert_eq!(GitUpstreamOwnerRepo.render(&ctx).as_deref(), Some("baz/bar"));
    }

    #[test]
    fn no_remotes_yields_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitOriginOwner.render(&ctx), None);
        assert_eq!(GitUpstreamOwner.render(&ctx), None);
        assert_eq!(GitIsFork.render(&ctx), None);
    }

    #[test]
    fn is_fork_when_origin_differs_from_upstream() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        f.add_remote("upstream", "git@github.com:them/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx).as_deref(), Some("fork"));
    }

    #[test]
    fn is_fork_none_when_owners_match() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        f.add_remote("upstream", "git@github.com:me/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx), None);
    }

    #[test]
    fn is_fork_none_with_only_origin() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:me/bar.git");
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitIsFork.render(&ctx), None);
    }
}
```

- [ ] **Step 5: Регистрация**

Edit `src/widgets/mod.rs`:

Edit 1: `pub mod git_remote;` после `pub mod git_tracking;`.

Edit 2:
- `old_string`:
  ```rust
          // Phase 5 — Task 5 (tracking):
          WidgetConfig::GitAheadBehind => Box::new(git_tracking::GitAheadBehind),
      }
  ```
- `new_string`:
  ```rust
          // Phase 5 — Task 5 (tracking):
          WidgetConfig::GitAheadBehind => Box::new(git_tracking::GitAheadBehind),
          // Phase 5 — Task 6 (remote):
          WidgetConfig::GitOriginOwner => Box::new(git_remote::GitOriginOwner),
          WidgetConfig::GitOriginRepo => Box::new(git_remote::GitOriginRepo),
          WidgetConfig::GitOriginOwnerRepo => Box::new(git_remote::GitOriginOwnerRepo),
          WidgetConfig::GitUpstreamOwner => Box::new(git_remote::GitUpstreamOwner),
          WidgetConfig::GitUpstreamRepo => Box::new(git_remote::GitUpstreamRepo),
          WidgetConfig::GitUpstreamOwnerRepo => Box::new(git_remote::GitUpstreamOwnerRepo),
          WidgetConfig::GitIsFork => Box::new(git_remote::GitIsFork),
      }
  ```

- [ ] **Step 6: Тесты + standard gate**

```bash
cargo test --locked --lib git::remote
cargo test --locked --lib widgets::git_remote
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: ≥10 unit-тестов в `git::remote::tests` (4 формата + edge cases), 6 в `widgets::git_remote::tests`.

- [ ] **Step 7: Verification**

```bash
grep -c 'pub fn parse_url' src/git/remote.rs
grep -c 'GitOriginOwner\b' src/widgets/git_remote.rs
grep -c 'GitIsFork' src/widgets/git_remote.rs
grep -c 'parse_url' src/git/mod.rs
```

Expected: каждый ≥1.

- [ ] **Step 8: Commit**

```bash
git add src/git/remote.rs src/git/mod.rs src/widgets/git_remote.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T6 remote — parse_url + 7 widgets

Hand-written URL parser (no regex dep, ~300KB binary save) covering:
- git@host:owner/repo.git (SSH short)
- ssh://[user@]host/owner/repo.git (SSH explicit)
- https://host/owner/repo[.git][/]
- gitlab subgroup paths (uses immediate parent as owner)

7 widgets read GitInfo.remotes[origin|upstream]:
- GitOriginOwner / Repo / OwnerRepo
- GitUpstreamOwner / Repo / OwnerRepo
- GitIsFork ('fork' string when origin.owner ≠ upstream.owner)

Task 6/9 of Phase 5. Cumulative: 19/20.
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

- [ ] `src/git/remote.rs` создан с `parse_url`
- [ ] 4 формата URL парсятся корректно (SSH short/explicit, HTTPS с/без `.git`)
- [ ] GitLab subgroup → owner = immediate parent (`group/subgroup/repo` → `subgroup/repo`)
- [ ] Невалидные URL (Windows-пути, ftp://, garbage) → None
- [ ] `parse_remotes` заполняет `RemoteInfo.owner`/`.repo`
- [ ] 7 widget-структур в `src/widgets/git_remote.rs`
- [ ] 7 enum-вариантов в `WidgetConfig`
- [ ] `GitIsFork` рендерит `"fork"` только когда оба remote'а есть и owner'ы различны
- [ ] ≥10 unit-тестов на `parse_url`, ≥6 на виджеты
- [ ] Один commit `feat(phase-5): T6 remote ...`

## Files touched

- `src/git/remote.rs` (created)
- `src/git/mod.rs` (modified — `pub mod remote`, вызов `parse_url`)
- `src/widgets/git_remote.rs` (created)
- `src/widgets/mod.rs` (modified)
- `src/types/config.rs` (modified — 7 enum variants)

## Risks & rollback

- **GitLab subgroup-семантика спорна**: ccstatusline upstream поведение не документировано чётко. Текущий выбор — `immediate parent as owner` — прагматичный. Если пользователи жалуются — поправить в Phase 9 (без breaking change в API).
- **URLs с auth-токенами**: `https://oauth:token@github.com/owner/repo.git` парсится корректно? `url.split_once('/')` на пути после `https://` пройдёт через `oauth:token@github.com`. Тест на это не делаем — экзотический сценарий, можно добавить в Phase 7.
- **Windows path collision** `C:\foo`: тест `rejects_windows_path_lookalike` лочит. На Windows `git remote -v` всё равно даёт unix-style URL.
- **Rollback**: `git revert HEAD` — снимает 7 виджетов и parse_url; `parse_remotes` возвращается к None для owner/repo.
