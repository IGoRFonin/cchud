# Task 9 — Full envelope payload + insta snapshots

**Files:**
- Modify: `src/types/payload.rs` (расширить envelope до полных 15 полей, тяжёлые → `Option<serde_json::Value>`)
- Modify: `tests/snapshots.rs` (заменить ручной ассерт на `insta::assert_snapshot!` через `insta::glob`)
- Create: `tests/snapshots/` (директория с `.snap` файлами после `cargo insta review`)

## Goal

Расширить `StatusPayload` до полного envelope (15 полей по семплам Phase 0): `cost`, `context_window`, `rate_limits`, `effort`, `thinking`, `output_style`, `version`, `fast_mode`, `exceeds_200k_tokens` — как `Option<serde_json::Value>` (полная типизация — Phase 6). Заменить ручную проверку в `tests/snapshots.rs` на `insta::glob` snapshots — каждый семпл получает свой `.snap` файл, любое изменение output ловится в ревью. `cargo insta review` подтверждает первичные snapshot'ы и комитит их.

## Inputs

- Tasks 1–8 закрыты: walking skeleton + config + install + 5 install-тестов.
- `insta = { version = "1", features = ["json"] }` в `[dev-dependencies]` (Phase 1).
- `cargo install cargo-insta` локально установлен (если нет — `cargo install cargo-insta`).
- `benches/samples/payload-cchud-*.json`, `benches/samples/payload-posts-*.json` существуют (Phase 0, ≥4 файла).

---

- [ ] **Step 1: Расширить `src/types/payload.rs` до полного envelope**

Полностью заменить содержимое (кроме `#[cfg(test)]` блока):

```rust
//! Full StatusPayload envelope — Phase 2 Task 9 expansion.
//!
//! All 15 envelope fields from Claude Code statusLine payload (Phase 0
//! research). Heavy nested structures (`cost`, `context_window`,
//! `rate_limits`, `effort`, `thinking`, `output_style`) are kept as
//! `Option<serde_json::Value>` until the phase that consumes them:
//! - Phase 6: cost, context_window, rate_limits → typed
//! - Phase 7: effort, thinking, output_style → typed (or stay Value)
//!
//! Top-level envelope is fully typed so snapshot tests catch any
//! Anthropic schema drift in field names/presence.

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
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub fast_mode: Option<bool>,
    #[serde(default)]
    pub exceeds_200k_tokens: Option<bool>,

    // Heavy sub-structures — kept as Value, typed in Phase 6/7.
    #[serde(default)]
    pub output_style: Option<serde_json::Value>,
    #[serde(default)]
    pub cost: Option<serde_json::Value>,
    #[serde(default)]
    pub context_window: Option<serde_json::Value>,
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]
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
        // Heavy sub-structures should now parse into Some(Value)
        assert!(payload.cost.is_some(), "cost field must parse from sample");
        assert!(payload.context_window.is_some(), "context_window must parse");
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
        // Минимум, который Anthropic мог бы прислать в worst-case
        let minimal = r#"{
            "session_id": "x",
            "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
            "workspace": {"current_dir": "/tmp"}
        }"#;
        let p: StatusPayload = serde_json::from_str(minimal).unwrap();
        assert!(p.cost.is_none());
        assert!(p.context_window.is_none());
        assert!(p.fast_mode.is_none());
    }
}
```

**Note:** Если какое-то поле отсутствует в семпле — это ОК, `Option<...>` + `#[serde(default)]` обработает. Если в семпле есть НЕИЗВЕСТНОЕ поле, которого нет в struct — serde игнорирует unknown fields по умолчанию. Hard-fail mode (`#[serde(deny_unknown_fields)]`) пока НЕ ставим — Anthropic может добавить поле без предупреждения, не хочется упасть.

- [ ] **Step 2: Заменить `tests/snapshots.rs` на insta::glob**

Полностью заменить:

