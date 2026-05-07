# Task 5 — RenderContext: lazy transcript() + now_ms + util/now.rs

**Files:**
- Create: `src/util/now.rs` (`unix_now_ms`)
- Modify: `src/util/mod.rs` (`pub mod now;`)
- Modify: `src/widgets/mod.rs` (добавить `transcript: OnceCell<Option<TranscriptStats>>`, `now_ms: u64`; method `transcript()`; конструктор)

## Goal

Точка интеграции `cache::` в render pipeline.

Контракт T5:
1. **`unix_now_ms()`** — единственный helper для текущего времени; используется конструктором `RenderContext::new`. Тесты строят контекст через `RenderContext { now_ms, .. }` (никаких `_for_tests` хелперов).
2. **`RenderContext.transcript: OnceCell<Option<TranscriptStats>>`** — initialized at-first-call. Если `payload.transcript_path == None` → `OnceCell` хранит `None`, виджеты возвращают None.
3. **`RenderContext.now_ms: u64`** — pub field; legitimate access из `BlockTimer` (T7); инжектится через конструктор. Тестов с фиксированным временем — много.
4. **`RenderContext::transcript()`** — `&self -> Option<&TranscriptStats>`; вызывает `cache::load_or_build_incremental` максимум один раз за render.
5. **Стоимость отсутствия transcript-виджетов в строке = 0**: если ни один виджет не зовёт `ctx.transcript()`, `OnceCell` остаётся пустым; никаких IO.
6. **Phase 5 регрессия = 0**: `git()` метод и `git: OnceCell<...>` — без изменений.

## Inputs

- T1–T4 закрыты.
- `RenderContext::new` — текущая const fn (см. `src/widgets/mod.rs:54-62` после Phase 5).

---

- [ ] **Step 1: Создать `src/util/now.rs`**

Create `/Users/igor/mp/startup/cchud/src/util/now.rs`:

```rust
//! Unix-epoch ms helper — Phase 6 Task 5.
//!
//! Используется `RenderContext::new` для дефолтного `now_ms`. Тесты
//! инжектируют свой `now_ms` через explicit field-init синтаксис, не вызывая
//! этот хелпер.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Текущее время в Unix-ms. На системах с broken clock возвращает 0.
#[must_use]
pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn unix_now_ms_returns_positive() {
        // Тест прогоняется в 2026 — таймстамп явно > 1 января 1970.
        assert!(unix_now_ms() > 1_700_000_000_000);
    }

    #[test]
    fn unix_now_ms_monotonic_within_call() {
        let a = unix_now_ms();
        let b = unix_now_ms();
        assert!(b >= a);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/util/now.rs`.

- [ ] **Step 2: Зарегистрировать `now` в `src/util/mod.rs`**

Edit `src/util/mod.rs`:

- `old_string`:
  ```rust
  pub mod ansi;
  pub mod ascii_bar;
  pub mod duration;
  pub mod model_context_size;
  ```
- `new_string`:
  ```rust
  pub mod ansi;
  pub mod ascii_bar;
  pub mod duration;
  pub mod model_context_size;
  pub mod now;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/util/mod.rs`

