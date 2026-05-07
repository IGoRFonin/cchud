# Task 1 — Setup: deps, payload sub-types, kebab-case, util skeleton

**Files:**
- Modify: `Cargo.toml` (add `wait-timeout = "0.2"` to `[dependencies]`)
- Modify: `Cargo.lock` (auto-regenerated)
- Modify: `src/types/payload.rs` (типизация `cost`, `context_window`, `output_style`; новые поля `vim`, `worktree`)
- Modify: `src/types/config.rs` (`#[serde(rename_all = "kebab-case")]` + 22 новых варианта `WidgetConfig` БЕЗ impl)
- Create: `src/util/mod.rs` (skeleton: `pub mod model_context_size; pub mod duration; pub mod ascii_bar;` — в Task 1 пустые stub-модули)
- Create: `src/util/model_context_size.rs` (заглушка с `pub fn max_tokens_for(_: &str) -> Option<u64> { None }`)
- Create: `src/util/duration.rs` (заглушка с `pub fn format_duration(_: u64) -> String { String::new() }`)
- Create: `src/util/ascii_bar.rs` (заглушка с `pub fn render(_pct: f64, _width: u32) -> String { String::new() }`)
- Modify: `src/main.rs` (`mod util;`)
- Modify: `src/widgets/mod.rs` (`build_one` обрабатывает новые WidgetConfig-варианты через `unimplemented!()` — fallback ставится в T2–T7 по мере наполнения)
- Create: `benches/samples/payload-synthetic-vim-worktree.json`
- Create: `benches/samples/payload-synthetic-current-usage-total.json`
- Modify: `tests/snapshots.rs` (insta::glob pattern сужается до `payload-cchud-*.json` + `payload-posts-*.json`, чтобы synthetic-семплы не попали в default-line snapshot — детерминизм)
- Modify: `docs/DECISIONS.md` (две новые записи 2026-04-26)
- Create: `plan/phase-3/manual-test-log.md` (шаблон под Task 9)

## Goal

Подготовить фундамент Phase 3:

1. Добавить `wait-timeout` runtime-deps (нужна в T7 для CustomCommand subprocess).
2. Типизировать payload sub-types `CostInfo`, `ContextWindowInfo`, `CurrentUsage` (untagged enum для совместимости с upstream zod number|object), `Worktree`, `VimState`, `OutputStyle`. 14 виджетов из 23 читают эти поля; компилятор ловит опечатки в именах, snapshot-тесты лочат схему.
3. Добавить `vim` и `worktree` поля в `StatusPayload` envelope. Они отсутствуют во всех Phase 0 семплах — добавляем синтетические fixture-файлы (`payload-synthetic-*.json`).
4. Включить `#[serde(rename_all = "kebab-case")]` на `WidgetConfig` и retrofit Phase 2 unit-тест (`"type": "Model"` → `"type": "model"`). Паритет с upstream ccstatusline (REQ-006).
5. Объявить 22 новых варианта `WidgetConfig` (enum-only, без `Widget`-impl — impl растянуты на T2–T7). Это позволяет настройкам пользователя парситься уже в T1, и каждая последующая T-задача только меняет stub `unimplemented!()` на реальный `Box<dyn Widget>`.
6. Создать `src/util/` skeleton с тремя пустыми файлами — модули наполняются в T4 (model_context_size, ascii_bar) и T5 (duration). T1 только лочит структуру.
7. Зафиксировать два архитектурных расхождения с Phase 2 spec в `docs/DECISIONS.md`.

После Task 1 проект собирается, существующие Phase 2 тесты проходят, новые `types::payload::tests` парсят synthetic-семплы и `current_usage: 12345` (number-form). Виджетов ещё нет, рендер по-прежнему даёт "Sonnet 4.6" на default-line — Phase 2 регрессии нет.

## Inputs

- Phase 2 закрыта; ветка `master`, рабочее дерево чистое.
- `cargo build --release --locked` локально зелёный.
- `benches/samples/payload-cchud-sonnet-xlarge.json` существует и парсится.
- `tests/snapshots/*.snap` (4 файла из Phase 2) committed.
- `docs/DECISIONS.md` существует (Phase 0 артефакт).

---

- [ ] **Step 1: Добавить `wait-timeout` в `[dependencies]`**

В `Cargo.toml`, секция `[dependencies]`, после строки `terminal_size = "0.4"` (или в любом разумном месте hot-path блока), добавить:

```toml
# Phase 3 — CustomCommand subprocess timeout
wait-timeout = "0.2"
```

Edit tool:
- `old_string`:
  ```
  terminal_size = "0.4"
  supports-color = "3"
  ```
- `new_string`:
  ```
  terminal_size = "0.4"
  supports-color = "3"

  # Phase 3 — CustomCommand subprocess timeout
  wait-timeout = "0.2"
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

Verify:
```bash
grep -c '^wait-timeout' Cargo.toml
```
Expected: `1`.

- [ ] **Step 2: Регенерировать `Cargo.lock` (без --locked)**

```bash
cargo build --release
```

Expected: cargo тянет `wait-timeout 0.2.x` + transitive (`libc` уже есть, `winapi` на Windows). Build time +3–5 сек. `Cargo.lock` обновлён.

- [ ] **Step 3: Подтвердить чистоту Cargo.lock через `--locked`**

```bash
cargo build --release --locked
```

Expected: no-op, exit 0. Если падает — `Cargo.lock` не сохранился; повторить Step 2 без `--locked` и проверить `git status` что `Cargo.lock` модифицирован.

- [ ] **Step 4: Написать failing-тест для `CurrentUsage` untagged enum**

Открыть `src/types/payload.rs`, в `#[cfg(test)] mod tests` добавить:

