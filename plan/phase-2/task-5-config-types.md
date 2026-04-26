# Task 5 — Config types (Settings, Line, WidgetConfig, ThemeConfig)

**Files:**
- Create: `src/types/config.rs`
- Modify: `src/types/mod.rs` (добавить `pub mod config;`)
- Test: `src/types/config.rs` (unit-тесты — serde roundtrip)

## Goal

Определить типы конфига (`Settings`, `Line`, `WidgetConfig`, `ThemeConfig`), которые загрузит `config::load` в Task 6. WidgetConfig — enum с одним вариантом `Model` (Phase 3 добавит остальные). ThemeConfig — пустой stub под Phase 4 (Powerline). Покрыть serde roundtrip-тестом.

## Inputs

- Tasks 1–4 закрыты, walking skeleton работает.
- `serde` + `serde_json` в `[dependencies]` (Phase 1).
- `widgets::model::Model` существует, но `ModelParams` ещё нет — создаётся в этой задаче.

---

- [ ] **Step 1: Создать `src/types/config.rs`**

```rust
//! User config schema — parsed from `~/.claude/settings.json` `cchud` block.
//!
//! Phase 2 supports only `WidgetConfig::Model`. Phase 3 adds 9 more variants.
//! `ThemeConfig` is an empty slot — populated in Phase 4 (Powerline colors,
//! separator overrides). Migrations infrastructure intentionally omitted —
//! current `version: 1` is the only version that exists.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub lines: Vec<Line>,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    #[serde(default)]
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetConfig {
    Model {
        #[serde(flatten, default)]
        params: ModelParams,
    },
}

/// Per-widget parameters. Phase 7 adds custom format strings, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelParams {}

/// Empty in Phase 2. Phase 4 will populate (powerline_colors, separator_override, ...).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {}

fn default_version() -> u32 {
    1
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: default_version(),
            lines: Vec::new(),
            theme: ThemeConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_cchud_block() {
        let json = r#"{
            "version": 1,
            "lines": [{"widgets": [{"type": "Model"}]}],
            "theme": {}
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(s.lines.len(), 1);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Model { .. }));
    }

    #[test]
    fn defaults_fill_missing_fields() {
        let json = r#"{}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.version, 1);
        assert!(s.lines.is_empty());
    }

    #[test]
    fn rejects_unknown_widget_type() {
        let json = r#"{
            "lines": [{"widgets": [{"type": "Branch"}]}]
        }"#;
        // Phase 2 не знает про Branch — должен упасть на парсинге.
        // Phase 3 добавит вариант, тест обновится.
        assert!(serde_json::from_str::<Settings>(json).is_err());
    }

    #[test]
    fn serde_roundtrip_preserves_shape() {
        let original = Settings {
            version: 1,
            lines: vec![Line {
                widgets: vec![WidgetConfig::Model {
                    params: ModelParams::default(),
                }],
            }],
            theme: ThemeConfig::default(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.lines.len(), 1);
        assert!(matches!(back.lines[0].widgets[0], WidgetConfig::Model { .. }));
    }
}
```

**Note про `WidgetConfig` enum:** `#[serde(tag = "type")]` означает что в JSON ожидается `{"type": "Model", ...flatten params...}`. Это формат, совместимый с upstream ccstatusline. Phase 3 добавит варианты — `Branch { ... }`, `GitStatus { ... }`, etc.

**Note про `unwrap()` в тестах:** `#[cfg(test)]` модуль не подпадает под crate-level `deny(unwrap_used)`. Если падает — добавить `#![allow(clippy::unwrap_used, clippy::expect_used)]` в начало `mod tests`.

- [ ] **Step 2: Расширить `src/types/mod.rs`**

Текущий:
```rust
//! Domain types for cchud — payload from Claude Code, user config.

pub mod payload;
```

Заменить на:
```rust
//! Domain types for cchud — payload from Claude Code, user config.

pub mod config;
pub mod payload;
```

Edit tool:
- `old_string`: `pub mod payload;`
- `new_string`: `pub mod config;\npub mod payload;`