- [ ] **Step 3: Расширить `RenderContext` в `src/widgets/mod.rs`**

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
  pub struct RenderContext<'a> {
      pub payload: &'a StatusPayload,
      #[allow(dead_code)]
      pub settings: &'a Settings,
      /// Phase 5: lazy git discover. None если cwd не git-репо.
      #[allow(dead_code)]
      git: std::cell::OnceCell<Option<crate::git::GitInfo>>,
  }

  impl<'a> RenderContext<'a> {
      #[must_use]
      pub const fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
          Self {
              payload,
              settings,
              git: std::cell::OnceCell::new(),
          }
      }

      /// Lazy: вызывает `gix::discover(cwd)` максимум один раз. None если
      /// payload без cwd или cwd вне git-репо.
      #[allow(dead_code)]
      pub fn git(&self) -> Option<&crate::git::GitInfo> {
          self.git
              .get_or_init(|| {
                  let cwd = self.payload.workspace.current_dir.as_str();
                  crate::git::GitInfo::discover(std::path::Path::new(cwd))
              })
              .as_ref()
      }
  }
  ```
- `new_string`:
  ```rust
  pub struct RenderContext<'a> {
      pub payload: &'a StatusPayload,
      #[allow(dead_code)]
      pub settings: &'a Settings,
      /// Phase 5: lazy git discover. None если cwd не git-репо.
      #[allow(dead_code)]
      git: std::cell::OnceCell<Option<crate::git::GitInfo>>,
      /// Phase 6: lazy transcript-кэш. None если payload без transcript_path
      /// или транскрипт недоступен.
      #[allow(dead_code)]
      transcript: std::cell::OnceCell<Option<crate::cache::TranscriptStats>>,
      /// Phase 6: текущее время в Unix-ms. Дефолт = `unix_now_ms()`.
      /// Тесты могут перезаписать через field-init синтаксис.
      pub now_ms: u64,
  }

  impl<'a> RenderContext<'a> {
      #[must_use]
      pub fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
          Self {
              payload,
              settings,
              git: std::cell::OnceCell::new(),
              transcript: std::cell::OnceCell::new(),
              now_ms: crate::util::now::unix_now_ms(),
          }
      }

      /// Lazy: вызывает `gix::discover(cwd)` максимум один раз. None если
      /// payload без cwd или cwd вне git-репо.
      #[allow(dead_code)]
      pub fn git(&self) -> Option<&crate::git::GitInfo> {
          self.git
              .get_or_init(|| {
                  let cwd = self.payload.workspace.current_dir.as_str();
                  crate::git::GitInfo::discover(std::path::Path::new(cwd))
              })
              .as_ref()
      }

      /// Lazy: парсит JSONL-транскрипт через `cache::load_or_build_incremental`
      /// максимум один раз за render. None если `payload.transcript_path`
      /// пусто, файл не читается или JSONL битый.
      #[allow(dead_code)]
      pub fn transcript(&self) -> Option<&crate::cache::TranscriptStats> {
          self.transcript
              .get_or_init(|| {
                  let path = self.payload.transcript_path.as_deref()?;
                  crate::cache::load_or_build_incremental(std::path::Path::new(path))
              })
              .as_ref()
      }

      /// Test-only: pre-populate transcript cell с фиксированной `TranscriptStats`.
      /// Используется в unit-тестах T6/T7/T8 чтобы не зависеть от файлового IO.
      #[cfg(test)]
      pub fn set_transcript_for_tests(&self, stats: Option<crate::cache::TranscriptStats>) {
          let _ = self.transcript.set(stats);
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

> **`const fn new` → `fn new`**: добавление `now_ms: unix_now_ms()` делает функцию не-const (на edition 2024 stable `SystemTime::now` не const). Это намеренно. Существующие call site'ы `const RenderContext::new(...)` — все ОК потому что были runtime call'ами; clippy не пожалуется.

- [ ] **Step 4: Запустить тесты — никаких регрессий**

```bash
cargo build --locked 2>&1 | tail -5
```

Expected: успешная сборка. Если есть `cannot call non-const fn` — снять `const` где-то в call sites (не должно быть).

```bash
cargo test --locked 2>&1 | tail -10
```

Expected: все Phase 2/3/5 тесты зелёные. Никаких изменений в их выводе.

```bash
cargo test --locked widgets:: 2>&1 | tail -5
```

Expected: все widget тесты зелёные.

- [ ] **Step 5: Smoke test — `transcript()` без виджетов = 0 IO**

Append unit test в `src/widgets/mod.rs` (под `#[cfg(test)] mod ...`):

Edit `src/widgets/mod.rs` — найти конец файла (после `default_style_tests`-блока):

- `old_string`:
  ```rust
  fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
  ```
- `new_string`:
  ```rust
  #[cfg(test)]
  #[allow(clippy::unwrap_used, clippy::expect_used)]
  mod transcript_ctx_tests {
      use super::*;
      use crate::config::default_line;
      use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

      fn payload_no_transcript() -> StatusPayload {
          StatusPayload {
              session_id: "test".into(),
              model: ModelInfo {
                  id: "m".into(),
                  display_name: "M".into(),
              },
              workspace: Workspace {
                  current_dir: "/tmp".into(),
                  project_dir: None,
                  added_dirs: None,
              },
              transcript_path: None,
              cwd: None,
              version: None,
              fast_mode: None,
              exceeds_200k_tokens: None,
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

      #[test]
      fn transcript_returns_none_when_path_missing() {
          let p = payload_no_transcript();
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert!(ctx.transcript().is_none());
      }

      #[test]
      fn transcript_returns_none_for_invalid_path() {
          let mut p = payload_no_transcript();
          p.transcript_path = Some("/nonexistent/__cchud_test_does_not_exist.jsonl".into());
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert!(ctx.transcript().is_none());
      }

      #[test]
      fn now_ms_can_be_overridden_for_tests() {
          let p = payload_no_transcript();
          let s = default_line();
          let mut ctx = RenderContext::new(&p, &s);
          ctx.now_ms = 1_234_567_890;
          assert_eq!(ctx.now_ms, 1_234_567_890);
      }
  }

  fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 6: Запустить новые тесты**

```bash
cargo test --locked widgets::transcript_ctx_tests 2>&1 | tail -10
```

Expected: 3 теста passed.

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Phase 5 hyperfine не требуется в T5 — Phase 5 snapshot'ы покрывают регрессию.

- [ ] **Step 8: Verification — task-specific**

```bash
grep -c 'pub fn transcript' src/widgets/mod.rs
grep -c 'pub now_ms: u64' src/widgets/mod.rs
grep -c 'transcript: std::cell::OnceCell' src/widgets/mod.rs
grep -c 'pub mod now;' src/util/mod.rs
grep -c 'pub fn unix_now_ms' src/util/now.rs
```

Expected:
```
1
1
1
1
1
```

- [ ] **Step 9: Phase 5 regression check**

```bash
cargo test --locked snapshots 2>&1 | tail -10
cargo insta pending-snapshots 2>&1 | head -5
```

Expected: Phase 5 snapshot'ы зелёные, pending пусто.

- [ ] **Step 10: Commit**

```bash
git add src/util/now.rs src/util/mod.rs src/widgets/mod.rs
git commit -m "feat(phase-6): T5 RenderContext::transcript() + now_ms

Adds two RenderContext members for Phase 6:
- transcript: OnceCell<Option<TranscriptStats>> — lazy via
  cache::load_or_build_incremental(payload.transcript_path). None if no
  path or cache build fails. Single load per render.
- now_ms: u64 — pub field, default = util::now::unix_now_ms(). Tests
  override via explicit field-init (BlockTimer needs deterministic now).

util/now.rs: unix_now_ms() helper. Broken clock → 0; positive guaranteed
in any sane runtime.

Cost for non-transcript widgets remains 0: ctx.transcript() is never
called, OnceCell stays empty, no IO performed.

Task 5/10 of Phase 6. T6 starts adding the 8 transcript widgets."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/util/now.rs` создан с `pub fn unix_now_ms() -> u64`
- [ ] `pub mod now;` в `src/util/mod.rs`
- [ ] `RenderContext` получил `transcript: OnceCell<Option<TranscriptStats>>` и `pub now_ms: u64`
- [ ] `RenderContext::new` инжектит `now_ms = unix_now_ms()`
- [ ] `RenderContext::transcript() -> Option<&TranscriptStats>` зовёт `cache::load_or_build_incremental` максимум один раз
- [ ] При `payload.transcript_path = None` → `transcript()` возвращает None без IO
- [ ] `RenderContext::set_transcript_for_tests(Option<TranscriptStats>)` под `#[cfg(test)]` доступен (для T6/T7/T8 unit-тестов)
- [ ] Phase 5 тесты не регрессируют (`cargo test --locked` зелёный); `transcript_ctx_tests` добавлены (3 теста)
- [ ] Один commit `feat(phase-6): T5 RenderContext::transcript() ...`

## Files touched

- `src/util/now.rs` (created)
- `src/util/mod.rs` (modified — `pub mod now;`)
- `src/widgets/mod.rs` (modified — RenderContext fields + method + smoke tests)

## Risks & rollback

- **`RenderContext::new` стал не-const**: один вызов — runtime construction всё равно не пострадает; clippy `pedantic::missing_const_for_fn` уже невозможен (`unix_now_ms` не const). Если кто-то держит `const X: RenderContext = RenderContext::new(...)` (не должен) — поломается; grep'ом проверим.
- **Изменение API для тестов**: бывшие тесты использовали `RenderContext::new(...)`. Новый ctor работает идентично; тесты, которые хотят кастомный `now_ms`, делают `let mut ctx = ...; ctx.now_ms = ...;` — стандартный паттерн.
- **`OnceCell` не reset'ится** между рендерами: каждый рендер создаёт свежий `RenderContext` (вызов `build_widgets` → `render_pipeline` в `main.rs`). Кэш на уровне диска, в памяти — только пер-рендер.
- **Rollback**: `git revert HEAD` снимает Phase 6 ctx-changes; T1–T4 модули остаются скомпилируемыми (никто их не использует кроме `pub use` в lib.rs).