```rust
    #[test]
    fn parses_current_usage_detailed_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {
                "current_usage": {
                    "input_tokens": 1,
                    "output_tokens": 2,
                    "cache_creation_input_tokens": 3,
                    "cache_read_input_tokens": 4
                }
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Detailed { input_tokens, output_tokens, .. } => {
                assert_eq!(input_tokens, Some(1));
                assert_eq!(output_tokens, Some(2));
            }
            CurrentUsage::Total(_) => panic!("expected Detailed"),
        }
    }

    #[test]
    fn parses_current_usage_total_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {"current_usage": 12345}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Total(v) => assert_eq!(v, 12345),
            CurrentUsage::Detailed { .. } => panic!("expected Total"),
        }
    }

    #[test]
    fn parses_vim_and_worktree_envelope_fields() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "vim": {"mode": "NORMAL"},
            "worktree": {
                "name": "wt-feature",
                "branch": "feature/x",
                "original_branch": "main"
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.vim.as_ref().and_then(|v| v.mode.as_deref()), Some("NORMAL"));
        let wt = p.worktree.expect("worktree present");
        assert_eq!(wt.name.as_deref(), Some("wt-feature"));
        assert_eq!(wt.branch.as_deref(), Some("feature/x"));
        assert_eq!(wt.original_branch.as_deref(), Some("main"));
    }

    #[test]
    fn parses_typed_cost_and_output_style() {
        const SAMPLE_TYPED: &str = include_str!(
            "../../benches/samples/payload-cchud-sonnet-xlarge.json"
        );
        let p: StatusPayload = serde_json::from_str(SAMPLE_TYPED).unwrap();
        let cost = p.cost.expect("cost present");
        assert!(cost.total_cost_usd.unwrap_or(0.0) > 0.0);
        assert!(cost.total_duration_ms.unwrap_or(0) > 0);
        let style = p.output_style.expect("output_style present");
        assert_eq!(style.name.as_deref(), Some("default"));
    }
```

- [ ] **Step 5: Запустить тесты — должны упасть с "cannot find type/field"**

```bash
cargo test --locked --lib types::payload 2>&1 | head -60
```

Expected: компиляция падает примерно так:
```
error[E0412]: cannot find type `CurrentUsage` in this scope
error[E0609]: no field `vim` on type `StatusPayload`
error[E0609]: no field `worktree` on type `StatusPayload`
```

Это и есть TDD red.

- [ ] **Step 6: Реализовать payload sub-types**

Заменить `src/types/payload.rs` целиком (сохраняя структуру: модуль-doc-comment + `#[allow(dead_code)]`):

```rust
//! Full `StatusPayload` envelope — Phase 3 типизация.
//!
//! Phase 2 walking skeleton хранил `cost`, `context_window`, `output_style`
//! как `Option<serde_json::Value>` — Phase 3 типизирует эти три поля
//! (14 виджетов их читают). `vim` и `worktree` — новые поля envelope
//! (отсутствуют во всех Phase 0 семплах; добавляются под synthetic
//! fixture-файлы). `rate_limits`, `effort`, `thinking` остаются Value
//! до Phase 6/7.
//!
//! `CurrentUsage` — untagged enum: upstream zod допускает форму
//! `current_usage?: number | { ... } | null`. В Phase 0 семплах — только
//! object | null, но защитный fallback на `Total(u64)` стоит копейки.
//!
//! Top-level envelope полностью типизирован; snapshot-тесты ловят
//! schema drift в именах/наличии полей.

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StatusPayload {
    pub session_id: String,
    pub model: ModelInfo,
    pub workspace: Workspace,

    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub fast_mode: Option<bool>,
    #[serde(default)]
    pub exceeds_200k_tokens: Option<bool>,

    // Phase 3 — typed:
    #[serde(default)]
    pub output_style: Option<OutputStyle>,
    #[serde(default)]
    pub cost: Option<CostInfo>,
    #[serde(default)]
    pub context_window: Option<ContextWindowInfo>,
    #[serde(default)]
    pub worktree: Option<Worktree>,
    #[serde(default)]
    pub vim: Option<VimState>,

    // Остаются Value — типизация позже:
    #[serde(default)]
    pub rate_limits: Option<serde_json::Value>,
    #[serde(default)]
    pub effort: Option<serde_json::Value>,
    #[serde(default)]
    pub thinking: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub current_dir: String,
    #[serde(default)]
    pub project_dir: Option<String>,
    #[serde(default)]
    pub added_dirs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostInfo {
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub total_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_api_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_lines_added: Option<u64>,
    #[serde(default)]
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextWindowInfo {
    #[serde(default)]
    pub context_window_size: Option<u64>,
    #[serde(default)]
    pub total_input_tokens: Option<u64>,
    #[serde(default)]
    pub total_output_tokens: Option<u64>,
    #[serde(default)]
    pub current_usage: Option<CurrentUsage>,
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub remaining_percentage: Option<f64>,
}

/// Upstream zod допускает `current_usage?: number | object | null`.
/// Phase 0 семплы шлют только object; Total — защита от потенциального
/// упрощения схемы Anthropic.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CurrentUsage {
    Detailed {
        #[serde(default)]
        input_tokens: Option<u64>,
        #[serde(default)]
        output_tokens: Option<u64>,
        #[serde(default)]
        cache_creation_input_tokens: Option<u64>,
        #[serde(default)]
        cache_read_input_tokens: Option<u64>,
    },
    Total(u64),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Worktree {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub original_cwd: Option<String>,
    #[serde(default)]
    pub original_branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VimState {
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputStyle {
    #[serde(default)]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    const SAMPLE: &str = include_str!("../../benches/samples/payload-cchud-sonnet-xlarge.json");

    #[test]
    fn parses_real_payload_sample() {
        let payload: StatusPayload = serde_json::from_str(SAMPLE).expect("sample must parse");
        assert_eq!(payload.model.id, "claude-sonnet-4-6");
        assert_eq!(payload.model.display_name, "Sonnet 4.6");
        assert!(!payload.session_id.is_empty());
        assert_eq!(
            payload.workspace.current_dir,
            "/Users/igor/mp/startup/cchud"
        );
        assert!(payload.cost.is_some(), "cost field must parse from sample");
        assert!(
            payload.context_window.is_some(),
            "context_window must parse"
        );
        assert!(payload.rate_limits.is_some(), "rate_limits must parse");
        assert_eq!(payload.version.as_deref(), Some("2.1.119"));
    }

    #[test]
    fn rejects_missing_required_fields() {
        let bad = r#"{"session_id":"x"}"#;
        assert!(serde_json::from_str::<StatusPayload>(bad).is_err());
    }

    #[test]
    fn parses_minimal_payload_without_heavy_fields() {
        let minimal = r#"{
            "session_id": "x",
            "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
            "workspace": {"current_dir": "/tmp"}
        }"#;
        let p: StatusPayload = serde_json::from_str(minimal).unwrap();
        assert!(p.cost.is_none());
        assert!(p.context_window.is_none());
        assert!(p.fast_mode.is_none());
        assert!(p.vim.is_none());
        assert!(p.worktree.is_none());
    }

    #[test]
    fn parses_current_usage_detailed_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {
                "current_usage": {
                    "input_tokens": 1,
                    "output_tokens": 2,
                    "cache_creation_input_tokens": 3,
                    "cache_read_input_tokens": 4
                }
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Detailed { input_tokens, output_tokens, .. } => {
                assert_eq!(input_tokens, Some(1));
                assert_eq!(output_tokens, Some(2));
            }
            CurrentUsage::Total(_) => panic!("expected Detailed"),
        }
    }

    #[test]
    fn parses_current_usage_total_form() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "context_window": {"current_usage": 12345}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let cw = p.context_window.expect("context_window present");
        let usage = cw.current_usage.expect("current_usage present");
        match usage {
            CurrentUsage::Total(v) => assert_eq!(v, 12345),
            CurrentUsage::Detailed { .. } => panic!("expected Total"),
        }
    }

    #[test]
    fn parses_vim_and_worktree_envelope_fields() {
        let json = r#"{
            "session_id": "x",
            "model": {"id": "m", "display_name": "M"},
            "workspace": {"current_dir": "/tmp"},
            "vim": {"mode": "NORMAL"},
            "worktree": {
                "name": "wt-feature",
                "branch": "feature/x",
                "original_branch": "main"
            }
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.vim.as_ref().and_then(|v| v.mode.as_deref()), Some("NORMAL"));
        let wt = p.worktree.expect("worktree present");
        assert_eq!(wt.name.as_deref(), Some("wt-feature"));
        assert_eq!(wt.branch.as_deref(), Some("feature/x"));
        assert_eq!(wt.original_branch.as_deref(), Some("main"));
    }

    #[test]
    fn parses_typed_cost_and_output_style() {
        let p: StatusPayload = serde_json::from_str(SAMPLE).unwrap();
        let cost = p.cost.expect("cost present");
        assert!(cost.total_cost_usd.unwrap_or(0.0) > 0.0);
        assert!(cost.total_duration_ms.unwrap_or(0) > 0);
        let style = p.output_style.expect("output_style present");
        assert_eq!(style.name.as_deref(), Some("default"));
    }
}
```