```rust
//! Insta snapshot tests over Phase 0 payload samples.
//!
//! Each sample → its own .snap file. Any change in rendered output
//! (display_name format, separator, widget order, etc.) requires
//! `cargo insta review` to acknowledge. See `tests/snapshots/`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::error::Error;

#[test]
fn render_default_line_for_phase0_samples() {
    insta::glob!("../benches/samples", "payload-*.json", |path| {
        let payload = std::fs::read_to_string(path).unwrap();
        let output = Command::cargo_bin("cchud")
            .unwrap()
            .write_stdin(payload)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "non-zero exit on {path:?}: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        // Trim trailing newline from println! for stable snapshot
        let stdout = stdout.trim_end();
        insta::assert_snapshot!(stdout);
    });
}

#[test]
fn graceful_fallback_on_broken_json() -> Result<(), Box<dyn Error>> {
    let output = Command::cargo_bin("cchud")?
        .write_stdin("not json {{")
        .output()?;
    assert!(output.status.success(), "exit must be 0 on broken JSON (AC-007)");
    assert!(
        output.stdout.is_empty(),
        "stdout must be empty on broken JSON, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.starts_with("cchud:"), "stderr must start with 'cchud:', got {stderr:?}");
    Ok(())
}
```

**Note про `insta::glob!`:** path первым аргументом — относительный к директории, в которой лежит test-файл. `../benches/samples` относительно `tests/snapshots.rs` указывает на `/Users/igor/mp/startup/cchud/benches/samples` — корректно.

**Note про `trim_end`:** `println!` добавляет `\n` к выводу. Snapshot без `\n` чище и стабильнее cross-platform (Windows `\r\n`).

- [ ] **Step 3: Запустить тесты — первичная генерация `.snap.new`**

```bash
cargo test --locked --test snapshots
```

Expected output (примерно):
```
running 2 tests
test graceful_fallback_on_broken_json ... ok

stored new snapshot tests/snapshots/snapshots__render_default_line_for_phase0_samples@payload-cchud-opus-xlarge.json.snap.new
stored new snapshot tests/snapshots/snapshots__render_default_line_for_phase0_samples@payload-cchud-sonnet-xlarge.json.snap.new
... (по одному .snap.new на каждый семпл)

test render_default_line_for_phase0_samples ... FAILED  (snapshots not yet acknowledged)
```

Это нормально на первом запуске — insta создаёт `.snap.new` файлы и помечает тест как failed до ревью. На втором запуске после `cargo insta accept` они уже `.snap` и тест пройдёт.

- [ ] **Step 4: Ревью snapshot'ов через `cargo insta review`**

```bash
cargo insta review
```

Expected: интерактивное ТУИ. Для каждого `.snap.new` показывает diff (left = empty, right = новый snapshot — например `Sonnet 4.6` или `Opus 4.6`). Подтвердить каждый клавишей `a` (accept) или `A` (accept all).

После accept-all `.snap.new` файлы переименовываются в `.snap`. Структура:
```
tests/snapshots/
├── snapshots__render_default_line_for_phase0_samples@payload-cchud-opus-xlarge.json.snap
├── snapshots__render_default_line_for_phase0_samples@payload-cchud-sonnet-xlarge.json.snap
├── snapshots__render_default_line_for_phase0_samples@payload-cchud-sonnet-xlarge2.json.snap
└── snapshots__render_default_line_for_phase0_samples@payload-posts-sonnet-fresh.json.snap
```

(≥4 файла, по числу payload-*.json в `benches/samples/`.)

**Альтернатива без TUI** (для CI / автоматизации):
```bash
INSTA_UPDATE=always cargo test --locked --test snapshots
```
— автоматически принимает все snapshot'ы. ОК для первичной генерации, но НЕ запускать на CI или после ручных правок (заглушит реальные расхождения).

- [ ] **Step 5: Verify snapshots committed**

```bash
ls tests/snapshots/
cat tests/snapshots/*.snap | head -30
```

Expected: 4+ файла `.snap` (без `.new`). Каждый содержит:
```
---
source: tests/snapshots.rs
expression: stdout
---
Sonnet 4.6
```
(или `Opus 4.6`, в зависимости от семпла.)

- [ ] **Step 6: Запустить тесты повторно — теперь должны быть зелёные**

```bash
cargo test --locked --test snapshots
```

Expected:
```
running 2 tests
test graceful_fallback_on_broken_json ... ok
test render_default_line_for_phase0_samples ... ok

test result: ok. 2 passed; 0 failed
```

(`render_default_line_for_phase0_samples` — 1 тест, обходит 4+ файла внутри.)

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Тесты теперь:
- payload tests (3, обновлены)
- config tests (4)
- render tests (3)
- model tests (2)
- install tests (5)
- snapshots tests (2 — один обходит 4+ семпла)

