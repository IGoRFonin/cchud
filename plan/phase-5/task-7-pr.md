# Task 7 — GitPr (HTTP + auth + bincode disk cache)

**Files:**
- Modify: `Cargo.toml` (add `ureq = "2"`, `bincode = "1"`, `serde` уже есть; dev: `mockito = "1"`)
- Create: `src/git/pr.rs` (`PrCache`, `CachedPr`, `PrInfo`, `lookup_or_fetch`, `github_token`)
- Create: `src/widgets/git_pr.rs` (`GitPr`)
- Modify: `src/widgets/mod.rs` (`pub mod git_pr;` + 1 match-arm)
- Modify: `src/types/config.rs` (1 enum-вариант: `GitPr`)

## Goal

Самый сложный виджет фазы — единственный, который ходит в сеть. Контракт:

1. **Auth priority**: `GITHUB_TOKEN` env → `gh auth token` (subprocess) → анонимно.
2. **HTTP**: `ureq` GET `api.github.com/repos/{owner}/{repo}/pulls?head={owner}:{branch}` с timeout 200 ms.
3. **Cache**: `~/.cache/cchud/pr-cache.bincode`, TTL 30 s, `{ version: u8, entries: HashMap<String, CachedPr> }`. Schema mismatch → молча reset.
4. **Offline soft-fail**: timeout/network error → возвращаем закэшированную (даже stale) запись или None; виджет молча скрывается, pipeline не падает.
5. **Hot-path**: cache-hit < 1 ms (read file + bincode-decode + lookup); cache-miss = 200 ms hard cap.

| Widget | Render | Note |
|---|---|---|
| `GitPr` | `"PR #{number}"` если открытый PR найден | None если нет origin/branch, нет токена + rate-limited, или offline без кэша |

## Inputs

- T6 закрыт. `info.remotes["origin"].owner/.repo` заполнены.
- `info.head.branch` доступен.
- `cargo test --locked widgets::git_remote` зелёный.

---

- [ ] **Step 1: Добавить deps**

Edit `Cargo.toml`:
- `old_string`:
  ```toml
  # Phase 5 — git via gix (pure Rust, no libgit2 system dep).
  # Pinned to exact version: gix has frequent minor breaking changes;
  # bumping is a deliberate PR with smoke-тест.
  gix = { version = "=0.81.0", default-features = false, features = ["max-performance-safe"] }

  # Lazy для виджетов (добавятся в фазах 6-7)
  # sonic-rs, ureq, bincode, ratatui, crossterm — позже
  ```
- `new_string`:
  ```toml
  # Phase 5 — git via gix (pure Rust, no libgit2 system dep).
  gix = { version = "=0.81.0", default-features = false, features = ["max-performance-safe"] }

  # Phase 5 — Task 7: GitPr HTTP + cache.
  # ureq sync HTTP (no async runtime); rustls-tls (pure Rust).
  ureq = { version = "2", default-features = false, features = ["json", "tls", "gzip"] }
  bincode = "1"

  # Lazy для виджетов (добавятся в фазе 6)
  # sonic-rs, ratatui, crossterm — позже
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

Edit dev-deps:
- `old_string`:
  ```toml
  [dev-dependencies]
  insta = { version = "1", features = ["json", "glob"] }
  assert_cmd = "2"
  predicates = "3"
  serial_test = "3"
  tempfile = "3"
  ```
- `new_string`:
  ```toml
  [dev-dependencies]
  insta = { version = "1", features = ["json", "glob"] }
  assert_cmd = "2"
  predicates = "3"
  serial_test = "3"
  tempfile = "3"
  # Phase 5 T7: HTTP mock for GitPr.
  mockito = "1"
  ```

- [ ] **Step 2: Создать `src/git/pr.rs`**

```rust
//! GitPr — Phase 5 Task 7.
//!
//! Единственный сетевой виджет фазы. Контракт:
//! - cache-hit < 1 ms; cache-miss = 200 ms hard timeout.
//! - offline soft-fail: error → cached value (даже stale) или None.
//! - кэш bincode-сериализован, TTL 30 s, schema-mismatch → reset.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const CACHE_VERSION: u8 = 1;
const TTL_SECS: u64 = 30;
const FETCH_TIMEOUT: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrInfo {
    pub number: u32,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPr {
    pub fetched_at: u64,
    pub pr: Option<PrInfo>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PrCache {
    pub version: u8,
    pub entries: HashMap<String, CachedPr>,
}

impl PrCache {
    fn fresh() -> Self {
        Self { version: CACHE_VERSION, entries: HashMap::new() }
    }
}

fn cache_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("cchud").join("pr-cache.bincode"))
}

