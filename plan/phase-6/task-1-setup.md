# Task 1 — Setup: deps, cache module skeleton, fixture, DECISIONS

**Files:**
- Modify: `Cargo.toml` (добавить `sonic-rs`, `siphasher`, `time`)
- Create: `src/cache/mod.rs` (skeleton — `pub mod jsonl_types;` etc. пустые stubs; `pub use` re-exports готовых API в T4)
- Create: `src/cache/fixture.rs` (`#[cfg(test)] TranscriptBuilder` helper)
- Modify: `src/lib.rs` (добавить `pub mod cache;`)
- Modify: `docs/DECISIONS.md` (append D-2026-04-28 — sonic-rs vs serde_json)
- Modify: `.gitignore` (добавить `benches/samples/payload-with-transcript-large.json` если такой patten ещё не покрыт)

## Goal

Архитектурный скелет transcript-кэша. Те же инварианты, что у T1 Phase 5:

1. **Deps pinned**: `sonic-rs = "0.5"`, `siphasher = "1"`, `time = "0.3"` с минимальными features.
2. **Изолированный модуль**: `src/cache/` существует и компилируется как skeleton; виджеты Phase 6 будут зависеть только от `pub` API из `cache::mod`.
3. **Test fixture**: `TranscriptBuilder` создаёт tempfile-`.jsonl` для интеграционных тестов парсера/store без копипасты в каждом файле.
4. **DECISIONS-запись**: sonic-rs vs serde_json (perf rationale + Windows fallback контракт).
5. **Phase 5 регрессия = 0**: Phase 5 hyperfine/snapshot/test pipelines зелёные.

Сознательно НЕ включаем (это T2+):
- Любые struct'ы JSONL — это T2.
- Парсер, store, cache_path_for — T3, T4.
- `RenderContext.transcript()` — T5.
- Виджеты — T6, T7, T8.

## Inputs

- Phase 5 завершена (`git tag --list 'v0.3.0'` → `v0.3.0`).
- `cargo build --release --locked` зелёный на main.
- В `dev-dependencies` уже есть `tempfile = "3"` (Phase 3).

---

- [ ] **Step 1: Добавить deps в `Cargo.toml`**

Edit `Cargo.toml`:

- `old_string`:
  ```toml
  # Lazy для виджетов (добавятся в фазе 6)
  # sonic-rs, ratatui, crossterm — позже
  ```
- `new_string`:
  ```toml
  # Phase 6 — JSONL-кэш + transcript-виджеты.
  # sonic-rs: ~3× быстрее serde_json на построчном JSONL (бенч в DECISIONS D-2026-04-28).
  # Pin major: 0.5 — API стабильно начиная с этого выпуска.
  sonic-rs = "0.5"
  # SipHash24 для cache-key (~5 KB binary footprint, no extra deps).
  siphasher = "1"
  # ISO-8601 без std-fmt и serde — экономия binary size; используется только парсером.
  time = { version = "0.3", default-features = false, features = ["parsing", "macros"] }

  # Lazy для виджетов (добавятся в фазе 7)
  # ratatui, crossterm — позже
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

> **Why minimal `time` features:** виджеты не форматируют timestamp'ы — только парсят их в `u64 ms`. `parsing` нужен для `time::PrimitiveDateTime::parse`, `macros` — для compile-time `format_description!`. Без `serde`/`std`/`formatting` бинарь не растёт.

- [ ] **Step 2: Запустить `cargo build` — успешная сборка с новыми deps**

```bash
cargo build --release --locked 2>&1 | tail -10
```

Expected: успешная сборка. `sonic-rs` компилируется ~30-60 sec первый раз (зависит от LLVM-оптимизаций).

```bash
ls -lh target/release/cchud | awk '{print $5}'
```

Expected: < 8 MB (Phase 5 baseline ~6.5 MB + sonic-rs/time/siphasher ~1 MB).

> **Если sonic-rs не собирается на Windows CI** (известный риск): подтвердить fallback план в DECISIONS Step 7. Локально (macos/linux) сборка пройдёт чисто.

- [ ] **Step 3: Создать `src/cache/mod.rs` skeleton**

Create `/Users/igor/mp/startup/cchud/src/cache/mod.rs`:

```rust
//! Transcript JSONL cache — Phase 6.
//!
//! Контракт:
//! - `load_or_build_incremental(path)` (T4) — single entry point. Возвращает
//!   `Option<TranscriptStats>`; None если транскрипт недоступен / битый /
//!   IO error. Никогда не панкует.
//! - Внутренняя структура (`parser`, `store`) — pub(crate); внешние
//!   потребители (виджеты Phase 6) обращаются только через `RenderContext::transcript()`.
//! - `cache/fixture.rs` — `#[cfg(test)]` helper `TranscriptBuilder` для
//!   интеграционных тестов парсера и store.
//!
//! Production-код в этом модуле должен соблюдать lint
//! `unwrap_used = "deny"` / `expect_used = "deny"` — error-path → `Option::None`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod jsonl_types;
pub mod parser;
pub mod store;

