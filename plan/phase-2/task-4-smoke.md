# Task 4 — Walking skeleton: Plain renderer + Model widget + main pipeline

**Files:**
- Create: `src/render/mod.rs`
- Create: `src/widgets/model.rs`
- Modify: `src/widgets/mod.rs` (зарегистрировать Model в `build_widgets`, добавить `pub mod model;`)
- Modify: `src/main.rs` (заменить skeleton-логику на real render pipeline + `mod render;`)
- Modify: `tests/snapshots.rs` (изменить ассерт со skeleton-маркера на реальный display_name)

## Goal

Первый end-to-end: `echo $payload | cchud` → `Sonnet 4.6`. Соединяет всё, что было определено в Tasks 2–3, реализует `Plain` renderer и `Model` widget, переписывает `main.rs` под render pipeline. Hardcoded default-line (нет ещё `config::load`, это Task 6). Phase 1 integration-тест переписывается под новый output-формат.

## Inputs

- Tasks 1–3 закрыты: `serial_test`+`tempfile` deps, `types::payload::StatusPayload`, `widgets::{Widget, RenderContext, build_widgets}`.
- `src/main.rs` пока имеет skeleton-логику Phase 1 (`println!("cchud (skeleton) | input bytes: {len}")`).
- `benches/samples/payload-cchud-sonnet-xlarge.json` парсится в `StatusPayload` (проверено в Task 2).

---

- [ ] **Step 1: Создать `src/render/mod.rs`**

```rust
//! Renderer trait — joins widget segments into a single line.
//! Phase 2 ships only `Plain`. `Powerline` lands in Phase 4.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub trait Renderer {
    fn render(&self, segments: &[String]) -> String;
}

pub struct Plain {
    pub separator: String,
}

impl Renderer for Plain {
    fn render(&self, segments: &[String]) -> String {
        segments
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(&self.separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_with_separator() {
        let r = Plain { separator: " | ".into() };
        let out = r.render(&["a".into(), "b".into(), "c".into()]);
        assert_eq!(out, "a | b | c");
    }

    #[test]
    fn filters_empty_segments() {
        let r = Plain { separator: " | ".into() };
        let out = r.render(&["a".into(), String::new(), "b".into()]);
        assert_eq!(out, "a | b");
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let r = Plain { separator: " | ".into() };
        assert_eq!(r.render(&[]), "");
    }
}
```

- [ ] **Step 2: Создать `src/widgets/model.rs`**

```rust
//! Model widget — renders the model display name from the payload.

use crate::widgets::{RenderContext, Widget};

pub struct Model;

impl Widget for Model {
    fn id(&self) -> &'static str {
        "Model"
    }

    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = &ctx.payload.model.display_name;
        if name.is_empty() {
            None
        } else {
            Some(name.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_display_name(name: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: name.into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
        }
    }

    #[test]
    fn renders_display_name() {
        let p = payload_with_display_name("Sonnet 4.6");
        let ctx = RenderContext::new(&p);
        assert_eq!(Model.render(&ctx), Some("Sonnet 4.6".into()));
    }

    #[test]
    fn returns_none_for_empty_name() {
        let p = payload_with_display_name("");
        let ctx = RenderContext::new(&p);
        assert_eq!(Model.render(&ctx), None);
    }
}
```

- [ ] **Step 3: Обновить `src/widgets/mod.rs` — зарегистрировать Model**

В существующем `src/widgets/mod.rs`:

1. Добавить под `use crate::types::payload::StatusPayload;`:
   ```rust
   pub mod model;
   ```

2. Заменить тело `build_widgets`:
   ```rust
   #[must_use]
   pub fn build_widgets() -> Vec<Box<dyn Widget>> {
       vec![Box::new(model::Model)]
   }
   ```
   (Hardcoded default-line — соответствует решению A из спеки. Config-driven вариант появляется в Task 6.)