fn read_cache() -> PrCache {
    let Some(path) = cache_path() else { return PrCache::fresh(); };
    let Ok(bytes) = std::fs::read(&path) else { return PrCache::fresh(); };
    let Ok(cache): Result<PrCache, _> = bincode::deserialize(&bytes) else {
        return PrCache::fresh();
    };
    if cache.version != CACHE_VERSION {
        return PrCache::fresh();
    }
    cache
}

fn write_cache(cache: &PrCache) {
    let Some(path) = cache_path() else { return; };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(bytes) = bincode::serialize(cache) {
        let _ = std::fs::write(&path, bytes);
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Auth priority:
/// 1. `GITHUB_TOKEN` env
/// 2. `gh auth token` subprocess (~50 ms, кэшируется в первый вызов)
/// 3. None — анонимный запрос (rate limit 60/h)
#[must_use]
pub fn github_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    let out = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[derive(Deserialize)]
struct PrJson {
    number: u32,
    state: String,
}

fn fetch_pr(api_base: &str, owner: &str, repo: &str, branch: &str, token: Option<&str>) -> Option<PrInfo> {
    let url = format!("{api_base}/repos/{owner}/{repo}/pulls?head={owner}:{branch}&state=open");
    let mut req = ureq::get(&url)
        .set("User-Agent", "cchud")
        .set("Accept", "application/vnd.github+json")
        .timeout(FETCH_TIMEOUT);
    if let Some(t) = token {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let resp = req.call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let prs: Vec<PrJson> = resp.into_json().ok()?;
    let p = prs.into_iter().next()?;
    Some(PrInfo { number: p.number, state: p.state })
}

/// Look up `(owner, repo, branch)` in cache. Cache-hit (< TTL) → return.
/// Cache-miss → fetch with timeout. On any error → return cached entry
/// (even stale) or None.
pub fn lookup_or_fetch(owner: &str, repo: &str, branch: &str) -> Option<PrInfo> {
    lookup_or_fetch_with_base("https://api.github.com", owner, repo, branch)
}

pub fn lookup_or_fetch_with_base(
    api_base: &str,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Option<PrInfo> {
    let key = format!("{owner}/{repo}:{branch}");
    let mut cache = read_cache();
    let now = now_secs();

    if let Some(entry) = cache.entries.get(&key) {
        if now.saturating_sub(entry.fetched_at) < TTL_SECS {
            return entry.pr.clone();
        }
    }

    let token = github_token();
    let pr = fetch_pr(api_base, owner, repo, branch, token.as_deref());

    // Если fetch упал, но есть stale запись — отдадим её.
    if pr.is_none() {
        if let Some(entry) = cache.entries.get(&key) {
            return entry.pr.clone();
        }
    }

    cache.entries.insert(key, CachedPr { fetched_at: now, pr: pr.clone() });
    write_cache(&cache);
    pr
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use serial_test::serial;

    #[test]
    fn cache_roundtrip_via_bincode() {
        let mut cache = PrCache::fresh();
        cache.entries.insert(
            "foo/bar:main".into(),
            CachedPr { fetched_at: 1234, pr: Some(PrInfo { number: 42, state: "open".into() }) },
        );
        let bytes = bincode::serialize(&cache).unwrap();
        let decoded: PrCache = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries["foo/bar:main"].pr.as_ref().unwrap().number, 42);
    }

    #[test]
    fn cache_with_wrong_version_is_reset() {
        let bad = PrCache { version: 99, entries: HashMap::new() };
        let bytes = bincode::serialize(&bad).unwrap();
        // Имитируем чтение через парс + version-check.
        let cache: PrCache = bincode::deserialize(&bytes).unwrap();
        let cache = if cache.version != CACHE_VERSION { PrCache::fresh() } else { cache };
        assert_eq!(cache.version, CACHE_VERSION);
        assert!(cache.entries.is_empty());
    }

    #[test]
    #[serial]
    fn fetch_with_mock_returns_first_open_pr() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/repos/foo/bar/pulls")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("head".into(), "foo:main".into()),
                mockito::Matcher::UrlEncoded("state".into(), "open".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"number":42,"state":"open"}]"#)
            .create();

        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert_eq!(pr, Some(PrInfo { number: 42, state: "open".into() }));
    }

    #[test]
    #[serial]
    fn fetch_returns_none_on_404() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/repos/foo/bar/pulls").with_status(404).create();
        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert!(pr.is_none());
    }

    #[test]
    #[serial]
    fn fetch_returns_none_on_empty_pr_list() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/repos/foo/bar/pulls")
            .with_status(200)
            .with_body("[]")
            .create();
        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert!(pr.is_none());
    }

    #[test]
    #[serial]
    fn fetch_includes_authorization_header_when_token_set() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/repos/foo/bar/pulls")
            .match_header("authorization", "Bearer mytoken")
            .with_status(200)
            .with_body("[]")
            .create();
        let _ = fetch_pr(&server.url(), "foo", "bar", "main", Some("mytoken"));
        // Mockito panics if matcher fails — explicit assertion.
    }

    #[test]
    #[serial]
    fn offline_returns_none_within_timeout() {
        // Адрес, на котором гарантированно никого нет.
        let start = std::time::Instant::now();
        let pr = fetch_pr("http://127.0.0.1:1", "foo", "bar", "main", None);
        let elapsed = start.elapsed();
        assert!(pr.is_none());
        assert!(elapsed < Duration::from_millis(500), "must respect timeout, took {elapsed:?}");
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/git/pr.rs`.

> **Critical**: тесты используют `serial_test` (уже в dev-deps) для изоляции — `GITHUB_TOKEN` env влияет на `github_token()`, mockito-серверы могут конфликтовать в parallel.

> **Note**: `dirs` уже подключён в Cargo.toml runtime deps. Если нет — добавить `dirs = "6"`.

- [ ] **Step 3: Подключить mod**

Edit `src/git/mod.rs`:
- `old_string`: `pub mod remote;`
- `new_string`: `pub mod remote;\npub mod pr;`

- [ ] **Step 4: Расширить `WidgetConfig`**

Edit `src/types/config.rs`:
- `old_string`:
  ```rust
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
- `new_string`:
  ```rust
      // Phase 5 — Task 6 (remote):
      GitOriginOwner,
      GitOriginRepo,
      GitOriginOwnerRepo,
      GitUpstreamOwner,
      GitUpstreamRepo,
      GitUpstreamOwnerRepo,
      GitIsFork,
      // Phase 5 — Task 7 (PR):
      GitPr,
  }
  ```

Тест:
```rust
#[test]
fn parses_phase5_pr_widget() {
    let json = r#"[{ "type": "git-pr" }]"#;
    let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0], WidgetConfig::GitPr));
}
```

- [ ] **Step 5: Создать `src/widgets/git_pr.rs`**

```rust
//! GitPr widget — Phase 5 Task 7.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::git::pr;
use crate::widgets::{RenderContext, Widget};

pub struct GitPr;

impl Widget for GitPr {
    fn id(&self) -> &'static str { "GitPr" }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let git = ctx.git()?;
        let origin = git.remotes.get("origin")?;
        let owner = origin.owner.as_deref()?;
        let repo = origin.repo.as_deref()?;
        let branch = git.head.branch.as_deref()?;
        let info = pr::lookup_or_fetch(owner, repo, branch)?;
        Some(format!("PR #{}", info.number))
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
    fn no_origin_returns_none() {
        let f = GitFixture::new();
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitPr.render(&ctx), None);
    }

    #[test]
    fn no_branch_returns_none() {
        let f = GitFixture::new();
        f.add_remote("origin", "git@github.com:foo/bar.git");
        f.write_file("a.txt", "x");
        f.git(&["add", "a.txt"]);
        f.commit("c2");
        f.git(&["checkout", "--detach", "HEAD"]);
        let p = payload(f.path().to_str().unwrap());
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(GitPr.render(&ctx), None);
    }

    // Note: интеграционный тест GitPr с моком api.github.com сложен —
    // lookup_or_fetch хардкодит "https://api.github.com". Тесты в pr::tests
    // покрывают fetch через lookup_or_fetch_with_base. Этот widget — тонкая обёртка.
}
```

- [ ] **Step 6: Регистрация**

Edit `src/widgets/mod.rs`:

Edit 1: `pub mod git_pr;` после `pub mod git_remote;`.

Edit 2:
- `old_string`:
  ```rust
          WidgetConfig::GitIsFork => Box::new(git_remote::GitIsFork),
      }
  ```
- `new_string`:
  ```rust
          WidgetConfig::GitIsFork => Box::new(git_remote::GitIsFork),
          // Phase 5 — Task 7 (PR):
          WidgetConfig::GitPr => Box::new(git_pr::GitPr),
      }
  ```

- [ ] **Step 7: Тесты + standard gate**

```bash
cargo build --release --locked
cargo test --locked --lib git::pr
cargo test --locked --lib widgets::git_pr
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. ureq/bincode/mockito компилируются (~30-60 sec первый раз).

```bash
ls -lh target/release/cchud | awk '{print $5}'
```

Expected: < 7.5 MB (gix + ureq+rustls добавят ~1.5-2 MB к T1 baseline).

- [ ] **Step 8: Manual real-test (опционально, требует сети)**

```bash
cd /Users/igor/mp/startup/cchud
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}' \
  | target/release/cchud