- [ ] **Step 3: Запустить unit-тесты**

```bash
cargo test --locked types::config
```

Expected output (примерно):
```
running 4 tests
test types::config::tests::defaults_fill_missing_fields ... ok
test types::config::tests::parses_minimal_cchud_block ... ok
test types::config::tests::rejects_unknown_widget_type ... ok
test types::config::tests::serde_roundtrip_preserves_shape ... ok
```

Если `parses_minimal_cchud_block` падает на `matches!(...WidgetConfig::Model)` — проверить `#[serde(tag = "type")]` указан правильно, и `#[serde(flatten, default)] params` не ломает парсинг. Если ломается — попробовать без `flatten`:
```rust
Model { #[serde(default)] params: ModelParams },
```

- [ ] **Step 4: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. К предыдущим тестам добавляются 4 unit-теста в `types::config::tests`.

Если clippy ругается на `clippy::missing_errors_doc` для serde `Deserialize` — false positive, можно через `#[allow(clippy::missing_errors_doc)]` на impl блоке или crate-level в lib.rs (мы пока без lib.rs, в main.rs).

- [ ] **Step 5: Verification — task-specific gate**

```bash
test -f src/types/config.rs && echo "types/config.rs ok"
grep -q 'pub mod config' src/types/mod.rs && echo "config wired ok"
grep -q 'pub struct Settings' src/types/config.rs && echo "Settings ok"
grep -q 'pub enum WidgetConfig' src/types/config.rs && echo "WidgetConfig enum ok"
grep -q 'tag = "type"' src/types/config.rs && echo "tag attribute ok"
grep -q 'fn default_version' src/types/config.rs && echo "default_version ok"
cargo test --locked types::config 2>&1 | grep -E "test result: ok\. \d+ passed"
```

Expected: все строки `ok` + последняя строка показывает 4 теста passed.

- [ ] **Step 6: Commit**

```bash
git add src/types/config.rs src/types/mod.rs
git commit -m "feat(phase-2): config types — Settings, Line, WidgetConfig

- Settings { version, lines, theme } with serde defaults
- Line { widgets: Vec<WidgetConfig> }
- WidgetConfig::Model { params: ModelParams } — Phase 3 will add
  9 more variants
- ThemeConfig {} — empty slot, populated in Phase 4 Powerline
- Migrations infrastructure intentionally omitted (version=1 is
  the only version, drift handled when introduced)

4 unit tests covering serde roundtrip, defaults, and unknown-type
rejection.

Task 5/10 of Phase 2.
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

- [ ] `src/types/config.rs` создан, содержит `Settings`, `Line`, `WidgetConfig::Model { params: ModelParams }`, `ThemeConfig`
- [ ] `WidgetConfig` использует `#[serde(tag = "type")]`
- [ ] `Settings` поля обёрнуты в `#[serde(default)]` где уместно
- [ ] 4 unit-теста зелёные (parses, defaults, rejects unknown, roundtrip)
- [ ] `src/types/mod.rs` экспортирует `pub mod config;`
- [ ] Один коммит `feat(phase-2): config types`

## Files touched

- `src/types/config.rs` (created)
- `src/types/mod.rs` (modified — добавлен `pub mod config`)

## Risks & rollback

- **`#[serde(flatten, default)]` несовместим с tagged enum**: если парсинг падает — упростить до `#[serde(default)] params: ModelParams` без flatten. Phase 7 пересмотрит когда `ModelParams` получит реальные поля.
- **`#[serde(tag = "type")]` конфликтует с другим полем `type`**: формально нет, JSON узнаёт ключ `type` именно как discriminator. Если Anthropic в payload использует `type` где-то — это про payload, не про config.
- **clippy nursery `clippy::derive_partial_eq_without_eq`**: добавить `#[derive(PartialEq, Eq)]` к `Settings`/`Line`/`WidgetConfig` если нужно. Phase 2 пока не требует — `assert!(matches!(...))` достаточно.
- **Rollback**: `git revert HEAD` — изолировано.
