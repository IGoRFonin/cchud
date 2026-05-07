# Фаза 6 — Implementation Plan (Transcript & JSONL cache, 0.4.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Релиз `0.4.0` cchud — 8 transcript-виджетов поверх нового JSONL-кэша с incremental tail-merge. Парсинг 50 МБ удерживается в `< 10 ms` cold и `< 3 ms` warm (после 1 МБ append). Lazy `RenderContext::transcript()` через `OnceCell` — нулевая стоимость для строк без transcript-виджетов. Архитектурный задел Phase 7 (HTTP-кластер, `Skills`, `SessionUsage`, `WeeklyUsage`) остаётся нетронутым.

**Architecture:** Линейная цепочка из 10 vertical-slice задач. T1 — deps + `cache/mod.rs` skeleton + DECISIONS. T2 — pure-types `cache/jsonl_types.rs`. T3 — `cache/parser.rs` (sonic-rs построчно, `parse_from_offset`, `merge_stats`, block-boundary deterministic). T4 — `cache/store.rs` (bincode read/write, `load_or_build_incremental`, siphasher cache key). T5 — `RenderContext.transcript()` через `OnceCell` + `now_ms: u64` + `util/now.rs`. T6–T8 — 8 виджетов в трёх файлах (`transcript_tokens.rs`, `transcript_timing.rs`, `transcript_meta.rs`) + `util/format_tokens.rs`. T9 — snapshot suite ≥5 фикстур + hyperfine cold/warm/append. T10 — release 0.4.0.

**Scope correction (vs `plan/phase-6-transcript.md`):** Старый outline обещал 19 виджетов, включая HTTP (`SessionUsage`, `WeeklyUsage`, `BlockResetTimer`, `WeeklyResetTimer`) и `ClaudeAccountEmail`/`Skills`. После canonical `docs/widgets.md` Phase 6 = 8 виджетов. HTTP-кластер + `Skills` уехали в Phase 7. См. spec § "Scope correction".

**Tech Stack:** Rust 1.85 (Edition 2024), `sonic-rs = "0.5"` (новая dep, T1, ≈3× быстрее `serde_json` на построчном JSONL), `siphasher = "1"` (новая dep, T1, hash для cache-key — SipHash24, ~5 KB), `time = { version = "0.3", default-features = false, features = ["parsing", "macros"] }` (новая dep, T1, ISO-8601 без std-fmt/serde), `bincode = "1"` (Phase 5, формат кэша), `serde`/`serde_json`/`tempfile` (уже есть), `dirs` (уже есть, для `~/.cache/cchud/`).

---

## DAG задач

```
T1 (setup) ──► T2 (types) ──► T3 (parser) ──► T4 (store) ──► T5 (render-ctx)
                                                                    │
                                                                    ▼
                                                              T6 (tokens, 5w)
                                                                    │
                                                                    ▼
                                                              T7 (timing, 2w)
                                                                    │
                                                                    ▼
                                                              T8 (thinking, 1w)
                                                                    │
                                                                    ▼
                                                       T9 (snapshots+bench) ──► T10 (release)
```

Линейная цепочка. Между задачами — review checkpoint. T2–T4 — модель + парсер + IO (без виджетов). T5 — точка интеграции. T6–T8 — каждый кластер виджетов работает независимо после T5.