ИТОГО ≥19 тестов в проекте.

- [ ] **Step 8: Verification — task-specific gate**

```bash
grep -c 'pub cost: Option<serde_json::Value>' src/types/payload.rs
grep -c 'pub rate_limits: Option<serde_json::Value>' src/types/payload.rs
grep -c 'insta::glob' tests/snapshots.rs
grep -c 'insta::assert_snapshot' tests/snapshots.rs
ls tests/snapshots/ | grep -c '\.snap$'
ls tests/snapshots/ | grep -c '\.snap\.new$'
```

Expected output:
```
1
1
1
1
4         (или больше, по числу семплов)
0         (никаких неподтверждённых .snap.new)
```

- [ ] **Step 9: Commit**

```bash
git add src/types/payload.rs tests/snapshots.rs tests/snapshots/
git commit -m "feat(phase-2): full envelope + insta::glob snapshots

StatusPayload now covers all 15 envelope fields from Phase 0
samples. Heavy sub-structures (cost, context_window, rate_limits,
effort, thinking, output_style) kept as Option<serde_json::Value>
until consumed in Phase 6/7. Top-level field names are typed —
catches Anthropic schema drift in tests.

tests/snapshots.rs uses insta::glob over benches/samples — one
.snap per payload, locked via cargo insta review. AC-007 graceful
fallback test retained.

cargo install cargo-insta required to review/accept snapshots.

Task 9/10 of Phase 2.
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

- [ ] `src/types/payload.rs` содержит все 15 envelope-полей
- [ ] Тяжёлые поля (`cost`, `context_window`, `rate_limits`, `effort`, `thinking`, `output_style`) — `Option<serde_json::Value>`
- [ ] Лёгкие поля (`session_id`, `version`, `fast_mode`, `exceeds_200k_tokens`, `transcript_path`, `cwd`) — типизированы
- [ ] `parses_minimal_payload_without_heavy_fields` тест проверяет robust-парсинг минимума
- [ ] `tests/snapshots.rs` использует `insta::glob!` с pattern `payload-*.json`
- [ ] `tests/snapshots/` содержит ≥4 `.snap` файла, ноль `.snap.new`
- [ ] AC-007 graceful test остаётся зелёным
- [ ] `cargo test --locked --test snapshots` зелёный после `cargo insta review`
- [ ] Один коммит `feat(phase-2): full envelope + insta::glob snapshots`

## Files touched

- `src/types/payload.rs` (modified — расширение envelope)
- `tests/snapshots.rs` (modified — insta::glob)
- `tests/snapshots/*.snap` (created — 4+ файла после `cargo insta review`)

## Risks & rollback

- **`cargo install cargo-insta` не установлен**: TUI ревью невозможен. Fallback: `INSTA_UPDATE=always cargo test --locked --test snapshots` (Step 4 alt). После этого вручную проверить `tests/snapshots/*.snap` — каждый должен содержать `Sonnet 4.6` или аналогичное.
- **Сэмпл содержит поле не в нашем struct**: serde игнорирует unknown fields. Если хотим catch'ить дрейф — добавить `#[serde(deny_unknown_fields)]` глобально. Phase 2 НЕ ставит — Anthropic может добавить поле, мы не хотим упасть.
- **Сэмпл НЕ содержит поля из нашего struct**: `Option<...>` + `#[serde(default)]` справится — поле станет `None`.
- **`include_str!` в test'е не находит payload**: путь `../../benches/samples/...` относительно `src/types/payload.rs` — должен работать. Альтернатива — `std::fs::read_to_string` с CARGO_MANIFEST_DIR (но это test-only механизм, не рантайм).
- **CI Windows: `\r\n` в snapshot'ах**: `trim_end` убирает trailing newline, но если внутри payload есть `\r\n` — могут быть расхождения. Если CI красный из-за этого — `stdout.replace("\r\n", "\n")` перед `assert_snapshot!`.
- **`.snap` файлы попадают в gitignore**: проверить что `tests/snapshots/` НЕ matched в `.gitignore` (Phase 1 .gitignore не упоминает snapshots — должно быть ОК).
- **Rollback**: `git revert HEAD` — снимает payload expansion + snapshot setup; `tests/snapshots/` директория удаляется.
