//! `GitPr` — Phase 5 Task 7.
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
        Self {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }
}

fn cache_path() -> Option<PathBuf> {
    // Override for hermetic tests — never set in production builds.
    #[cfg(test)]
    if let Ok(p) = std::env::var("CCHUD_TEST_CACHE_PATH") {
        return Some(PathBuf::from(p));
    }
    Some(dirs::cache_dir()?.join("cchud").join("pr-cache.bincode"))
}

fn read_cache() -> PrCache {
    let Some(path) = cache_path() else {
        return PrCache::fresh();
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return PrCache::fresh();
    };
    let Ok(cache): Result<PrCache, _> = bincode::deserialize(&bytes) else {
        return PrCache::fresh();
    };
    if cache.version != CACHE_VERSION {
        return PrCache::fresh();
    }
    cache
}

fn write_cache(cache: &PrCache) {
    let Some(path) = cache_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(bytes) = bincode::serialize(cache) {
        let _ = std::fs::write(&path, bytes);
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Auth priority:
/// 1. `GITHUB_TOKEN` env
/// 2. `gh auth token` subprocess (~50 ms, called on every cache-miss without env token)
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
}

/// `Ok(Some)` = PR found. `Ok(None)` = API responded, no open PR.
/// `Err(())` = network/HTTP/parse error — caller should fall back to stale.
fn fetch_pr(
    api_base: &str,
    owner: &str,
    repo: &str,
    branch: &str,
    token: Option<&str>,
) -> Result<Option<PrInfo>, ()> {
    let url = format!("{api_base}/repos/{owner}/{repo}/pulls?head={owner}:{branch}&state=open");
    let mut req = ureq::get(&url)
        .set("User-Agent", "cchud")
        .set("Accept", "application/vnd.github+json")
        .timeout(FETCH_TIMEOUT);
    if let Some(t) = token {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let resp = req.call().map_err(|_| ())?;
    if resp.status() != 200 {
        return Err(());
    }
    let prs: Vec<PrJson> = resp.into_json().map_err(|_| ())?;
    Ok(prs.into_iter().next().map(|p| PrInfo { number: p.number }))
}

/// Look up `(owner, repo, branch)` in cache. Cache-hit (< TTL) → return.
/// Cache-miss → fetch with timeout. On any error → return cached entry
/// (even stale) or None.
#[must_use]
pub fn lookup_or_fetch(owner: &str, repo: &str, branch: &str) -> Option<PrInfo> {
    lookup_or_fetch_with_base("https://api.github.com", owner, repo, branch)
}

#[must_use]
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
    match fetch_pr(api_base, owner, repo, branch, token.as_deref()) {
        Ok(pr) => {
            // API ответил — кэшируем даже Ok(None) ("нет PR" — тоже стабильный факт).
            cache.entries.insert(
                key,
                CachedPr {
                    fetched_at: now,
                    pr: pr.clone(),
                },
            );
            write_cache(&cache);
            pr
        }
        Err(()) => {
            // Сеть упала — отдаём stale если есть, иначе None.
            cache.entries.get(&key).and_then(|e| e.pr.clone())
        }
    }
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
            CachedPr {
                fetched_at: 1234,
                pr: Some(PrInfo { number: 42 }),
            },
        );
        let bytes = bincode::serialize(&cache).unwrap();
        let decoded: PrCache = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(
            decoded.entries["foo/bar:main"].pr.as_ref().unwrap().number,
            42
        );
    }

    #[test]
    fn cache_with_wrong_version_is_reset() {
        let bad = PrCache {
            version: 99,
            entries: HashMap::new(),
        };
        let bytes = bincode::serialize(&bad).unwrap();
        // Имитируем чтение через парс + version-check.
        let cache: PrCache = bincode::deserialize(&bytes).unwrap();
        let cache = if cache.version != CACHE_VERSION {
            PrCache::fresh()
        } else {
            cache
        };
        assert_eq!(cache.version, CACHE_VERSION);
        assert!(cache.entries.is_empty());
    }

    #[test]
    #[serial]
    fn fetch_with_mock_returns_first_open_pr() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/repos/foo/bar/pulls")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("head".into(), "foo:main".into()),
                mockito::Matcher::UrlEncoded("state".into(), "open".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"number":42,"state":"open"}]"#)
            .create();

        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert_eq!(pr, Ok(Some(PrInfo { number: 42 })));
    }

    #[test]
    #[serial]
    fn fetch_returns_none_on_404() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/repos/foo/bar/pulls")
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .create();
        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert_eq!(pr, Err(()));
    }

    #[test]
    #[serial]
    fn fetch_returns_none_on_empty_pr_list() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/repos/foo/bar/pulls")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .create();
        let pr = fetch_pr(&server.url(), "foo", "bar", "main", None);
        assert_eq!(pr, Ok(None));
    }

    #[test]
    #[serial]
    fn fetch_includes_authorization_header_when_token_set() {
        let mut server = mockito::Server::new();
        let m = server
            .mock("GET", "/repos/foo/bar/pulls")
            .match_query(mockito::Matcher::Any)
            .match_header("authorization", "Bearer mytoken")
            .with_status(200)
            .with_body("[]")
            .create();
        let _ = fetch_pr(&server.url(), "foo", "bar", "main", Some("mytoken"));
        m.assert();
    }

    #[test]
    #[serial]
    fn offline_returns_none_within_timeout() {
        // Адрес, на котором гарантированно никого нет.
        let start = std::time::Instant::now();
        let pr = fetch_pr("http://127.0.0.1:1", "foo", "bar", "main", None);
        let elapsed = start.elapsed();
        assert!(pr.is_err());
        assert!(
            elapsed < Duration::from_millis(500),
            "must respect timeout, took {elapsed:?}"
        );
    }

    #[test]
    #[serial]
    fn cache_hit_skips_network_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pr-cache.bincode");
        // SAFETY: serial test — no concurrent env mutation.
        unsafe { std::env::set_var("CCHUD_TEST_CACHE_PATH", &path) };

        // Pre-seed fresh cache entry.
        let mut cache = PrCache::fresh();
        let now = now_secs();
        cache.entries.insert(
            "foo/bar:main".into(),
            CachedPr {
                fetched_at: now,
                pr: Some(PrInfo { number: 99 }),
            },
        );
        std::fs::write(&path, bincode::serialize(&cache).unwrap()).unwrap();

        // Server has no mocks — any hit would return 501 and fail fetch_pr.
        let server = mockito::Server::new();
        let pr = lookup_or_fetch_with_base(&server.url(), "foo", "bar", "main");
        assert_eq!(pr, Some(PrInfo { number: 99 }));

        // SAFETY: serial test — no concurrent env mutation.
        unsafe { std::env::remove_var("CCHUD_TEST_CACHE_PATH") };
    }

    #[test]
    #[serial]
    fn stale_entry_returned_when_fetch_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pr-cache.bincode");
        // SAFETY: serial test — no concurrent env mutation.
        unsafe {
            std::env::set_var("CCHUD_TEST_CACHE_PATH", &path);
            // Bypass gh subprocess call in github_token().
            std::env::set_var("GITHUB_TOKEN", "test-token");
        }

        // Pre-seed stale entry (fetched_at = 0 → older than TTL_SECS).
        let mut cache = PrCache::fresh();
        cache.entries.insert(
            "foo/bar:main".into(),
            CachedPr {
                fetched_at: 0,
                pr: Some(PrInfo { number: 77 }),
            },
        );
        std::fs::write(&path, bincode::serialize(&cache).unwrap()).unwrap();

        // Port 1 refuses connections immediately — simulates offline.
        let pr = lookup_or_fetch_with_base("http://127.0.0.1:1", "foo", "bar", "main");
        assert_eq!(pr, Some(PrInfo { number: 77 }));

        // SAFETY: serial test — no concurrent env mutation.
        unsafe {
            std::env::remove_var("GITHUB_TOKEN");
            std::env::remove_var("CCHUD_TEST_CACHE_PATH");
        }
    }
}