| # | Файл | Цель | Виджеты | Гейт |
|---|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | `sonic-rs`, `siphasher`, `time` deps; `src/cache/mod.rs` skeleton; `cache/fixture.rs` `TranscriptBuilder`; `pub mod cache;` в `lib.rs`; DECISIONS-запись (sonic-rs vs serde_json) | 0 | `cargo test cache::fixture` зелёный; binary < 7 MB; Phase 5 snapshot'ы зелёные |
| 2 | [`task-2-jsonl-types.md`](./task-2-jsonl-types.md) | `cache/jsonl_types.rs` — `TranscriptEntry`, `MessagePayload`, `Usage`, `ThinkingMeta`, `TranscriptStats`, `MessageStats`, `BillingBlock`, `CacheMeta`, `CacheFile`; `FORMAT_VERSION = 1`; bincode round-trip + format_version mismatch detection | 0 | `cargo test cache::jsonl_types` зелёный |
| 3 | [`task-3-parser.md`](./task-3-parser.md) | `cache/parser.rs` — `parse_transcript`, `parse_from_offset`, `merge_stats`, `apply_entry`, `ParseState`, `parse_iso_to_ms`, `floor_5h`; broken lines skip; block-boundary detеrministic; merge_stats associative + tail-wins на `last_*` полях | 0 | `cargo test cache::parser` зелёный (≥10 unit-тестов) |
| 4 | [`task-4-cache-store.md`](./task-4-cache-store.md) | `cache/store.rs` — `load_or_build_incremental`, `read_cache`, `write_cache_best_effort`, `cache_path_for` (siphasher); format-version bump → silent reset; corrupt bincode → silent reset; `~/.cache/cchud/transcript-<hex16>.bincode` | 0 | `cargo test cache::store` зелёный (full / append / truncate / corrupt / format-bump сценарии) |
| 5 | [`task-5-render-ctx.md`](./task-5-render-ctx.md) | `RenderContext.transcript: OnceCell<Option<TranscriptStats>>`; `RenderContext.now_ms: u64`; `util/now.rs` (`unix_now_ms`); конструктор передаёт `now_ms = unix_now_ms()`; в тестах поле перезаписывается через `RenderContext { now_ms, .. }` | 0 | `cargo test widgets::` зелёный; ноль регрессий Phase 3/5 |
| 6 | [`task-6-tokens.md`](./task-6-tokens.md) | `util/format_tokens.rs` (k/M formatter без хвостовых нулей); `widgets/transcript_tokens.rs` — `TokensCached`, `TokensTotal`, `InputSpeed`, `OutputSpeed`, `TotalSpeed`; 5 enum-вариантов в `WidgetConfig`; 5 case'ов в `build_one`; ≥10 unit-тестов | 5 | `cargo test widgets::transcript_tokens` зелёный; `cargo test util::format_tokens` зелёный |
| 7 | [`task-7-timing.md`](./task-7-timing.md) | `widgets/transcript_timing.rs` — `BlockTimer`, `SessionDuration`; 2 enum-варианта; 2 case'а в `build_one`; tests с фиксированным `now_ms` | 2 | `cargo test widgets::transcript_timing` зелёный |
| 8 | [`task-8-thinking.md`](./task-8-thinking.md) | `widgets/transcript_meta.rs` — `ThinkingEffort`; 1 enum-вариант; 1 case в `build_one`; формат `🧠 {level}` | 1 | `cargo test widgets::transcript_meta` зелёный |
| 9 | [`task-9-snapshots-bench.md`](./task-9-snapshots-bench.md) | `tests/snapshots_transcript.rs` ≥5 фикстур (`empty`/`small-fresh`/`with-thinking`/`block-rollover`/`partial-tail`); `benches/configs/phase-6-8w.json`; `benches/phase-6.md`; `scripts/gen-large-transcript.sh`; hyperfine cold < 10 ms / warm < 2 ms / +1MB append < 3 ms; cchud-20w без регрессии > 10% | 0 | hyperfine targets met; insta review — все snapshot'ы committed без `.snap.new` |
| 10 | [`task-10-release.md`](./task-10-release.md) | `Cargo.toml` 0.4.0; `CHANGELOG.md`; README таблица supported 54/60; `docs/widgets.md` 8 строк TODO → DONE, Skills 6 → 7; `plan/README.md` Phase 6 → `[x]`, Phase 7 → `[~]`; `git tag v0.4.0`; `manual-test-log.md` (battle-test 5+ мин) | 0 | `cchud --version` показывает `0.4.0`; tag создан и запушен; CI matrix зелёный |

**Итого:** 8 виджетов, ~28–38 часов, ~3–5 рабочих дней.

## Pre-flight (требования до старта Task 1)

- **Phase 5 завершена** — git-виджеты merged, ветка main стабильна, релиз 0.3.0 запушен (`git tag --list 'v0.3.0'`).
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое**: `git status` пусто.
- **`rustc` ≥ 1.85** (см. `rust-toolchain.toml`).
- **`hyperfine` ≥ 1.20.0**.
- **`cargo-insta`** установлен (`cargo install cargo-insta`).
- **`benches/samples/`** содержит существующие Phase 0/3/5 JSON-payload'ы; T9 добавит `payload-with-transcript-{small,large}.json`.
- **CC активная сессия** для T10 manual-test (`transcript_path` в payload реально существует).
- **`du -sh ~/.cache/cchud/`** — должен быть пустым или ≤ 10 KB перед T9 cold-bench (cache wipe = `rm -rf ~/.cache/cchud/transcript-*.bincode`).

## Definition of Done всей Фазы 6