3. Снять `#[allow(dead_code)]` с `build_widgets` и `RenderContext::new` если они были добавлены в Task 3 (теперь оба используются из `main.rs`).

- [ ] **Step 4: Полностью переписать `src/main.rs` под render pipeline**

```rust
//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 2 walking skeleton. Reads JSON payload from stdin, renders
//! configured widgets joined by Plain renderer, prints to stdout.
//! Config loading lands in Task 6; install command in Task 7.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod render;
mod types;
mod widgets;

use std::process::ExitCode;

use crate::render::{Plain, Renderer};
use crate::types::payload::StatusPayload;
use crate::widgets::{build_widgets, RenderContext};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => render_pipeline(),
    }
}

fn render_pipeline() -> ExitCode {
    let payload: StatusPayload = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: invalid payload: {e}");
            return ExitCode::SUCCESS; // AC-007: graceful, ничего в stdout
        }
    };
    let ctx = RenderContext::new(&payload);
    let widgets = build_widgets();
    let segments: Vec<String> = widgets.iter().filter_map(|w| w.render(&ctx)).collect();
    let renderer = Plain { separator: " | ".into() };
    println!("{}", renderer.render(&segments));
    ExitCode::SUCCESS
}
```

**Note:** `commands` модуль и `install` команда не существуют — добавятся в Task 7.

- [ ] **Step 5: Обновить `tests/snapshots.rs` — ассерт на новый output**

Phase 1 тест ассертит `stdout.contains("cchud (skeleton)")` — это сломается после Step 4. Заменить полностью на:

```rust
//! Integration tests: render pipeline.
//!
//! Phase 2 — verifies binary parses stdin payloads from Phase 0 fixtures
//! and produces stdout containing the model display name. Insta-based
//! snapshot tests with full output capture land in Task 9.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use std::{error::Error, fs};

#[test]
fn renders_model_for_each_phase0_sample() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    for entry in fs::read_dir("benches/samples")? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let payload = fs::read_to_string(&path)?;
            // Извлекаем ожидаемый display_name из JSON напрямую,
            // чтобы тест работал с любым семплом.
            let parsed: serde_json::Value = serde_json::from_str(&payload)?;
            let expected = parsed
                .get("model")
                .and_then(|m| m.get("display_name"))
                .and_then(|v| v.as_str())
                .ok_or("sample missing model.display_name")?;

            let output = Command::cargo_bin("cchud")?
                .write_stdin(payload.clone())
                .output()?;

            assert!(
                output.status.success(),
                "non-zero exit on {path:?}: stderr={}",
                String::from_utf8_lossy(&output.stderr),
            );
            let stdout = String::from_utf8(output.stdout)?;
            assert!(
                stdout.trim() == expected,
                "stdout {stdout:?} does not equal expected display_name {expected:?} for {path:?}",
            );

            count += 1;
        }
    }
    assert!(count >= 4, "expected ≥4 payload samples, found {count}");
    Ok(())
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

**Note:** `graceful_fallback_on_broken_json` — это полное покрытие AC-007. Дублироваться в Task 9 не будет.

- [ ] **Step 6: Smoke test вручную**

```bash
cargo build --release --locked
cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud
```

Expected output:
```
Sonnet 4.6
```

(И ничего больше; ровно одна строка.)

Если выводит пустую строку — проверить что `display_name` в семпле не пустой. Если выводит `cchud: invalid payload: ...` — schema mismatch (см. Task 2 fallback).

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Тесты:
- `tests/snapshots.rs::renders_model_for_each_phase0_sample` — зелёный (≥4 семпла)
- `tests/snapshots.rs::graceful_fallback_on_broken_json` — зелёный (AC-007)
- `src/render/mod.rs::tests::*` — 3 unit-теста зелёные
- `src/widgets/model.rs::tests::*` — 2 unit-теста зелёные
- `src/types/payload.rs::tests::*` — 2 unit-теста зелёные (Task 2)

Если clippy ругается на `clippy::needless_pass_by_value` или `clippy::redundant_clone` в `widgets/model.rs` (`name.clone()`) — это false positive (нам нужен owned String), добавить аннотацию `#[allow(clippy::redundant_clone)]` локально или принять warning через rationale-комментарий.

