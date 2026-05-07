# Task 3 — Parser: parse_transcript, parse_from_offset, merge_stats

**Files:**
- Modify: `src/cache/parser.rs` (заменяет T1 stub полным парсером)

## Goal

Чистый парсер JSONL → `TranscriptStats` без знания о cache file (это T4).

Контракт T3:
1. `parse_transcript(path) -> Option<TranscriptStats>` — full parse от offset 0.
2. `parse_from_offset(path, offset) -> Option<(TranscriptStats, u64)>` — incremental tail парс; вторая пара — новый `last_parsed_offset`.
3. `merge_stats(prev, tail) -> TranscriptStats` — ассоциативное слияние двух статов; `last_assistant`/`last_thinking_effort`/`session_last_at_ms` берутся из `tail` если есть, иначе из `prev`; `session_started_at_ms` — из `prev` если есть, иначе `tail`. `blocks` склеиваются на границе.
4. `parse_iso_to_ms(s) -> Option<u64>` — ISO-8601 в Unix ms, поддержка форматов с/без миллисекунд / `Z` / `+00:00` / `+0000`.
5. `floor_5h(ms) -> u64` — `(ms / 5h_ms) * 5h_ms`. Деление u64, никаких float.
6. Битые JSON-строки и неparseable timestamp'ы skip'ятся; `offset` всегда продвигается.
7. `apply_entry` switch'ит по `kind`: user → запоминает `last_user_ts`; assistant → накапливает usage, обновляет `last_assistant`, `current_block`, `last_thinking_effort`; system/tool/прочее → ignore.
8. Финальный `into_stats` push'ит residual `current_block` если есть.

## Inputs

- T1, T2 закрыты.
- `cache::fixture::TranscriptBuilder` доступен в тестах.
- `time = "0.3"` с feature `parsing` — для `parse_iso_to_ms`.

---

- [ ] **Step 1: Заменить `src/cache/parser.rs` полным парсером**

Replace `/Users/igor/mp/startup/cchud/src/cache/parser.rs` (T1 stub) полностью:

```rust
//! JSONL transcript parser — Phase 6 Task 3.
//!
//! - `parse_transcript(path)` — full parse от offset 0.
//! - `parse_from_offset(path, start)` — incremental tail парс; возвращает
//!   `(stats, new_offset)`.
//! - `merge_stats(prev, tail)` — ассоциативное слияние двух статов; tail
//!   побеждает на `last_*` полях, sums суммируются, blocks склеиваются на
//!   границе (последний block.prev может совпадать start с первым block.tail).
//! - `parse_iso_to_ms(s)` — ISO-8601 → Unix ms; 4 формата с/без ms / Z / +00:00.
//! - `floor_5h(ms) = (ms / 5h_ms) * 5h_ms` — детерминистичная сетка для
//!   block_start_ms (см. spec § Decision 9).
//!
//! Никаких unwrap/expect — все error-path → пропуск строки или None.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

use crate::cache::jsonl_types::{
    BillingBlock, MessagePayload, MessageStats, ThinkingMeta, TranscriptEntry, TranscriptStats,
    Usage,
};

const FIVE_HOURS_MS: u64 = 5 * 3600 * 1000;

/// Полный парс от начала файла.
#[must_use]
pub fn parse_transcript(path: &Path) -> Option<TranscriptStats> {
    parse_from_offset(path, 0).map(|(stats, _)| stats)
}

/// Incremental парс от `start`. Возвращает `(stats_for_tail, new_offset)`
/// — где `new_offset` — позиция после последней успешно прочитанной строки.
/// Битые строки skip'аются, но offset продвигается всегда.
#[must_use]
pub fn parse_from_offset(path: &Path, start: u64) -> Option<(TranscriptStats, u64)> {
    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut state = ParseState::default();
    let mut line = String::new();
    let mut offset = start;

    loop {
        line.clear();
        let n = match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let trimmed = line.trim_end_matches(['\n', '\r']);
        // Пустая строка — skip без увеличения broken counter.
        if !trimmed.is_empty() {
            if let Ok(entry) = sonic_rs::from_str::<TranscriptEntry>(trimmed) {
                apply_entry(&mut state, entry);
            }
            // Битый JSON: silently skip; offset всё равно продвигается ниже.
        }
        offset = offset.saturating_add(n as u64);
    }

    Some((state.into_stats(), offset))
}

/// Ассоциативное слияние двух статов. tail побеждает на `last_*` полях.
#[must_use]
pub fn merge_stats(prev: TranscriptStats, tail: TranscriptStats) -> TranscriptStats {
    let session_started_at_ms = prev.session_started_at_ms.or(tail.session_started_at_ms);
    let session_last_at_ms = tail.session_last_at_ms.or(prev.session_last_at_ms);

    let mut blocks = prev.blocks;
    for tb in tail.blocks {
        if let Some(last) = blocks.last_mut() {
            // Тот же блок (детерминистичный floor_5h) → расширяем end если tail длиннее.
            if last.started_at_ms == tb.started_at_ms {
                if tb.ends_at_ms > last.ends_at_ms {
                    last.ends_at_ms = tb.ends_at_ms;
                }
                continue;
            }
        }
        blocks.push(tb);
    }

    let last_assistant = tail.last_assistant.or(prev.last_assistant);
    let last_thinking_effort = tail
        .last_thinking_effort
        .clone()
        .or(prev.last_thinking_effort);

    TranscriptStats {
        session_started_at_ms,
        session_last_at_ms,
        messages: prev.messages.saturating_add(tail.messages),
        tokens_in_total: prev.tokens_in_total.saturating_add(tail.tokens_in_total),
        tokens_out_total: prev.tokens_out_total.saturating_add(tail.tokens_out_total),
        tokens_cache_read_total: prev
            .tokens_cache_read_total
            .saturating_add(tail.tokens_cache_read_total),
        tokens_cache_creation_total: prev
            .tokens_cache_creation_total
            .saturating_add(tail.tokens_cache_creation_total),
        last_assistant,
        blocks,
        last_thinking_effort,
    }
}

/// ISO-8601 → Unix ms. Поддержка:
/// - `2026-04-28T10:30:00Z`
/// - `2026-04-28T10:30:00.123Z`
/// - `2026-04-28T10:30:00+00:00`
/// - `2026-04-28T10:30:00.123+00:00`
#[must_use]
pub fn parse_iso_to_ms(s: &str) -> Option<u64> {
    use time::format_description::well_known::Iso8601;
    use time::OffsetDateTime;

    let dt = OffsetDateTime::parse(s, &Iso8601::DEFAULT).ok()?;
    let ts_nanos = dt.unix_timestamp_nanos();
    if ts_nanos < 0 {
        return None;
    }
    u64::try_from(ts_nanos / 1_000_000).ok()
}

/// `(ms / 5h_ms) * 5h_ms` — нижняя граница 5-часового billing-окна.
#[must_use]
pub const fn floor_5h(ms: u64) -> u64 {
    (ms / FIVE_HOURS_MS) * FIVE_HOURS_MS
}

// ──────────────────── Internal state ────────────────────

#[derive(Default)]
struct ParseState {
    last_user_ts: Option<u64>,
    current_block: Option<BillingBlock>,
    stats: TranscriptStats,
}

impl ParseState {
    fn into_stats(mut self) -> TranscriptStats {
        if let Some(b) = self.current_block.take() {
            self.stats.blocks.push(b);
        }
        self.stats
    }
}

fn apply_entry(state: &mut ParseState, entry: TranscriptEntry) {
    let kind = entry.kind.as_deref().unwrap_or("");
    let ts = entry.timestamp.as_deref().and_then(parse_iso_to_ms);

    match kind {
        "user" => {
            if let Some(ts) = ts {
                state.last_user_ts = Some(ts);
                update_session_bounds(&mut state.stats, ts);
            }
            state.stats.messages = state.stats.messages.saturating_add(1);
        }
        "assistant" => {
            apply_assistant(state, entry, ts);
        }
        _ => {
            // system / tool / etc. — игнорируем; messages не увеличиваем.
        }
    }
}

fn apply_assistant(
    state: &mut ParseState,
    entry: TranscriptEntry,
    ts: Option<u64>,
) {
    state.stats.messages = state.stats.messages.saturating_add(1);

    let Some(ts) = ts else { return };
    update_session_bounds(&mut state.stats, ts);

    if let Some(MessagePayload { usage: Some(u), .. }) = entry.message {
        accumulate_usage(state, u, ts);
    }
    advance_block(state, ts);

    if let Some(ThinkingMeta { effort: Some(level) }) = entry.thinking {
        if !level.is_empty() {
            state.stats.last_thinking_effort = Some(level);
        }
    }
}

fn accumulate_usage(state: &mut ParseState, u: Usage, ts_completed: u64) {
    let in_t = u.input_tokens.unwrap_or(0);
    let out_t = u.output_tokens.unwrap_or(0);
    let cache_r = u.cache_read_input_tokens.unwrap_or(0);
    let cache_c = u.cache_creation_input_tokens.unwrap_or(0);

    state.stats.tokens_in_total = state.stats.tokens_in_total.saturating_add(in_t);
    state.stats.tokens_out_total = state.stats.tokens_out_total.saturating_add(out_t);
    state.stats.tokens_cache_read_total = state
        .stats
        .tokens_cache_read_total
        .saturating_add(cache_r);
    state.stats.tokens_cache_creation_total = state
        .stats
        .tokens_cache_creation_total
        .saturating_add(cache_c);

    let started = state.last_user_ts.unwrap_or(ts_completed);
    state.stats.last_assistant = Some(MessageStats {
        started_at_ms: started,
        completed_at_ms: ts_completed,
        tokens_in: in_t.saturating_add(cache_r).saturating_add(cache_c),
        tokens_out: out_t,
    });
}

fn advance_block(state: &mut ParseState, ts: u64) {
    let block_start = floor_5h(ts);
    let block_end = block_start.saturating_add(FIVE_HOURS_MS);

    match state.current_block {
        Some(b) if b.started_at_ms == block_start => {
            // тот же блок — оставляем как есть.
        }
        Some(b) => {
            state.stats.blocks.push(b);
            state.current_block = Some(BillingBlock {
                started_at_ms: block_start,
                ends_at_ms: block_end,
            });
        }
        None => {
            state.current_block = Some(BillingBlock {
                started_at_ms: block_start,
                ends_at_ms: block_end,
            });
        }
    }
}

fn update_session_bounds(stats: &mut TranscriptStats, ts: u64) {
    if stats.session_started_at_ms.is_none() {
        stats.session_started_at_ms = Some(ts);
    }
    stats.session_last_at_ms = Some(ts);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::fixture::TranscriptBuilder;

    #[test]
    fn parse_iso_z_no_millis() {
        let ms = parse_iso_to_ms("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(ms, 1_767_225_600_000);
    }

    #[test]
    fn parse_iso_z_with_millis() {
        let ms = parse_iso_to_ms("2026-01-01T00:00:00.500Z").unwrap();
        assert_eq!(ms, 1_767_225_600_500);
    }

    #[test]
    fn parse_iso_offset_no_millis() {
        let ms = parse_iso_to_ms("2026-01-01T00:00:00+00:00").unwrap();
        assert_eq!(ms, 1_767_225_600_000);
    }

    #[test]
    fn parse_iso_offset_with_millis() {
        let ms = parse_iso_to_ms("2026-01-01T00:00:00.250+00:00").unwrap();
        assert_eq!(ms, 1_767_225_600_250);
    }

    #[test]
    fn parse_iso_invalid_returns_none() {
        assert!(parse_iso_to_ms("not a date").is_none());
        assert!(parse_iso_to_ms("").is_none());
    }

    #[test]
    fn floor_5h_at_zero() {
        assert_eq!(floor_5h(0), 0);
    }

    #[test]
    fn floor_5h_at_boundary() {
        assert_eq!(floor_5h(FIVE_HOURS_MS), FIVE_HOURS_MS);
        assert_eq!(floor_5h(FIVE_HOURS_MS - 1), 0);
    }

    #[test]
    fn floor_5h_two_windows() {
        let ts = 2 * FIVE_HOURS_MS + 12_345;
        assert_eq!(floor_5h(ts), 2 * FIVE_HOURS_MS);
    }

    #[test]
    fn parse_empty_file_yields_default_stats() {
        let b = TranscriptBuilder::new();
        let (stats, off) = parse_from_offset(b.path(), 0).unwrap();
        assert_eq!(stats, TranscriptStats::default());
        assert_eq!(off, 0);
    }

    #[test]
    fn parse_one_user_one_assistant() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(500, 100, 50, 200, 10, None);
        let stats = parse_transcript(b.path()).unwrap();
        assert_eq!(stats.messages, 2);
        assert_eq!(stats.tokens_in_total, 100);
        assert_eq!(stats.tokens_out_total, 50);
        assert_eq!(stats.tokens_cache_read_total, 200);
        assert_eq!(stats.tokens_cache_creation_total, 10);
        let last = stats.last_assistant.unwrap();
        // tokens_in включает input + cache_read + cache_creation.
        assert_eq!(last.tokens_in, 100 + 200 + 10);
        assert_eq!(last.tokens_out, 50);
        // session bounds выставлены.
        assert!(stats.session_started_at_ms.is_some());
        assert!(stats.session_last_at_ms.is_some());
        // ровно один block.
        assert_eq!(stats.blocks.len(), 1);
    }

    #[test]
    fn parse_skips_broken_lines() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.raw("garbled }}}{ not json");
        b.add_assistant(100, 5, 5, 0, 0, None);
        let stats = parse_transcript(b.path()).unwrap();
        // 1 user + 1 assistant; garbage не считается.
        assert_eq!(stats.messages, 2);
        assert_eq!(stats.tokens_in_total, 5);
    }

    #[test]
    fn parse_assistant_without_usage_still_updates_messages() {
        let b = TranscriptBuilder::new();
        b.raw(r#"{"type":"assistant","timestamp":"2026-01-01T00:00:00Z"}"#);
        let stats = parse_transcript(b.path()).unwrap();
        assert_eq!(stats.messages, 1);
        assert_eq!(stats.tokens_in_total, 0);
        assert!(stats.last_assistant.is_none());
    }

    #[test]
    fn thinking_effort_captured_for_all_levels() {
        for level in ["low", "medium", "high", "xhigh", "max"] {
            let mut b = TranscriptBuilder::new();
            b.add_user();
            b.add_assistant(100, 1, 1, 0, 0, Some(level));
            let stats = parse_transcript(b.path()).unwrap();
            assert_eq!(stats.last_thinking_effort.as_deref(), Some(level));
        }
    }

    #[test]
    fn block_boundary_detected_at_5h() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 1, 1, 0, 0, None);
        b.advance(5 * 3600 * 1000); // ровно через 5h
        b.add_user();
        b.add_assistant(100, 1, 1, 0, 0, None);
        let stats = parse_transcript(b.path()).unwrap();
        assert_eq!(
            stats.blocks.len(),
            2,
            "сообщения на границе 5h должны быть в разных блоках"
        );
    }

    #[test]
    fn merge_stats_associative() {
        let mut a = TranscriptBuilder::new();
        a.add_user();
        a.add_assistant(100, 10, 10, 0, 0, None);
        let stats_a = parse_transcript(a.path()).unwrap();

        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 20, 20, 0, 0, None);
        let stats_b = parse_transcript(b.path()).unwrap();

        let mut c = TranscriptBuilder::new();
        c.add_user();
        c.add_assistant(100, 30, 30, 0, 0, None);
        let stats_c = parse_transcript(c.path()).unwrap();

        let left = merge_stats(merge_stats(stats_a.clone(), stats_b.clone()), stats_c.clone());
        let right = merge_stats(stats_a, merge_stats(stats_b, stats_c));
        assert_eq!(left.tokens_in_total, right.tokens_in_total);
        assert_eq!(left.tokens_out_total, right.tokens_out_total);
        assert_eq!(left.messages, right.messages);
    }

    #[test]
    fn merge_stats_tail_wins_on_last_assistant_and_thinking() {
        let prev = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 1,
                completed_at_ms: 2,
                tokens_in: 10,
                tokens_out: 5,
            }),
            last_thinking_effort: Some("low".into()),
            ..TranscriptStats::default()
        };
        let tail = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 100,
                completed_at_ms: 200,
                tokens_in: 1000,
                tokens_out: 500,
            }),
            last_thinking_effort: Some("max".into()),
            ..TranscriptStats::default()
        };
        let merged = merge_stats(prev, tail);
        assert_eq!(merged.last_assistant.unwrap().completed_at_ms, 200);
        assert_eq!(merged.last_thinking_effort.as_deref(), Some("max"));
    }

    #[test]
    fn merge_stats_tail_keeps_prev_when_tail_empty() {
        let prev = TranscriptStats {
            last_thinking_effort: Some("high".into()),
            session_started_at_ms: Some(10),
            ..TranscriptStats::default()
        };
        let tail = TranscriptStats::default();
        let merged = merge_stats(prev, tail);
        assert_eq!(merged.last_thinking_effort.as_deref(), Some("high"));
        assert_eq!(merged.session_started_at_ms, Some(10));
    }

    #[test]
    fn merge_stats_merges_blocks_at_boundary() {
        let block = BillingBlock {
            started_at_ms: 0,
            ends_at_ms: FIVE_HOURS_MS,
        };
        let prev = TranscriptStats {
            blocks: vec![block],
            ..TranscriptStats::default()
        };
        let tail = TranscriptStats {
            blocks: vec![block, BillingBlock {
                started_at_ms: FIVE_HOURS_MS,
                ends_at_ms: 2 * FIVE_HOURS_MS,
            }],
            ..TranscriptStats::default()
        };
        let merged = merge_stats(prev, tail);
        // Дублирующийся block не плодится, второй приклеивается.
        assert_eq!(merged.blocks.len(), 2);
        assert_eq!(merged.blocks[0].started_at_ms, 0);
        assert_eq!(merged.blocks[1].started_at_ms, FIVE_HOURS_MS);
    }

    #[test]
    fn parse_from_offset_resumes_after_first_line() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        let user_size = b.size();
        b.add_assistant(100, 7, 3, 0, 0, None);
        let (tail, _) = parse_from_offset(b.path(), user_size).unwrap();
        // user — пропущен; в tail только assistant.
        assert_eq!(tail.messages, 1);
        assert_eq!(tail.tokens_in_total, 7);
    }

    #[test]
    fn parse_partial_last_line_skipped() {
        let b = TranscriptBuilder::new();
        // Нет trailing newline → строка незавершённая.
        std::fs::write(
            b.path(),
            r#"{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
{"type":"assistant","timestamp":"2026-01"#, // обрыв
        )
        .unwrap();
        let stats = parse_transcript(b.path()).unwrap();
        // Первая строка распарсилась, вторая — broken JSON, skipped.
        assert_eq!(stats.messages, 1);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/cache/parser.rs`.