```

Сделать конфиг с `git-pr` и запустить:
```bash
mkdir -p ~/.config/ccstatusline
cat > /tmp/cchud-pr-test.json <<'EOF'
{ "lines": [{ "widgets": [{ "type": "git-pr" }] }] }
EOF
echo '{"session_id":"x","model":{"id":"m","display_name":"M"},"workspace":{"current_dir":"'$(pwd)'"}}' \
  | CCHUD_CONFIG=/tmp/cchud-pr-test.json target/release/cchud
```

Expected: если на текущей ветке есть PR — `PR #N`. Иначе — пусто (виджет вернул None).

```bash
ls ~/.cache/cchud/
```

Expected: `pr-cache.bincode` создан после первого запроса.

- [ ] **Step 9: Verification**

```bash
grep -c 'pub fn lookup_or_fetch' src/git/pr.rs
grep -c 'pub fn github_token' src/git/pr.rs
grep -c 'CACHE_VERSION' src/git/pr.rs
grep -c 'FETCH_TIMEOUT' src/git/pr.rs
grep -c 'GitPr' src/widgets/git_pr.rs
grep -c 'git_pr::GitPr' src/widgets/mod.rs
grep -c 'ureq' Cargo.toml
grep -c 'bincode' Cargo.toml
grep -c 'mockito' Cargo.toml
```