- [ ] **Step 8: Verification — task-specific gate**

```bash
test -f src/render/mod.rs && echo "render/mod.rs ok"
test -f src/widgets/model.rs && echo "widgets/model.rs ok"
grep -q 'pub mod model' src/widgets/mod.rs && echo "model registered ok"
grep -q 'render_pipeline' src/main.rs && echo "main rewritten ok"
echo '{"session_id":"x","model":{"id":"y","display_name":"TestModel"},"workspace":{"current_dir":"/"}}' | ./target/release/cchud
```

Expected output:
```
render/mod.rs ok
widgets/model.rs ok
model registered ok
main rewritten ok
TestModel
```

- [ ] **Step 9: Commit**

```bash
git add src/render/mod.rs src/widgets/model.rs src/widgets/mod.rs src/main.rs tests/snapshots.rs
git commit -m "feat(phase-2): walking skeleton — Plain + Model + render pipeline

First end-to-end: stdin → parse → Model widget → Plain renderer →
stdout. Hardcoded default-line (config layer in Task 6).

- src/render/mod.rs: Renderer trait + Plain (3 unit tests)
- src/widgets/model.rs: Model widget (2 unit tests)
- src/widgets/mod.rs: register Model in build_widgets
- src/main.rs: rewrite — args dispatch (--version) + render pipeline
- tests/snapshots.rs: assert display_name appears + AC-007 graceful

Smoke: 'echo \$payload | cchud' prints \"Sonnet 4.6\". Real
benchmarks vs ccstatusline in Task 10.

Task 4/10 of Phase 2.
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

- [ ] `src/render/mod.rs` существует, `Plain::render` фильтрует empty + join'ит через separator
- [ ] `src/widgets/model.rs` существует, `Model` impl Widget, возвращает display_name или None
- [ ] `src/widgets/mod.rs::build_widgets` возвращает `vec![Box::new(Model)]`
- [ ] `src/main.rs` имеет `render_pipeline()`, обрабатывает `--version`, AC-007 graceful
- [ ] `tests/snapshots.rs` ассертит точное равенство stdout = display_name
- [ ] AC-007 тест присутствует и зелёный
- [ ] `echo '{...sample...}' | ./target/release/cchud` печатает "Sonnet 4.6"
- [ ] `cargo test --locked` зелёный, ≥7 новых тестов добавлено (3 Plain + 2 Model + 2 integration)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] Один коммит `feat(phase-2): walking skeleton`

## Files touched

- `src/render/mod.rs` (created)
- `src/widgets/model.rs` (created)
- `src/widgets/mod.rs` (modified — `pub mod model`, `build_widgets` impl)
- `src/main.rs` (rewritten)
- `tests/snapshots.rs` (rewritten под новый output-формат)

## Risks & rollback

- **Phase 0 семпл `display_name` отличается от текущего CC-формата**: маловероятно, проверено в Task 2. Если падает — обновить семпл или ослабить ассерт в `tests/snapshots.rs` до `stdout.contains(expected)`.
- **`println!` добавляет `\n`, ассерт `==` не учитывает это**: тест уже использует `stdout.trim() == expected` — корректно.
- **AC-007 тест ловит `cchud:` префикс на stderr**: если `panic = "abort"` сработает где-то — stderr будет другой. Проверять на `not json {{` — гарантированно не паникует, путь через `serde_json::Error`.
- **`Box<dyn Widget>` overhead**: 1 виджет, dispatch <1ns — не в hot path. Phase 7 ревизия.
- **Rollback**: `git revert HEAD` снимает все 5 файлов; `tests/snapshots.rs` вернётся в Phase 1 формат. Если только Phase 1 версия теста нужна — `git checkout HEAD~1 -- tests/snapshots.rs`.