- [ ] **Step 7: Запустить payload-тесты — должны быть зелёные**

```bash
cargo test --locked --lib types::payload
```

Expected: 7 тестов passed (3 старых + 4 новых).

```
running 7 tests
test types::payload::tests::parses_real_payload_sample ... ok
test types::payload::tests::rejects_missing_required_fields ... ok
test types::payload::tests::parses_minimal_payload_without_heavy_fields ... ok
test types::payload::tests::parses_current_usage_detailed_form ... ok
test types::payload::tests::parses_current_usage_total_form ... ok
test types::payload::tests::parses_vim_and_worktree_envelope_fields ... ok
test types::payload::tests::parses_typed_cost_and_output_style ... ok

test result: ok. 7 passed
```

**Note**: `model.rs` тест (`payload_with_display_name`) собирает `StatusPayload` через struct-литерал — он сейчас сломается, потому что добавились поля `vim` и `worktree`. Починим в Step 8.

- [ ] **Step 8: Починить fixture-helper в `src/widgets/model.rs`**

В тесте `payload_with_display_name` добавить инициализацию двух новых полей. Edit:

- `old_string`:
  ```rust
              output_style: None,
              cost: None,
              context_window: None,
              rate_limits: None,
              effort: None,
              thinking: None,
          }
      }
  ```
- `new_string`:
  ```rust
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
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/model.rs`

Verify:
```bash
cargo test --locked --lib widgets::model
```

Expected: 2 тестов passed (renders_display_name, returns_none_for_empty_name).

- [ ] **Step 9: Создать synthetic-семпл `payload-synthetic-vim-worktree.json`**

Create file `/Users/igor/mp/startup/cchud/benches/samples/payload-synthetic-vim-worktree.json` (single line, как другие семплы):

```json
{"session_id":"synthetic-vim-wt-0001","transcript_path":"/Users/igor/.claude/projects/-tmp-synthetic/synthetic-vim-wt-0001.jsonl","cwd":"/tmp/synthetic-wt","model":{"id":"claude-sonnet-4-6","display_name":"Sonnet 4.6"},"workspace":{"current_dir":"/tmp/synthetic-wt","project_dir":"/tmp/synthetic-wt","added_dirs":[]},"version":"2.1.119","output_style":{"name":"default"},"cost":{"total_cost_usd":0.0123,"total_duration_ms":4567,"total_api_duration_ms":2222,"total_lines_added":0,"total_lines_removed":0},"context_window":{"total_input_tokens":100,"total_output_tokens":200,"context_window_size":200000,"current_usage":{"input_tokens":10,"output_tokens":20,"cache_creation_input_tokens":30,"cache_read_input_tokens":40},"used_percentage":1,"remaining_percentage":99},"exceeds_200k_tokens":false,"fast_mode":false,"vim":{"mode":"NORMAL"},"worktree":{"name":"wt-feature","path":"/tmp/synthetic-wt","branch":"feature/synthetic","original_cwd":"/tmp/synthetic-orig","original_branch":"main"}}
```

