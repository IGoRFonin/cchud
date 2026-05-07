# Task 9 — widgets/env.rs — ClaudeAccountEmail + FreeMemory

**Цель:** Реализовать 2 env-виджета: `ClaudeAccountEmail` (читает через `commands::env_loader::claude_account_email()`) и `FreeMemory` (через `sysinfo::System::refresh_memory()` + `format_memory::format`). Привязать в `build_one`.

**Files:**
- Modify: `src/widgets/env.rs` — `ClaudeAccountEmail`, `FreeMemory`
- Modify: `src/widgets/mod.rs::build_one` — 2 case'а

---

- [ ] **Step 1: Write failing tests**

В `src/widgets/env.rs`:

```rust
//! Environment widgets — Phase 7 Task 9.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct ClaudeAccountEmail;
pub struct FreeMemory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::test_helpers::payload_no_transcript;

    #[test]
    fn free_memory_returns_some_string() {
        let p = payload_no_transcript();
        let s = crate::config::default_line();
        let ctx = RenderContext::new(&p, &s);
        let out = FreeMemory.render(&ctx).expect("system memory always present");
        assert!(out.starts_with("💾 "), "got: {out}");
        // суффикс: одна из b/k/M/G/T
        assert!(out.chars().last().is_some_and(|c| "bkMGT".contains(c)));
    }

    #[test]
    fn claude_account_email_reads_from_test_file() {
        // Hermetic: не используем глобальный OnceLock.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        std::fs::write(&path, r#"{"oauth_account":{"email_address":"u@x.com"}}"#).unwrap();
        let json = crate::commands::env_loader::read_from_path(&path).unwrap();
        assert_eq!(
            json.oauth_account.unwrap().email_address.as_deref(),
            Some("u@x.com")
        );
    }
}
```

- [ ] **Step 2: Run failing test**

Run: `cargo test --lib widgets::env`
Expected: FAIL.

- [ ] **Step 3: Реализовать виджеты**

В `src/widgets/env.rs` (после struct-определений):

```rust
impl Widget for ClaudeAccountEmail {
    fn id(&self) -> &'static str { "claude-account-email" }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        crate::commands::env_loader::claude_account_email().map(str::to_string)
    }
}

impl Widget for FreeMemory {
    fn id(&self) -> &'static str { "free-memory" }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        let bytes = sys.available_memory();
        Some(format!("💾 {}", crate::util::format_memory::format(bytes)))
    }
}
```

- [ ] **Step 4: Привязать в build_one**

В `src/widgets/mod.rs::build_one`:

```rust
        // Phase 7 — env cluster:
        WidgetConfig::ClaudeAccountEmail => Box::new(env::ClaudeAccountEmail),
        WidgetConfig::FreeMemory => Box::new(env::FreeMemory),
```

(Удалить эти варианты из stub-блока T4.)

- [ ] **Step 5: Run env tests**

Run: `cargo test --lib widgets::env`
Expected: PASS — оба теста.

- [ ] **Step 6: Run full test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/widgets/env.rs src/widgets/mod.rs
git commit -m "feat(phase-7): T9 — env cluster (ClaudeAccountEmail/FreeMemory)"
```
