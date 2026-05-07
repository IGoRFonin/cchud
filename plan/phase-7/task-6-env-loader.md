# Task 6 — commands::env_loader (ClaudeJson + OnceLock)

**Цель:** Реализовать `~/.claude.json` reader с process-wide `OnceLock`-кэшем. Возвращает `oauth_account.email_address` для виджета `ClaudeAccountEmail` (T9). Test API через `read_from_path` для hermetic-тестов.

**Files:**
- Modify: `src/commands/env_loader.rs` — `ClaudeJson`, `OauthAccount`, `OnceLock`, `claude_account_email()`, `read_from_path()`, `set_claude_json_for_tests()`

---

- [ ] **Step 1: Write failing tests**

В `src/commands/env_loader.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_no_oauth_account() {
        let json = ClaudeJson { oauth_account: None };
        assert_eq!(extract_email(&Some(json)), None);
    }

    #[test]
    fn returns_email_when_present() {
        let json = ClaudeJson {
            oauth_account: Some(OauthAccount {
                email_address: Some("igor@example.com".into()),
            }),
        };
        assert_eq!(extract_email(&Some(json)), Some("igor@example.com"));
    }

    #[test]
    fn returns_none_when_email_missing() {
        let json = ClaudeJson {
            oauth_account: Some(OauthAccount { email_address: None }),
        };
        assert_eq!(extract_email(&Some(json)), None);
    }

    #[test]
    fn read_from_path_returns_none_for_missing_file() {
        let none = read_from_path(std::path::Path::new("/__nonexistent_does_not_exist.json"));
        assert!(none.is_none());
    }

    #[test]
    fn read_from_path_parses_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        std::fs::write(&path, r#"{"oauth_account":{"email_address":"u@x.com"}}"#).unwrap();
        let json = read_from_path(&path).unwrap();
        assert_eq!(
            json.oauth_account.unwrap().email_address.as_deref(),
            Some("u@x.com")
        );
    }

    #[test]
    fn parses_claude_json_with_oauth_email() {
        let body = r#"{"oauth_account": {"email_address": "user@example.com"}}"#;
        let parsed: ClaudeJson = serde_json::from_str(body).unwrap();
        assert_eq!(
            parsed.oauth_account.unwrap().email_address.unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn parses_claude_json_without_oauth() {
        let body = r#"{"theme": "dark"}"#;
        let parsed: ClaudeJson = serde_json::from_str(body).unwrap();
        assert!(parsed.oauth_account.is_none());
    }
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --lib commands::env_loader`
Expected: FAIL.

- [ ] **Step 3: Реализовать env_loader**

В `src/commands/env_loader.rs`:

```rust
//! ~/.claude.json reader — process-wide OnceLock cache.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeJson {
    #[serde(default)]
    pub oauth_account: Option<OauthAccount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthAccount {
    #[serde(default)]
    pub email_address: Option<String>,
}

static CLAUDE_JSON: OnceLock<Option<ClaudeJson>> = OnceLock::new();

/// Public API. Возвращает email из `~/.claude.json::oauth_account.email_address`.
/// None если файла нет, не парсится, или поля отсутствуют.
#[must_use]
pub fn claude_account_email() -> Option<&'static str> {
    let json = CLAUDE_JSON
        .get_or_init(|| {
            let path = dirs::home_dir()?.join(".claude.json");
            read_from_path(&path)
        })
        .as_ref()?;
    json.oauth_account.as_ref()?.email_address.as_deref()
}

/// Pure helper: читает и парсит файл по абсолютному пути. Не трогает OnceLock.
#[must_use]
pub fn read_from_path(path: &Path) -> Option<ClaudeJson> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice::<ClaudeJson>(&bytes).ok()
}

/// Pure helper для unit-тестов.
#[cfg(test)]
fn extract_email(json: &Option<ClaudeJson>) -> Option<&str> {
    json.as_ref()?.oauth_account.as_ref()?.email_address.as_deref()
}

#[cfg(test)]
#[allow(dead_code)]
pub fn set_claude_json_for_tests(json: Option<ClaudeJson>) {
    let _ = CLAUDE_JSON.set(json);
}
```

- [ ] **Step 4: Run env_loader tests**

Run: `cargo test --lib commands::env_loader`
Expected: PASS — все 7 тестов.

- [ ] **Step 5: Run full test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/commands/env_loader.rs
git commit -m "feat(phase-7): T6 — commands::env_loader with OnceLock-cached ClaudeJson"
```
