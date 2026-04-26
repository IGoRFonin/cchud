# Task 2 — Минимальный StatusPayload

**Files:**
- Create: `src/types/mod.rs`
- Create: `src/types/payload.rs`
- Modify: `src/main.rs` (добавить `mod types;` или временно `#[allow(dead_code)] mod types;` чтобы компилялось)
- Test: `src/types/payload.rs` (unit-тест внизу файла, deserialize реального семпла)

## Goal

Создать минимальную типизацию `StatusPayload` — только то, что нужно `Model` widget'у (`session_id`, `model.display_name`, `model.id`, `workspace.current_dir`). Покрыть unit-тестом, который реально парсит семпл из `benches/samples/`. Полный envelope (15 полей) расширяется в Task 9.

## Inputs

- Task 1 закрыта (`serial_test` и `tempfile` в Cargo.toml).
- `benches/samples/payload-cchud-sonnet-xlarge.json` существует (Phase 0 артефакт).
- `serde` + `serde_json` уже в `[dependencies]`.

---

- [ ] **Step 1: Создать `src/types/mod.rs`**

Файл-маршрутизатор подмодулей:

```rust
//! Domain types for cchud — payload from Claude Code, user config.

pub mod payload;
```

(`config` подмодуль появится в Task 5, добавим позже.)

- [ ] **Step 2: Создать `src/types/payload.rs` с минимальной схемой**

```rust
//! Minimal StatusPayload — Phase 2 walking skeleton.
//! Full envelope (15 fields incl. cost/context_window/rate_limits as
//! `Option<serde_json::Value>`) is added in Task 9.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub session_id: String,
    pub model: ModelInfo,
    pub workspace: Workspace,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub current_dir: String,
    #[serde(default)]
    pub project_dir: Option<String>,
    #[serde(default)]
    pub added_dirs: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!(
        "../../benches/samples/payload-cchud-sonnet-xlarge.json"
    );

    #[test]
    fn parses_real_payload_sample() {
        let payload: StatusPayload =
            serde_json::from_str(SAMPLE).expect("sample must parse");
        assert_eq!(payload.model.id, "claude-sonnet-4-6");
        assert_eq!(payload.model.display_name, "Sonnet 4.6");
        assert!(!payload.session_id.is_empty());
        assert_eq!(payload.workspace.current_dir, "/Users/igor/mp/startup/cchud");
    }

    #[test]
    fn rejects_missing_required_fields() {
        let bad = r#"{"session_id":"x"}"#;
        assert!(serde_json::from_str::<StatusPayload>(bad).is_err());
    }
}
```

**Note по `include_str!`:** путь `../../benches/samples/...` — относительно `src/types/payload.rs`. Это компилирует семпл в бинарь test-runner'а; не зависит от cwd при `cargo test`.

- [ ] **Step 3: Подключить `mod types;` в `src/main.rs`**

Текущий `main.rs` (Phase 1 skeleton) — заменить полностью на:

```rust
//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 2 walking skeleton: types defined, render pipeline lands in Task 4.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod types;

use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let len = input.len();
    println!("cchud (skeleton) | input bytes: {len}");
    Ok(())
}
```

**Зачем `#![deny(...)]` уже сейчас:** настраивает hot-path lint-discipline на crate-уровне до того, как реальный код появится; Task 4 добавит больше кода, к тому моменту lint уже включён.

**Note:** `mod types` сейчас не используется в `main`, но компилируется и тесты в нём запускаются. Без `#[allow(dead_code)]` потому что `pub` элементы не считаются dead code.

- [ ] **Step 4: Запустить unit-тесты**

```bash
cargo test --locked types::payload
```

Expected output (примерно):
```
running 2 tests
test types::payload::tests::parses_real_payload_sample ... ok
test types::payload::tests::rejects_missing_required_fields ... ok
```