Verify:
```bash
python3 -m json.tool benches/samples/payload-synthetic-vim-worktree.json > /dev/null
```
Expected: exit 0 (валидный JSON).

- [ ] **Step 10: Создать synthetic-семпл `payload-synthetic-current-usage-total.json`**

Create file `/Users/igor/mp/startup/cchud/benches/samples/payload-synthetic-current-usage-total.json`:

```json
{"session_id":"synthetic-cu-total-0001","cwd":"/tmp","model":{"id":"claude-sonnet-4-6","display_name":"Sonnet 4.6"},"workspace":{"current_dir":"/tmp"},"version":"2.1.119","context_window":{"total_input_tokens":50,"total_output_tokens":100,"context_window_size":200000,"current_usage":12345,"used_percentage":1,"remaining_percentage":99}}
```

Verify:
```bash
python3 -m json.tool benches/samples/payload-synthetic-current-usage-total.json > /dev/null
cargo test --locked --lib types::payload::tests::parses_current_usage_total_form
```
Expected: оба exit 0.

- [ ] **Step 11: Сузить glob-pattern в `tests/snapshots.rs`**

Чтобы synthetic-семплы не попали в default-line snapshot и не создавали .snap-файлы, которые ломают регрессионный гейт Phase 2 (snapshot 1 в Task 8 будет повторять Phase 2 default-line только для НЕ-synthetic семплов), сузить glob.

Edit:
- `old_string`: `insta::glob!("../benches/samples", "payload-*.json", |path| {`
- `new_string`: `insta::glob!("../benches/samples", "payload-cchud-*.json", |path| {`
- `file_path`: `/Users/igor/mp/startup/cchud/tests/snapshots.rs`

**Rationale:** Phase 2 snapshot-pattern `payload-*.json` — он по факту ловил только 4 файла (3× `payload-cchud-*` + 1× `payload-posts-*`). Synthetic-семплы Phase 3 НЕ должны участвовать в default-line snapshot — их рендер-результат проверяется явными снапшотами Task 8 на специфичных configs, не на default-line.

Phase 0 файл `payload-posts-sonnet-fresh.json` — НЕ-cchud (другой проект `posts/`), он по сути сторонний sample. Чтобы не потерять покрытие, расширим до двух pattern'ов: pattern matches либо `payload-cchud-*` либо `payload-posts-*`.

`insta::glob!` принимает один pattern. Если хотим оба — два вызова `insta::glob!`. НО: Phase 2 фактическое поведение — все 4 файла под `payload-*.json` (без synthetic). После Step 11 потеряется `payload-posts-sonnet-fresh.json`.

**Решение:** оставляем pattern `payload-*.json`, но ВЫКЛЮЧАЕМ synthetic-семплы фильтром по имени внутри `glob!`-замыкания:

Edit (revert + extend):
- `old_string`:
  ```rust
  insta::glob!("../benches/samples", "payload-*.json", |path| {
          let payload = std::fs::read_to_string(path).unwrap();
  ```
- `new_string`:
  ```rust
  insta::glob!("../benches/samples", "payload-*.json", |path| {
          // Phase 3: synthetic-семплы рендерятся явными snapshot-сценариями
          // в Task 8 на специфичных configs, не на default-line.
          let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
          if fname.starts_with("payload-synthetic-") {
              return;
          }
          let payload = std::fs::read_to_string(path).unwrap();
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/tests/snapshots.rs`

Verify:
```bash
cargo test --locked --test snapshots
```
Expected: 4 snapshot'а (как в Phase 2), оба новых synthetic-файла отфильтрованы. Тесты зелёные.

- [ ] **Step 12: Включить kebab-case на `WidgetConfig` + retrofit Phase 2 unit-теста**

В `src/types/config.rs`:

Edit 1 (kebab-case retrofit):
- `old_string`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(tag = "type")]
  pub enum WidgetConfig {
      Model {
          #[serde(flatten, default)]
          params: ModelParams,
      },
  }
  ```
- `new_string`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(tag = "type", rename_all = "kebab-case")]
  pub enum WidgetConfig {
      // Phase 2:
      Model {
          #[serde(flatten, default)]
          params: ModelParams,
      },

      // Phase 3 — без параметров:
      Version,
      ClaudeSessionId,
      TerminalWidth,
      OutputStyle,
      VimMode,
      SessionName,
      SessionClock,
      SessionCost,
      ContextLength,
      ContextPercentage,
      ContextPercentageUsable,
      TokensInput,
      TokensOutput,
      Worktree,
      WorktreeMode,
      WorktreeName,
      WorktreeBranch,
      WorktreeOriginalBranch,

      // Phase 3 — с параметрами:
      CustomText {
          #[serde(flatten)]
          params: CustomTextParams,
      },
      CustomSymbol {
          #[serde(flatten)]
          params: CustomSymbolParams,
      },
      Link {
          #[serde(flatten)]
          params: LinkParams,
      },
      CustomCommand {
          #[serde(flatten)]
          params: CustomCommandParams,
      },
      ContextBar {
          #[serde(flatten, default)]
          params: ContextBarParams,
      },
  }
  ```

Edit 2 (добавить новые Params-структуры после `ModelParams`):
- `old_string`:
  ```rust
  /// Per-widget parameters. Phase 7 adds custom format strings, etc.
  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  pub struct ModelParams {}
  ```