- [ ] 8 transcript-виджетов реализованы; каждый покрыт ≥2 unit-тестами (happy + 1+ edge)
- [ ] `cache::` модуль изолирован — виджеты обращаются к нему только через `RenderContext::transcript()`, не импортируют `parser`/`store` напрямую
- [ ] Incremental tail-merge срабатывает при `prev_size <= new_size && mtime_ns >= prev_mtime`; truncate / format-bump / corrupt → silent full rebuild
- [ ] `format_version` mismatch не крэшит, тихо пересобирает кэш
- [ ] Битые JSONL-строки и нераспаршеваемые ISO-8601 timestamp'ы пропускаются молча
- [ ] `cargo build --release --locked` зелёный; binary < 8 MB
- [ ] `cargo test --locked` зелёный (≥150 тестов суммарно: ≥34 новых unit + Phase 3/5 без регрессии)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] `cargo insta review` — все snapshot'ы committed без `.snap.new`
- [ ] CI matrix зелёный (macos / ubuntu / windows)
- [ ] Hyperfine: cold 50 МБ < 10 ms; warm hit < 2 ms; warm + 1 МБ append < 3 ms; cchud-20w (Phase 5 baseline) без регрессии > 10% — `benches/phase-6.md` сохранён в репо
- [ ] `cchud --version` печатает `0.4.0`
- [ ] Никаких `unwrap`/`expect` в `src/cache/` (production-код) и `src/widgets/transcript_*` (lint enforced на уровне модуля)
- [ ] Manual battle-test ≥ 5 мин на этом репо во время сессии с CC, лог в `plan/phase-6/manual-test-log.md`
- [ ] `git tag v0.4.0` создан и запушен
- [ ] `plan/README.md` отмечает Phase 6 как `[x]`, Phase 7 как `[~]`
- [ ] `docs/widgets.md`: 8 transcript-виджетов TODO → DONE; Skills counter 6 → 7
- [ ] README таблица "supported widgets" 54/60 (46 после Phase 5 + 8 новых)
- [ ] `CHANGELOG.md` запись 0.4.0
- [ ] 1+ запись в `docs/DECISIONS.md` (sonic-rs vs serde_json)

## Связи

- **Spec этого плана:** [`../../docs/superpowers/specs/2026-04-28-phase-6-transcript-design.md`](../../docs/superpowers/specs/2026-04-28-phase-6-transcript-design.md) — full design rationale, decision table, error contract
- **Outline (predecessor):** [`../phase-6-transcript.md`](../phase-6-transcript.md) — оригинальный план; scope скорректирован 19 → 8 (см. spec § "Scope correction")
- **PRD:** [`../../docs/prd-cchud.md`](../../docs/prd-cchud.md) — REQ-101 (JSONL bincode cache), NFR §6 (parsing < 10 ms cold / < 2 ms warm + append)
- **Predecessor plan:** [`../phase-5/`](../phase-5/) — конвенции стиля, gate-структура, manual-test-log шаблон, lazy `OnceCell` паттерн `RenderContext::git()`
- **Widgets registry:** [`../../docs/widgets.md`](../../docs/widgets.md) — canonical список 60 виджетов; Phase 6 = 8 transcript-строк
- **Future phases:**
  - Phase 7 — env/http-кластер (6+ виджетов, `Skills`, `SessionUsage`, `WeeklyUsage`, `BlockResetTimer`, `WeeklyResetTimer`, `ClaudeAccountEmail`); `Skills` использует `RenderContext::transcript()` (готов в Phase 6)
  - Phase 9 — release engineering, миграция на bincode 2

## Риски и митигации

- **sonic-rs Windows-баги** — Windows CI runner red. Митигация: fallback на `serde_json` через `cfg(windows)` в `cache/parser.rs` (опциональная ветка); CI matrix покрывает сценарий с момента T1.
- **50 МБ-фикстура раздувает CI runtime** — генерируется на лету в `scripts/gen-large-transcript.sh` (≈ 2 sec), git-ignored через `.gitignore`.
- **Конкурентная запись CC во время read** — best-effort snapshot semantics, никакого `flock`. Недосчитанные сообщения подберутся следующим рендером (через append-merge).
- **Bincode breaking change** — pin `bincode = "=1.x.y"` (точная версия как gix); `format_version` mismatch → silent reset кэша.
- **Block-boundary edge на 5h** — детерминистичный `floor(ts / 5h_ms) * 5h_ms`; unit-тест на timestamp ровно на границе и за 1 ms до неё.
- **ISO-8601 разные форматы** — `time` crate с custom parse spec; unit-тест на 4 формата (с/без миллисекунд / `Z` / `+00:00`).
- **Старые сессии без `thinking.effort`** — `Option<String>`, `ThinkingEffort` молча скрывается. Lint enforced.
- **Огромный `Vec<BillingBlock>` за год** — Phase 6 рендерит только `last()`; ~32 KB max при 2k блоков; неcritical.
- **`~/.cache/cchud/` no write** — best-effort write; функционал сохраняется (full reparse каждый раз).
- **Truncate transcript в середине read** — full rebuild на следующем рендере; текущий рендер вернёт частичные stats без crash.
- **`OnceCell` thread-safety** — single-threaded render path. Phase 8 TUI потребует `OnceLock` — отдельный PR.
- **`unwrap`/`expect` regression в hot-path** — `#![deny(clippy::unwrap_used, clippy::expect_used)]` в `src/cache/mod.rs` и каждом `widgets/transcript_*.rs`. Тесты могут использовать через `#![allow(...)]` на `mod tests`.