Если `parses_real_payload_sample` падает на `assert_eq!(payload.model.id, ...)` — открыть `benches/samples/payload-cchud-sonnet-xlarge.json` и сверить точное значение `model.id`. Если payload сменился (Anthropic сменил формат) — обновить assert на актуальное значение и записать в `docs/DECISIONS.md`.

Если падает на `expect("sample must parse")` — JSON не соответствует нашей минимальной схеме. Скорее всего `model.display_name` или `workspace.current_dir` отсутствуют/переименованы. Открыть payload, найти расхождение, обновить либо payload sample (если он устарел), либо схему.

- [ ] **Step 5: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все 4 команды exit 0. `cargo test` теперь запускает 1 Phase 1 integration-тест (`renders_skeleton_for_each_phase0_sample`) + 2 новых unit-теста.

Если clippy ругается на `clippy::module_name_repetitions` (имя `ModelInfo` в `types::payload`) — добавить в `src/types/payload.rs` сверху `use` блока:
```rust
#![allow(clippy::module_name_repetitions)]
```

- [ ] **Step 6: Verification — task-specific gate**

```bash
test -f src/types/mod.rs && echo "types/mod.rs ok"
test -f src/types/payload.rs && echo "types/payload.rs ok"
grep -q 'pub struct StatusPayload' src/types/payload.rs && echo "StatusPayload ok"
grep -q 'rename_all = "camelCase"' src/types/payload.rs && echo "camelCase ok"
cargo test --locked types::payload 2>&1 | grep -c "test result: ok"
```

Expected output:
```
types/mod.rs ok
types/payload.rs ok
StatusPayload ok
camelCase ok
1
```

- [ ] **Step 7: Commit**

```bash
git add src/types/mod.rs src/types/payload.rs src/main.rs
git commit -m "feat(phase-2): minimal StatusPayload types

Add types/payload.rs with StatusPayload + ModelInfo + Workspace —
just enough fields for Model widget. Full envelope (cost,
context_window, rate_limits as Option<Value>) lands in Task 9.

Unit tests parse a real Phase 0 sample to lock the schema.
Crate-level deny(unwrap_used, expect_used) enabled.

Task 2/10 of Phase 2.
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

- [ ] `src/types/mod.rs` создан, экспортирует `pub mod payload`
- [ ] `src/types/payload.rs` содержит `StatusPayload`, `ModelInfo`, `Workspace` с `#[serde(rename_all = "camelCase")]`
- [ ] 2 unit-теста в `src/types/payload.rs::tests` зелёные
- [ ] `src/main.rs` компилируется с `mod types;` и `#![deny(clippy::unwrap_used, clippy::expect_used)]`
- [ ] `cargo test --locked` показывает Phase 1 integration + 2 новых unit-теста, все зелёные
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] Один коммит `feat(phase-2): minimal StatusPayload types`

## Files touched

- `src/types/mod.rs` (created)
- `src/types/payload.rs` (created)
- `src/main.rs` (modified — `mod types`, deny lints)

## Risks & rollback

- **Payload sample устарел** (Anthropic переименовал поле): Step 4 fallback — обновить либо схему, либо семпл, записать в `docs/DECISIONS.md`. Скорее всего семпл актуален (Phase 0 свежий).
- **`include_str!` относительный путь не находит файл**: путь `../../benches/samples/...` корректен относительно `src/types/payload.rs`. Если упало — `pwd` при компиляции = корень крейта, а `include_str!` относителен к файлу с макросом. Проверить ровно: `src/types/payload.rs` → `../../benches/...`.
- **`#![deny(clippy::unwrap_used)]` ломает существующий `src/main.rs`**: Phase 1 skeleton использует `?` operator, не `unwrap`/`expect` — должен быть чист. Если падает — найти источник и заменить на `if let Err(e) = ... { eprintln!(...); return ... }`.
- **Rollback**: `git revert HEAD` — безопасно, изменения изолированы.