- `new_string`:
  ```rust
  /// Per-widget parameters. Phase 7 adds custom format strings, etc.
  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  pub struct ModelParams {}

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CustomTextParams {
      pub text: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CustomSymbolParams {
      pub symbol: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct LinkParams {
      pub url: String,
      #[serde(default)]
      pub label: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CustomCommandParams {
      pub command: String,
      #[serde(default)]
      pub args: Vec<String>,
      #[serde(default = "default_command_timeout_ms")]
      pub timeout_ms: u64,
  }
  const fn default_command_timeout_ms() -> u64 {
      200
  }

  #[derive(Debug, Clone, Default, Serialize, Deserialize)]
  pub struct ContextBarParams {
      #[serde(default = "default_context_bar_width")]
      pub width: u32,
  }
  const fn default_context_bar_width() -> u32 {
      10
  }
  ```

Edit 3 (retrofit Phase 2 unit-теста — `"Model"` → `"model"`):
- `old_string`: `"lines": [{"widgets": [{"type": "Model"}]}],`
- `new_string`: `"lines": [{"widgets": [{"type": "model"}]}],`
- `replace_all`: `true`
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

(Этот edit зацепит все три появления литерала в `parses_minimal_cchud_block`, `rejects_unknown_widget_type` и `serde_roundtrip_preserves_shape`.)

Edit 4 (retrofit `rejects_unknown_widget_type`: `"Branch"` → `"branch"` — иначе тест перестанет проверять то, что декларирует):
- `old_string`: `"lines": [{"widgets": [{"type": "Branch"}]}]`
- `new_string`: `"lines": [{"widgets": [{"type": "branch"}]}]`
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

Также retrofit `src/config/mod.rs` тестов:

Edit 5:
- `old_string`: `"cchud":{"version":1,"lines":[{"widgets":[{"type":"Model"}]}]}`
- `new_string`: `"cchud":{"version":1,"lines":[{"widgets":[{"type":"model"}]}]}`
- `file_path`: `/Users/igor/mp/startup/cchud/src/config/mod.rs`

- [ ] **Step 13: Добавить unit-тест на парсинг новых variants**

В `src/types/config.rs`, в `mod tests`, добавить:

```rust
    #[test]
    fn parses_phase3_widget_kinds() {
        let json = r#"{
            "lines": [{"widgets": [
                {"type": "version"},
                {"type": "claude-session-id"},
                {"type": "context-bar", "width": 20},
                {"type": "custom-text", "text": "hello"},
                {"type": "custom-symbol", "symbol": "★"},
                {"type": "link", "url": "https://x.com", "label": "X"},
                {"type": "custom-command", "command": "echo", "args": ["hi"]}
            ]}]
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.lines[0].widgets.len(), 7);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Version));
        assert!(matches!(s.lines[0].widgets[1], WidgetConfig::ClaudeSessionId));
        match &s.lines[0].widgets[2] {
            WidgetConfig::ContextBar { params } => assert_eq!(params.width, 20),
            other => panic!("expected ContextBar, got {other:?}"),
        }
        match &s.lines[0].widgets[3] {
            WidgetConfig::CustomText { params } => assert_eq!(params.text, "hello"),
            other => panic!("expected CustomText, got {other:?}"),
        }
        match &s.lines[0].widgets[6] {
            WidgetConfig::CustomCommand { params } => {
                assert_eq!(params.command, "echo");
                assert_eq!(params.args, vec!["hi".to_string()]);
                assert_eq!(params.timeout_ms, 200, "default timeout_ms = 200");
            }
            other => panic!("expected CustomCommand, got {other:?}"),
        }
    }

    #[test]
    fn context_bar_default_width_is_ten() {
        let json = r#"{"lines":[{"widgets":[{"type":"context-bar"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        match &s.lines[0].widgets[0] {
            WidgetConfig::ContextBar { params } => assert_eq!(params.width, 10),
            other => panic!("expected ContextBar, got {other:?}"),
        }
    }
```

- [ ] **Step 14: Запустить config-тесты — должны быть зелёные**

```bash
cargo test --locked --lib types::config
```