Expected: каждый ≥1.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock src/git/pr.rs src/git/mod.rs src/widgets/git_pr.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-5): T7 GitPr — HTTP + auth + bincode disk cache

Adds ureq (rustls-tls, no async), bincode, mockito (dev) deps.

src/git/pr.rs:
- github_token() priority: GITHUB_TOKEN → 'gh auth token' → anonymous
- lookup_or_fetch(): TTL 30s, hard timeout 200ms, schema-versioned cache
  at ~/.cache/cchud/pr-cache.bincode
- offline soft-fail: network error → stale cached value or None;
  pipeline never blocks
- fetch_pr() params api_base for mockito tests via lookup_or_fetch_with_base

src/widgets/git_pr.rs:
- GitPr renders 'PR #N' when origin owner+repo+branch all parse and
  GitHub returns ≥1 open PR head=owner:branch
- None if anything missing (no origin, detached HEAD, offline+no cache)

7 tests in pr::tests (mockito for HTTP), 2 widget tests for None paths.
GitPr cluster cumulative cost (cache-hit): < 1ms.

Task 7/9 of Phase 5. Cumulative: 20/20 widgets — full git set complete.
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

- [ ] `Cargo.toml` содержит `ureq = "2"` (rustls), `bincode = "1"`, `mockito = "1"` (dev)
- [ ] Binary < 7.5 MB
- [ ] `src/git/pr.rs` создан с `PrCache`, `CachedPr`, `PrInfo`, `lookup_or_fetch`, `github_token`
- [ ] `CACHE_VERSION = 1`, `TTL_SECS = 30`, `FETCH_TIMEOUT = 200ms`
- [ ] Schema mismatch (version ≠ 1) → cache reset
- [ ] Auth priority: env → gh CLI → anonymous
- [ ] Offline (`http://127.0.0.1:1`) укладывается в < 500 ms (timeout 200 ms + jitter)
- [ ] `widgets/git_pr.rs` рендерит `"PR #N"` или None
- [ ] ≥6 unit-тестов в `pr::tests` (mockito) + 2 в `widgets::git_pr::tests`
- [ ] Manual test: cache-файл создаётся в `~/.cache/cchud/pr-cache.bincode`
- [ ] Один commit `feat(phase-5): T7 GitPr ...`