> **Why `time` crate с `Iso8601::DEFAULT`:** `time` поддерживает все 4 формата нативно; не нужен ручной parse. `unix_timestamp_nanos() / 1_000_000` даёт ms точно; отрицательные timestamps (до 1970) → None (CC такого не пишет).

> **Why partial-tail tested не через TranscriptBuilder.advance():** TranscriptBuilder всегда добавляет `\n`. Для partial-tail берём `std::fs::write` напрямую — точно контролируем отсутствие newline.

- [ ] **Step 2: Запустить parser tests**

```bash
cargo test --locked cache::parser 2>&1 | tail -30
```

Expected: 18+ тестов passed. Если sonic-rs тяжело компилируется — первый запуск может занять минуту.

- [ ] **Step 3: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 4: Verification — task-specific**

```bash
grep -c 'pub fn parse_transcript' src/cache/parser.rs
grep -c 'pub fn parse_from_offset' src/cache/parser.rs
grep -c 'pub fn merge_stats' src/cache/parser.rs
grep -c 'pub fn parse_iso_to_ms' src/cache/parser.rs
grep -c 'pub const fn floor_5h' src/cache/parser.rs
grep -cE 'unwrap\(\)|expect\(' src/cache/parser.rs
```

Expected:
```
1
1
1
1
1
0     (никаких unwrap/expect в production-коде; тесты в `mod tests` имеют свой allow)
```