Expected: 6 тестов passed (4 Phase 2 retrofit'нутых + 2 новых).

```
test types::config::tests::parses_minimal_cchud_block ... ok
test types::config::tests::defaults_fill_missing_fields ... ok
test types::config::tests::rejects_unknown_widget_type ... ok
test types::config::tests::serde_roundtrip_preserves_shape ... ok
test types::config::tests::parses_phase3_widget_kinds ... ok
test types::config::tests::context_bar_default_width_is_ten ... ok
```

Также:
```bash
cargo test --locked --lib config::
```
Expected: 4 теста зелёные (Phase 2 + retrofit-литерала).

- [ ] **Step 15: Создать `src/util/` skeleton**

Create `/Users/igor/mp/startup/cchud/src/util/mod.rs`:

```rust
//! Cross-widget helper modules used across multiple widget files.
//!
//! Naполняются по мере роста потребностей кластеров:
//! - `model_context_size` — Task 4 (ContextPercentageUsable lookup)
//! - `duration` — Task 5 (SessionClock formatter)
//! - `ascii_bar` — Task 4 (ContextBar formatter)

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod ascii_bar;
pub mod duration;
pub mod model_context_size;
```

Create `/Users/igor/mp/startup/cchud/src/util/model_context_size.rs`:

```rust
//! Model id → max context tokens lookup. Populated in Task 4.

#![allow(dead_code)]

#[must_use]
pub fn max_tokens_for(_model_id: &str) -> Option<u64> {
    None
}
```

Create `/Users/igor/mp/startup/cchud/src/util/duration.rs`:

```rust
//! Duration formatter — `HH:MM:SS` or `MM:SS`. Populated in Task 5.

#![allow(dead_code)]

#[must_use]
pub fn format_duration(_total_ms: u64) -> String {
    String::new()
}
```

Create `/Users/igor/mp/startup/cchud/src/util/ascii_bar.rs`:

```rust
//! ASCII progress bar formatter. Populated in Task 4.

#![allow(dead_code)]

#[must_use]
pub fn render(_pct: f64, _width: u32) -> String {
    String::new()
}
```

- [ ] **Step 16: Подключить `mod util;` в `src/main.rs`**

Edit:
- `old_string`:
  ```rust
  mod commands;
  mod config;
  mod render;
  mod types;
  mod widgets;
  ```
- `new_string`:
  ```rust
  mod commands;
  mod config;
  mod render;
  mod types;
  mod util;
  mod widgets;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/main.rs`

Verify:
```bash
cargo build --release --locked
```
Expected: exit 0; новые модули скомпилировались как stub'ы.

- [ ] **Step 17: Расширить `widgets::build_one` под новые WidgetConfig-варианты**

`build_one` сейчас имеет одну match-arm `Model { .. }`. После Step 12 enum получил 22 новых варианта — без обработки Rust выдаст non-exhaustive match. Решение: catch-all arm, который возвращает stub-Widget, всегда отдающий None. На T2–T7 каждая task будет заменять конкретные arm'ы реальными impl'ами, постепенно опустошая catch-all.

Edit `src/widgets/mod.rs`:
- `old_string`:
  ```rust
  pub mod model;

  use crate::types::{
      config::{Settings, WidgetConfig},
      payload::StatusPayload,
  };

  pub trait Widget: Send + Sync {
      #[allow(dead_code)]
      fn id(&self) -> &'static str;
      fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
  }

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

  #[must_use]
  pub fn build_widgets(settings: &Settings) -> Vec<Box<dyn Widget>> {
      settings
          .lines
          .first()
          .map(|line| line.widgets.iter().map(build_one).collect())
          .unwrap_or_default()
  }

  fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
      match cfg {
          WidgetConfig::Model { .. } => Box::new(model::Model),
      }
  }
  ```
- `new_string`:
  ```rust
  pub mod model;

  use crate::types::{
      config::{Settings, WidgetConfig},
      payload::StatusPayload,
  };

  pub trait Widget: Send + Sync {
      #[allow(dead_code)]
      fn id(&self) -> &'static str;
      fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
  }

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

  #[must_use]
  pub fn build_widgets(settings: &Settings) -> Vec<Box<dyn Widget>> {
      settings
          .lines
          .first()
          .map(|line| line.widgets.iter().map(build_one).collect())
          .unwrap_or_default()
  }

  /// Stub Widget — placeholder для variants, чьи impl ещё не написаны
  /// (Phase 3: вытесняется match-arms по мере роста кластеров T2–T7).
  struct Stub(&'static str);
  impl Widget for Stub {
      fn id(&self) -> &'static str {
          self.0
      }
      fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
          None
      }
  }

  fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
      match cfg {
          WidgetConfig::Model { .. } => Box::new(model::Model),

          // Phase 3 stubs — заменяются на реальные impl в T2–T7:
          WidgetConfig::CustomText { .. } => Box::new(Stub("CustomText")),
          WidgetConfig::CustomSymbol { .. } => Box::new(Stub("CustomSymbol")),
          WidgetConfig::Link { .. } => Box::new(Stub("Link")),
          WidgetConfig::Version => Box::new(Stub("Version")),
          WidgetConfig::ClaudeSessionId => Box::new(Stub("ClaudeSessionId")),
          WidgetConfig::TerminalWidth => Box::new(Stub("TerminalWidth")),
          WidgetConfig::OutputStyle => Box::new(Stub("OutputStyle")),
          WidgetConfig::VimMode => Box::new(Stub("VimMode")),
          WidgetConfig::SessionName => Box::new(Stub("SessionName")),
          WidgetConfig::SessionClock => Box::new(Stub("SessionClock")),
          WidgetConfig::SessionCost => Box::new(Stub("SessionCost")),
          WidgetConfig::ContextLength => Box::new(Stub("ContextLength")),
          WidgetConfig::ContextPercentage => Box::new(Stub("ContextPercentage")),
          WidgetConfig::ContextPercentageUsable => Box::new(Stub("ContextPercentageUsable")),
          WidgetConfig::ContextBar { .. } => Box::new(Stub("ContextBar")),
          WidgetConfig::TokensInput => Box::new(Stub("TokensInput")),
          WidgetConfig::TokensOutput => Box::new(Stub("TokensOutput")),
          WidgetConfig::Worktree => Box::new(Stub("Worktree")),
          WidgetConfig::WorktreeMode => Box::new(Stub("WorktreeMode")),
          WidgetConfig::WorktreeName => Box::new(Stub("WorktreeName")),
          WidgetConfig::WorktreeBranch => Box::new(Stub("WorktreeBranch")),
          WidgetConfig::WorktreeOriginalBranch => Box::new(Stub("WorktreeOriginalBranch")),
          WidgetConfig::CustomCommand { .. } => Box::new(Stub("CustomCommand")),
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

**Note:** `Stub` рендерит `None`, значит на default-line cchud по-прежнему печатает `Sonnet 4.6` — Phase 2 регрессии нет.

- [ ] **Step 18: Standard gate (build + test + clippy + fmt)**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все 4 команды exit 0.

Тесты теперь:
- types::payload (7)
- types::config (6)
- config:: (4)
- render (3)
- widgets::model (2)
- install integration (5)
- snapshots integration (2: glob + AC-007)
- util::* (0 — stubs пока без тестов; добавятся в T4/T5)

**ИТОГО ≥29 тестов.**

Если clippy ругается на `dead_code` или `unused_imports` в `Stub` — `#[allow(dead_code)]` уже на `Widget::id`. Если падает на `enum WidgetConfig` без `#[non_exhaustive]` — игнорировать (внутренний enum).

Если clippy жалуется на `unused_imports` в `src/util/*.rs` — все три файла используют `#[allow(dead_code)]`, не должно. Если жалуется на `pedantic::missing_const_for_fn` — добавить `const` в stub-функции.

- [ ] **Step 19: Записать 2 решения в `docs/DECISIONS.md`**

Read `docs/DECISIONS.md` и определить формат записей (Phase 0 артефакт). Скорее всего: `## YYYY-MM-DD — title` + body.

Добавить две записи в конец файла:

```markdown
## 2026-04-26 — WidgetConfig serde tag = kebab-case

`#[serde(rename_all = "kebab-case")]` на `WidgetConfig`. Phase 2 ожидал
`"type": "Model"` (PascalCase, default serde). Phase 3 переключает на
kebab-case для паритета с upstream ccstatusline (REQ-006 — будущая
команда `cchud import` мигрирует существующие пользовательские
`~/.claude/settings.json` со статуслайном ccstatusline). Phase 2 unit-тест
`parses_minimal_cchud_block` ретрофитнут (`"Model"` → `"model"`); CI
зелёный. Пользовательские конфиги Phase 2 alpha — не существуют, потому
breaking change безопасен.

## 2026-04-26 — Phase 3 ускоряет типизацию cost/context_window

Phase 2 spec обещал "Phase 6 типизирует cost, context_window,
rate_limits". Phase 3 типизирует cost и context_window раньше — 14 из
23 виджетов Phase 3 их читают; держать их `Option<serde_json::Value>`
означало бы `Value::pointer` everywhere в widget-коде, без compile-time
защиты от опечаток в именах полей. `rate_limits` остаётся `Value` до
Phase 6 (Phase 3 не имеет виджета, который её читает). Также типизирован
`output_style` (виджет OutputStyle читает только `name`). Структуры:
`CostInfo`, `ContextWindowInfo`, `CurrentUsage` (untagged enum
number|object), `Worktree`, `VimState`, `OutputStyle` — в
`src/types/payload.rs`.
```

(Если в `DECISIONS.md` другой формат — следуй существующему стилю; основное требование: дата 2026-04-26 + два разных заголовка.)

Verify:
```bash
grep -c '^## 2026-04-26' docs/DECISIONS.md
```
Expected: ≥ 2 (могут быть и другие записи 2026-04-26).

- [ ] **Step 20: Создать `plan/phase-3/manual-test-log.md` (шаблон)**

Create file `/Users/igor/mp/startup/cchud/plan/phase-3/manual-test-log.md`:

```markdown
# Phase 3 — Manual Real-CC Test Log

> Шаблон под Task 9. Заполняется при ручной верификации в реальном Claude Code.

## Тестовая сессия

- **Дата:** TBD (проставить при выполнении)
- **Версия cchud:** 0.1.0-alpha (`cchud --version`)
- **Версия Claude Code:** TBD (`claude --version`)
- **OS / arch:** macOS 24.x / arm64 (M-серия)

## Сценарий

1. `cchud install` — wires в `~/.claude/settings.json`.
2. `~/.config/cchud/settings.json` — конфиг с включёнными:
   - `model`
   - `worktree-name` + `worktree-branch` + `worktree-mode`
   - `vim-mode`
   - `session-cost` + `session-clock`
   - `context-percentage` + `context-bar` (width 10)
   - `version` + `claude-session-id`
   - `custom-text` "demo" + `custom-command` `echo phase-3`
3. Открыть Claude Code в worktree (`git worktree add ../cchud-wt feature/demo`).
4. Включить vim mode.
5. Поработать ≥ 5 минут (любые задачи: чтение/правки файлов, run tests).

## Наблюдения

- [ ] Statusline рендерится без артефактов
- [ ] Worktree-name и Worktree-branch показывают правильные значения
- [ ] Vim mode переключается NORMAL ⟷ INSERT при `i` / `Esc`
- [ ] SessionCost растёт по мере работы
- [ ] SessionClock тикает (HH:MM:SS / MM:SS форматы)
- [ ] ContextPercentage обновляется
- [ ] ContextBar заполняется пропорционально
- [ ] CustomCommand "echo phase-3" печатает "phase-3"
- [ ] Лагов на UI Claude Code нет
- [ ] Stderr CC чист от `cchud:` warning'ов

## Метрики

- p95 cchud render time (из `hyperfine` Task 8): TBD ms
- Binary size (`ls -lh target/release/cchud`): TBD MB

## Замечания / отклонения от ожиданий

(Сюда — любые сюрпризы: payload поля в реальном CC отличаются от наших synthetic, отсутствуют ожидаемые виджеты, и т.п. Дата + описание.)

## Sign-off

- [ ] Все 10 чекбоксов выше отмечены
- [ ] manual-test-log.md committed в `plan/phase-3/`
```

- [ ] **Step 21: Verification — task-specific gate**

```bash
grep -c '^wait-timeout' Cargo.toml
grep -c 'pub struct CostInfo' src/types/payload.rs
grep -c 'pub struct ContextWindowInfo' src/types/payload.rs
grep -c 'pub enum CurrentUsage' src/types/payload.rs
grep -c 'pub struct Worktree' src/types/payload.rs
grep -c 'pub struct VimState' src/types/payload.rs
grep -c 'rename_all = "kebab-case"' src/types/config.rs
grep -c 'WidgetConfig::CustomCommand' src/widgets/mod.rs
ls src/util/ | wc -l
ls benches/samples/payload-synthetic-*.json | wc -l
grep -c '^## 2026-04-26' docs/DECISIONS.md
test -f plan/phase-3/manual-test-log.md && echo "manual-test-log: ok"
```

Expected output:
```
1
1
1
1
1
1
1
1
       4   (mod.rs + 3 stub'а)
       2
2 или больше
manual-test-log: ok
```

- [ ] **Step 22: Commit**

```bash
git add Cargo.toml Cargo.lock src/types/payload.rs src/types/config.rs \
        src/config/mod.rs src/widgets/mod.rs src/widgets/model.rs \
        src/util/ src/main.rs \
        benches/samples/payload-synthetic-vim-worktree.json \
        benches/samples/payload-synthetic-current-usage-total.json \
        tests/snapshots.rs docs/DECISIONS.md plan/phase-3/manual-test-log.md
git commit -m "feat(phase-3): T1 setup — typed sub-payloads, kebab-case, util skeleton

- Cargo.toml: + wait-timeout 0.2 (runtime, used in T7 CustomCommand)
- types::payload: typed CostInfo, ContextWindowInfo, CurrentUsage
  (untagged enum number|object), Worktree, VimState, OutputStyle.
  StatusPayload gains vim + worktree fields.
- types::config: WidgetConfig serde rename_all = kebab-case;
  22 new variants (Phase 3 widgets); CustomTextParams, CustomSymbolParams,
  LinkParams, CustomCommandParams (default timeout_ms=200), ContextBarParams
  (default width=10).
- widgets::build_one: stub-Widget for new variants (impl follows in T2–T7).
- src/util/: skeleton (model_context_size, duration, ascii_bar — populated
  in T4 + T5).
- benches/samples/payload-synthetic-vim-worktree.json
- benches/samples/payload-synthetic-current-usage-total.json
- tests/snapshots.rs: filter out synthetic-* from default-line glob.
- docs/DECISIONS.md: 2 entries (kebab-case retrofit, Phase 3 cost/cw typing).
- plan/phase-3/manual-test-log.md: template for T9.

Phase 2 unit-tests retrofitted: \"type\": \"Model\" → \"type\": \"model\".
Phase 2 snapshots remain green (default-line == \"Sonnet 4.6\").

Task 1/9 of Phase 3.
"
```

Verify: `git log --oneline | head -1` — `feat(phase-3): T1 setup ...`.

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `Cargo.toml` содержит `wait-timeout = "0.2"` в `[dependencies]`
- [ ] `src/types/payload.rs` содержит `CostInfo`, `ContextWindowInfo`, `CurrentUsage` (untagged), `Worktree`, `VimState`, `OutputStyle`
- [ ] `StatusPayload` envelope: `cost`, `context_window`, `output_style` — typed; `vim`, `worktree` — новые поля
- [ ] `src/types/config.rs` использует `#[serde(rename_all = "kebab-case")]` и содержит 23 варианта `WidgetConfig` (1 Phase 2 + 22 Phase 3)
- [ ] `CustomTextParams`, `CustomSymbolParams`, `LinkParams`, `CustomCommandParams` (timeout 200ms default), `ContextBarParams` (width 10 default) объявлены
- [ ] `src/util/mod.rs` + 3 stub-файла созданы; `mod util;` в `main.rs`
- [ ] `widgets::build_one` обрабатывает все 23 варианта (Stub для T2–T7, Model real)
- [ ] `benches/samples/payload-synthetic-vim-worktree.json` и `payload-synthetic-current-usage-total.json` созданы и валидны
- [ ] `tests/snapshots.rs` фильтрует synthetic-семплы из default-line glob
- [ ] Все Phase 2 тесты проходят после kebab-case retrofit
- [ ] 4+ новых unit-теста в `types::payload` (Detailed/Total/vim+worktree/typed-cost-output_style)
- [ ] 2+ новых unit-теста в `types::config` (parses_phase3_widget_kinds + context_bar_default_width_is_ten)
- [ ] `docs/DECISIONS.md` содержит 2 новые записи 2026-04-26
- [ ] `plan/phase-3/manual-test-log.md` шаблон создан
- [ ] Один commit `feat(phase-3): T1 setup ...`

## Files touched

- `Cargo.toml` (modified)
- `Cargo.lock` (auto-regenerated)
- `src/types/payload.rs` (modified, ~150 lines added)
- `src/types/config.rs` (modified, ~60 lines added)
- `src/config/mod.rs` (modified, 1 retrofit)
- `src/widgets/mod.rs` (modified, +Stub + 22 match-arms)
- `src/widgets/model.rs` (modified, fixture-helper)
- `src/main.rs` (modified, +mod util)
- `src/util/mod.rs` (created)
- `src/util/model_context_size.rs` (created, stub)
- `src/util/duration.rs` (created, stub)
- `src/util/ascii_bar.rs` (created, stub)
- `benches/samples/payload-synthetic-vim-worktree.json` (created)
- `benches/samples/payload-synthetic-current-usage-total.json` (created)
- `tests/snapshots.rs` (modified, glob-filter)
- `docs/DECISIONS.md` (modified, +2 entries)
- `plan/phase-3/manual-test-log.md` (created)

## Risks & rollback

- **`wait-timeout 0.2` ломает MSRV 1.85**: маловероятно (crate последний раз обновлялась 2023, MSRV ≤ 1.65). Если падает — посмотреть `cargo metadata --format-version 1 | jq '.packages[] | select(.name=="wait-timeout") | .rust_version'` и при необходимости поднять `rust-version` в `Cargo.toml`.
- **`#[serde(untagged)]` на `CurrentUsage` неверно сериализует обратно**: мы используем только `Deserialize`, не `Serialize`, для payload. Нет риска.
- **Phase 2 пользователи имеют `"type": "Model"` в `~/.claude/settings.json`**: Phase 2 — alpha, пользователей не существует. Если бы существовали — `cchud import` (Phase 9) сделал бы миграцию; в Phase 3 — breaking, но никого не задевает.
- **`#[serde(rename_all = "kebab-case")]` ломает Phase 2 snapshot'ы**: snapshot'ы лочат stdout cchud (рендеринг), не парсинг configs. Phase 2 тесты используют `Settings::default_line()` (in-memory `WidgetConfig::Model`), не JSON-парсинг. Snapshot'ы остаются зелёными.
- **`mod util;` ломает clippy `pedantic::module_inception`**: модуль называется `util`, файлы внутри `model_context_size`/`duration`/`ascii_bar` — нет inception. ОК.
- **Synthetic-семплы попадают в default-line snapshot и ломают Task 8**: явная фильтрация в Step 11 решает.
- **Rollback**: `git revert HEAD` — снимает все изменения T1; ничего не зависит.