## Files touched

- `Cargo.toml` (modified — 3 new deps)
- `Cargo.lock` (auto)
- `src/git/pr.rs` (created)
- `src/git/mod.rs` (modified — pub mod pr)
- `src/widgets/git_pr.rs` (created)
- `src/widgets/mod.rs` (modified)
- `src/types/config.rs` (modified — 1 enum variant)

## Risks & rollback

- **`ureq + rustls` size impact**: ~1.5 MB. Если binary > 7.5 MB — рассмотреть `ureq` с native-tls (системный openssl, меньше ~500 КБ, но требует системную либу). Pure-Rust выбран ради cargo-install / npm-loader совместимости.
- **`gh auth token` subprocess в hot-path**: ~50 ms на macOS, до 200 ms на Windows. Митигация: вызывается только при cache-miss + отсутствии env; в production cache-hit делает это редким.
- **`api.github.com` rate limit без токена**: 60 req/h. Кэш TTL 30 s даёт 120 запросов/час на одну ветку — превышает лимит при активном переключении веток. Документируем "set GITHUB_TOKEN или gh login для надёжной работы".
- **Bincode breaking change при bump'е версии bincode 1→2**: pin на `"1"` (не major). Bincode 2 имеет новый encode/decode API.
- **Кэш-файл доступен другим пользователям**: `~/.cache/cchud/` имеет дефолтные umask permissions. Содержимое — только PR numbers, не секреты. ОК.
- **Mockito server теряется между тестами**: `serial_test` сериализует. Если flaky — mockito 1.x имеет `#[mockito::async]` не нужен; sync API ОК.
- **Rollback**: `git revert HEAD` — снимает 3 deps + GitPr; Phase 5 остаётся на 19/20 виджетов.