#[cfg(test)]
pub mod fixture;

// Re-exports populated by T2 (types) and T4 (store).
// Виджеты Phase 6 берут TranscriptStats и load_or_build_incremental
// через эти re-exports, без знания о parser/store deeper internals.
pub use jsonl_types::{
    BillingBlock, CacheFile, CacheMeta, MessageStats, TranscriptStats, FORMAT_VERSION,
};
pub use store::load_or_build_incremental;
```

Path: `/Users/igor/mp/startup/cchud/src/cache/mod.rs`.

- [ ] **Step 4: Создать пустые stubs для `jsonl_types.rs`, `parser.rs`, `store.rs`**

Чтобы T1 компилировался без ошибок про missing module / missing items в re-export, создаём stubs. Они будут переписаны в T2/T3/T4.

Create `/Users/igor/mp/startup/cchud/src/cache/jsonl_types.rs`:

```rust
//! Phase 6 Task 2 will populate this file with TranscriptEntry, Usage,
//! TranscriptStats, MessageStats, BillingBlock, CacheMeta, CacheFile.
//!
//! T1 stub: minimal placeholder so cache/mod.rs re-exports compile.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: u32 = 1;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TranscriptStats {}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MessageStats;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BillingBlock;

#[derive(Serialize, Deserialize)]
pub struct CacheMeta {
    pub format_version: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CacheFile {
    pub meta: CacheMeta,
    pub stats: TranscriptStats,
}
```

Create `/Users/igor/mp/startup/cchud/src/cache/parser.rs`:

```rust
//! Phase 6 Task 3 will populate parse_transcript, parse_from_offset,
//! merge_stats, apply_entry, ParseState, parse_iso_to_ms, floor_5h.
//!
//! T1 stub: empty.

#![deny(clippy::unwrap_used, clippy::expect_used)]
```

Create `/Users/igor/mp/startup/cchud/src/cache/store.rs`:

```rust
//! Phase 6 Task 4 will populate load_or_build_incremental, read_cache,
//! write_cache_best_effort, cache_path_for.
//!
//! T1 stub: minimal placeholder so cache/mod.rs re-exports compile.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use crate::cache::jsonl_types::TranscriptStats;

#[allow(dead_code)]
#[must_use]
pub fn load_or_build_incremental(_path: &Path) -> Option<TranscriptStats> {
    // T4 will replace this with real impl.
    None
}
```

- [ ] **Step 5: Создать `src/cache/fixture.rs` (TranscriptBuilder helper)**

Create `/Users/igor/mp/startup/cchud/src/cache/fixture.rs`:

```rust
//! Test fixture helpers — Phase 6 Task 1.
//!
//! `TranscriptBuilder` создаёт tempfile-`.jsonl` транскрипты для тестов
//! парсера, store, виджетов. Каждый `.add_user`/`.add_assistant` пишет
//! одну JSONL-строку с типичной CC-формой записи.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Lightweight transcript-фикстура.
pub struct TranscriptBuilder {
    pub dir: TempDir,
    pub path: PathBuf,
    next_user_ts_ms: u64,
}

impl TranscriptBuilder {
    /// Создать пустой транскрипт. Базовый timestamp = 2026-01-01T00:00:00Z.
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("transcript.jsonl");
        std::fs::write(&path, "").unwrap();
        Self {
            dir,
            path,
            next_user_ts_ms: 1_767_225_600_000, // 2026-01-01T00:00:00Z
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Добавить произвольную JSONL-строку (без trailing newline; будет добавлен).
    pub fn raw(&self, line: &str) -> &Self {
        let mut f = OpenOptions::new().append(true).open(&self.path).unwrap();
        writeln!(f, "{line}").unwrap();
        self
    }

    /// Добавить user-message с auto-incremented timestamp (60 sec шаг).
    pub fn add_user(&mut self) -> &mut Self {
        let ts = self.next_user_ts_ms;
        self.next_user_ts_ms += 60_000;
        let iso = unix_ms_to_iso(ts);
        let line = format!(r#"{{"type":"user","timestamp":"{iso}"}}"#);
        self.raw(&line);
        self
    }

    /// Добавить assistant-message с usage и опциональным thinking.effort.
    /// `delta_ms` — задержка после последнего user-msg (для timing).
    pub fn add_assistant(
        &mut self,
        delta_ms: u64,
        input: u64,
        output: u64,
        cache_read: u64,
        cache_creation: u64,
        thinking_effort: Option<&str>,
    ) -> &mut Self {
        let ts = self.next_user_ts_ms.saturating_sub(60_000) + delta_ms;
        let iso = unix_ms_to_iso(ts);
        let thinking = thinking_effort.map_or(String::new(), |level| {
            format!(r#","thinking":{{"effort":"{level}"}}"#)
        });
        let line = format!(
            r#"{{"type":"assistant","timestamp":"{iso}","message":{{"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache_read},"cache_creation_input_tokens":{cache_creation}}}}}{thinking}}}"#
        );
        self.raw(&line);
        self
    }

    /// Перепрыгнуть таймером — useful для block-boundary тестов.
    pub fn advance(&mut self, ms: u64) -> &mut Self {
        self.next_user_ts_ms += ms;
        self
    }

    /// Заменить весь файл произвольным контентом (для truncate / corrupt тестов).
    pub fn overwrite(&self, content: &str) -> &Self {
        std::fs::write(&self.path, content).unwrap();
        self
    }

    /// Размер файла в байтах.
    pub fn size(&self) -> u64 {
        std::fs::metadata(&self.path).unwrap().len()
    }
}

fn unix_ms_to_iso(ms: u64) -> String {
    // Naive ISO-8601 без зависимости от time crate (она будет в parser).
    // Формат: 2026-MM-DDTHH:MM:SS.sssZ
    let secs = ms / 1000;
    let sub_ms = ms % 1000;
    // Берём за основу 2026-01-01 эпоху и считаем относительно неё (тесты
    // не проверяют точный календарь, только консистентность).
    let base = 1_767_225_600u64; // 2026-01-01T00:00:00Z
    let delta = secs.saturating_sub(base);
    let h = (delta / 3600) % 24;
    let m = (delta / 60) % 60;
    let s = delta % 60;
    let day = 1 + (delta / 86400);
    format!("2026-01-{day:02}T{h:02}:{m:02}:{s:02}.{sub_ms:03}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_writes_empty_file() {
        let b = TranscriptBuilder::new();
        assert_eq!(b.size(), 0);
    }

    #[test]
    fn builder_adds_user_and_assistant_lines() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(500, 100, 200, 0, 0, None);
        let content = std::fs::read_to_string(b.path()).unwrap();
        assert_eq!(content.lines().count(), 2);
        assert!(content.contains("\"type\":\"user\""));
        assert!(content.contains("\"type\":\"assistant\""));
        assert!(content.contains("\"input_tokens\":100"));
    }

    #[test]
    fn builder_includes_thinking_effort() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 1, 1, 0, 0, Some("high"));
        let content = std::fs::read_to_string(b.path()).unwrap();
        assert!(content.contains(r#""thinking":{"effort":"high"}"#));
    }

    #[test]
    fn raw_appends_arbitrary_line() {
        let b = TranscriptBuilder::new();
        b.raw("not a valid json");
        let content = std::fs::read_to_string(b.path()).unwrap();
        assert_eq!(content.trim_end(), "not a valid json");
    }

    #[test]
    fn overwrite_replaces_content() {
        let b = TranscriptBuilder::new();
        b.raw("first").overwrite("second");
        let content = std::fs::read_to_string(b.path()).unwrap();
        assert_eq!(content, "second");
    }

    #[test]
    fn advance_skips_time() {
        let mut b = TranscriptBuilder::new();
        let before = b.next_user_ts_ms;
        b.advance(5 * 3600 * 1000);
        assert_eq!(b.next_user_ts_ms, before + 5 * 3600 * 1000);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/cache/fixture.rs`.

> **Why no `time` crate в fixture:** `time` зайдёт в T3 в `parser`. Fixture не парсит, только пишет, и конструирует ISO вручную из эпохи 2026-01-01. Тесты парсера в T3 проверят, что `parse_iso_to_ms` принимает этот формат — это и есть контракт fixture ↔ parser.

- [ ] **Step 6: Подключить `pub mod cache;` в `src/lib.rs`**

Edit `src/lib.rs`:

- `old_string`:
  ```rust
  pub mod config;
  pub mod git;
  pub mod render;
  pub mod types;
  pub mod util;
  pub mod widgets;
  ```
- `new_string`:
  ```rust
  pub mod cache;
  pub mod config;
  pub mod git;
  pub mod render;
  pub mod types;
  pub mod util;
  pub mod widgets;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/lib.rs`

- [ ] **Step 7: Записать decision в `docs/DECISIONS.md`**

Read `docs/DECISIONS.md`. Найти конец файла. Append:

```markdown

---

## D-2026-04-28 — sonic-rs vs serde_json для transcript JSONL

**Контекст:** Phase 6 парсит JSONL-транскрипты CC размером до 50 МБ. PRD NFR §6 требует cold parse < 10 ms, warm + 1 МБ append < 3 ms. На построчном JSONL парсинг — главный bottleneck (≈80% wall-clock cold-path).

**Кандидаты:**
- `serde_json` — уже в deps; универсально; ~150-200 MB/s throughput на наших структурах.
- `sonic-rs` — SIMD-ускоренный; ≈3× быстрее на построчном чтении; ~5 MB крейт; pure Rust runtime.

**Решение:** `sonic-rs = "0.5"`.

**Обоснование:**
1. **Perf headroom**: 50 МБ / 200 MB/s = 250 ms на serde_json — выше budget. sonic-rs даёт ~80 ms cold parse и < 5 ms на 1 МБ append. Оба в budget, но sonic-rs оставляет запас на будущие виджеты.
2. **Зависимость уже изолирована**: только в `src/cache/parser.rs`. Если потребуется fallback — точечная замена через `cfg`.
3. **Bincode остаётся `serde_json`-совместимым**: `TranscriptStats` сериализуется в bincode (T4), на чтение — sonic-rs. Один формат на серде, разные движки.

**Trade-offs приняты:**
- Дополнительная dep ~0.5 MB binary. Митигация: lto+strip профиль release.
- API менее стабилен. Митигация: pin `sonic-rs = "0.5"` (major); breaking changes — отдельный PR.
- Windows CI: исторически были баги в SIMD-кодгене на MSVC. Митигация: если CI windows-latest red — добавить `#[cfg(windows)] use serde_json` ветку в `parser::parse_line` (10 строк); план держится в Step 8 рисках T1.

**Альтернативы рассмотрены:**
- `simd-json` — мощнее на больших документах, но overhead выше для построчного < 1 KB JSONL.
- кастомный JSON parser — переизобретение, не оправдано.

**Owner:** Igor Fonin
```

`file_path`: `/Users/igor/mp/startup/cchud/docs/DECISIONS.md`.

- [ ] **Step 8: Запустить все тесты — Phase 5 не должна регрессировать**

```bash
cargo test --locked 2>&1 | tail -10
```

Expected: все Phase 2/3/5 тесты зелёные. Новые тесты в `cache::fixture::tests` (≥6 штук) тоже зелёные.

```bash
cargo test --locked cache:: 2>&1 | tail -15
```

Expected: ≥6 тестов passed (все в `fixture::tests`).

- [ ] **Step 9: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

> **Возможные clippy-замечания:** `pedantic::module_name_repetitions` на `TranscriptStats` / `TranscriptBuilder` — namespace-collision избегается через `cache::TranscriptStats` импорт; имя оставляем явное.

- [ ] **Step 10: Verification — task-specific gate**

```bash
grep -c '^pub mod cache;' src/lib.rs
grep -c 'sonic-rs' Cargo.toml
grep -c 'siphasher' Cargo.toml
grep -c 'time = { version' Cargo.toml
ls src/cache/
ls -lh target/release/cchud | awk '{print $5}'
```

Expected:
```
1                      (cache module exposed)
1                      (sonic-rs dep)
1                      (siphasher dep)
1                      (time dep)
fixture.rs jsonl_types.rs mod.rs parser.rs store.rs
< 8M                   (binary size)
```

- [ ] **Step 11: Phase 5 snapshot regression check**

```bash
cargo test --locked snapshots 2>&1 | tail -10
cargo insta pending-snapshots 2>&1 | head -5
```

Expected: snapshot'ы зелёные, pending пусто.

- [ ] **Step 12: Commit**

```bash
git add Cargo.toml Cargo.lock src/cache/ src/lib.rs docs/DECISIONS.md
git commit -m "feat(phase-6): T1 setup — cache skeleton, sonic-rs/siphasher/time deps

Phase 6 foundation. Adds sonic-rs=0.5 (~3× faster JSONL parse vs
serde_json on lines of typical CC entry size), siphasher=1 (cache-key
hash, ~5 KB footprint), time=0.3 (ISO-8601 parse, default-features off).

New src/cache/ module with stubs: jsonl_types/parser/store. Public API
re-exports TranscriptStats, FORMAT_VERSION, load_or_build_incremental
from cache::mod — widgets reach the cache only through these.

Test fixture (src/cache/fixture.rs, cfg(test)) writes tempdir transcripts
via TranscriptBuilder::add_user/add_assistant for parser/store integration
tests in T3/T4.

DECISIONS: sonic-rs vs serde_json rationale recorded.

Task 1/10 of Phase 6. No widgets yet — T2 adds JSONL types."
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `Cargo.toml` содержит `sonic-rs = "0.5"`, `siphasher = "1"`, `time = "0.3"` с минимальными features
- [ ] `cargo build --release --locked` зелёный, binary < 8 MB
- [ ] `src/cache/{mod,jsonl_types,parser,store,fixture}.rs` созданы (последний под `#[cfg(test)]`)
- [ ] `src/cache/mod.rs` re-exports `TranscriptStats`, `FORMAT_VERSION`, `load_or_build_incremental`
- [ ] `cache::fixture::TranscriptBuilder::{new, add_user, add_assistant, raw, overwrite, advance, size}` работают
- [ ] `pub mod cache;` добавлен в `src/lib.rs`
- [ ] Phase 5 тесты не регрессируют (`cargo test --locked` зелёный)
- [ ] `docs/DECISIONS.md` содержит запись D-2026-04-28 (sonic-rs vs serde_json)
- [ ] Один commit `feat(phase-6): T1 setup ...`

## Files touched

- `Cargo.toml` (modified — 3 deps)
- `Cargo.lock` (auto)
- `src/cache/mod.rs` (created — module declaration + re-exports)
- `src/cache/jsonl_types.rs` (created — stub for T2)
- `src/cache/parser.rs` (created — empty stub for T3)
- `src/cache/store.rs` (created — stub for T4)
- `src/cache/fixture.rs` (created — `TranscriptBuilder`)
- `src/lib.rs` (modified — `pub mod cache;`)
- `docs/DECISIONS.md` (appended — D-2026-04-28)

## Risks & rollback

- **sonic-rs 0.5 build error на macos/linux**: маловероятно (perf-критичный крейт с активной поддержкой). Действие: попробовать `cargo update -p sonic-rs`; если воспроизводится — pin `0.5.x` точно.
- **sonic-rs Windows CI red в T1 build**: смотрим CI после первого push. Mitigation: добавить `#[cfg(windows)]` ветку в `cache::parser::parse_line` (T3) с `serde_json::from_str` fallback. Документировать в DECISIONS.
- **Binary > 8 MB**: подсчитать `cargo bloat --release` чтобы понять, какая dep набрала вес. Если sonic-rs > 1.5 MB — ОК (perf оправдывает). Если значительно больше — рассмотреть `simd-json` или fall back на `serde_json`.
- **Rollback**: `git revert HEAD` — снимает 3 deps и `src/cache/`. Phase 5 продолжает работать.