> **Note:** `grep -cE 'unwrap\(\)' src/cache/parser.rs` посчитает и в `mod tests`. Альтернатива:
> ```bash
> awk '/^#\[cfg\(test\)\]/{exit} {print}' src/cache/parser.rs | grep -cE 'unwrap\(\)|expect\(' 
> ```
> Должно быть 0.

- [ ] **Step 5: Commit**

```bash
git add src/cache/parser.rs
git commit -m "feat(phase-6): T3 parser — JSONL → TranscriptStats incremental

Replaces T1 stub with full parser:
- parse_transcript / parse_from_offset over BufReader, sonic-rs построчно
- merge_stats associative, tail-wins on last_assistant / last_thinking_effort,
  blocks deduped at 5h boundary
- parse_iso_to_ms via time::Iso8601::DEFAULT (Z, +00:00, with/without ms)
- floor_5h deterministic 5h grid

Internal ParseState:
- user → captures last_user_ts for next assistant's started_at
- assistant → accumulates tokens, updates last_assistant, advances current_block,
  captures thinking.effort if present
- system/tool/unknown → silent ignore (messages не считаются)

Broken JSONL lines silently skipped; offset always advances. Partial last
line: broken JSON → skip → last_parsed_offset фиксирует место обрыва.

Task 3/10 of Phase 6. T4 wires this into bincode disk cache."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked cache::parser
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `parse_transcript`, `parse_from_offset`, `merge_stats`, `parse_iso_to_ms`, `floor_5h` все `pub`
- [ ] `apply_entry` switch по `kind` ("user" / "assistant" / `_`); user/assistant обновляют `messages`, остальные — нет
- [ ] `merge_stats` ассоциативный; `last_*` поля → tail-wins; sums суммируются с `saturating_add`; blocks дедуплицируются по `started_at_ms`
- [ ] `parse_iso_to_ms` поддерживает 4 формата; невалидный → None
- [ ] `floor_5h` детерминирован; ровно на границе → новое окно (boundary inclusive lower, exclusive upper)
- [ ] Битые JSONL-строки skip'ятся, offset продвигается на байты строки
- [ ] `cargo test cache::parser` зелёный (≥18 тестов)
- [ ] Никаких `unwrap`/`expect` в production-коде (только `#[allow(...)]` в `mod tests`)
- [ ] Один commit `feat(phase-6): T3 parser ...`

## Files touched

- `src/cache/parser.rs` (rewritten)

## Risks & rollback

- **sonic-rs не парсит CC-формат**: маловероятно (CC использует стандартный JSON). Митигация: если в T3 unit-тесты падают на парсе — попробовать `serde_json::from_str` параллельно для контрольной точки; если sonic-rs действительно не справляется — `cfg(windows)` fallback расширяется на все платформы.
- **`time` парсер строгий**: `Iso8601::DEFAULT` принимает RFC 3339-совместимые форматы. Если CC использует нестандартный (типа `2026-04-28 10:30:00`) — добавить кастомный `format_description!`. Покрыто unit-тестом.
- **Block-boundary fence-post**: `floor_5h(FIVE_HOURS_MS) = FIVE_HOURS_MS` — точка ровно на границе принадлежит новому окну. Это и есть желаемое поведение (см. spec § Decision 9).
- **`merge_stats` не коммутативен**: tail-wins на `last_*` означает, что `merge(a, b) != merge(b, a)`. Это by design — `tail` всегда хронологически новее `prev`. Тест `merge_stats_associative` проверяет ассоциативность (что нам нужно для T4 incremental merge), не коммутативность.
- **Rollback**: `git revert HEAD` восстанавливает T2-состояние (T1 stub).
