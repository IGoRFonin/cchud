//! JSONL transcript parser.
//!
//! - `parse_from_offset(path, start)` — incremental tail парс; возвращает
//!   `(stats, new_offset)`.
//! - `merge_stats(prev, tail)` — ассоциативное слияние двух статов; tail
//!   побеждает на `last_*` полях, sums суммируются, blocks склеиваются на
//!   границе (последний block.prev может совпадать start с первым block.tail).
//! - `parse_iso_to_ms(s)` — ISO-8601 → Unix ms; 4 формата с/без ms / Z / +00:00.
//! - `floor_5h(ms) = (ms / 5h_ms) * 5h_ms` — детерминистичная сетка для
//!   `block_start_ms` (см. spec § Decision 9).
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
            Ok(0) | Err(_) => break,
            Ok(n) => n,
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

    let mut skill_names = prev.skill_names;
    skill_names.extend(tail.skill_names);
    skill_names.sort_unstable();
    skill_names.dedup();

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
        skill_names,
    }
}

/// ISO-8601 → Unix ms. Поддержка любого UTC-offset (не только Z/+00:00):
/// - `2026-04-28T10:30:00Z`
/// - `2026-04-28T10:30:00.123Z`
/// - `2026-04-28T10:30:00+00:00`
/// - `2026-04-28T10:30:00.123+00:00`
/// - `2026-04-28T10:30:00+05:30`
/// - `2026-04-28T10:30:00-07:00`
#[must_use]
pub fn parse_iso_to_ms(s: &str) -> Option<u64> {
    use time::OffsetDateTime;
    use time::format_description::well_known::Iso8601;

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
    skill_set: std::collections::BTreeSet<String>,
}

impl ParseState {
    fn into_stats(mut self) -> TranscriptStats {
        if let Some(b) = self.current_block.take() {
            self.stats.blocks.push(b);
        }
        self.stats.skill_names = self.skill_set.into_iter().collect();
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

fn apply_assistant(state: &mut ParseState, entry: TranscriptEntry, ts: Option<u64>) {
    state.stats.messages = state.stats.messages.saturating_add(1);

    let Some(ts) = ts else { return };
    update_session_bounds(&mut state.stats, ts);

    if let Some(content) = entry
        .message
        .as_ref()
        .and_then(|m| m.content.as_ref())
        .and_then(serde_json::Value::as_array)
    {
        for block in content {
            if block.get("type").and_then(serde_json::Value::as_str) != Some("tool_use") {
                continue;
            }
            if let Some(name) = block.get("name").and_then(serde_json::Value::as_str) {
                if let Some(skill) = name.strip_prefix("skill_") {
                    state.skill_set.insert(skill.to_string());
                }
            }
        }
    }

    if let Some(MessagePayload { usage: Some(u), .. }) = entry.message {
        accumulate_usage(state, u, ts);
    }
    advance_block(state, ts);

    if let Some(ThinkingMeta {
        effort: Some(level),
    }) = entry.thinking
    {
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
    state.stats.tokens_cache_read_total =
        state.stats.tokens_cache_read_total.saturating_add(cache_r);
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

const fn update_session_bounds(stats: &mut TranscriptStats, ts: u64) {
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

    fn parse_transcript(path: &std::path::Path) -> Option<TranscriptStats> {
        parse_from_offset(path, 0).map(|(stats, _)| stats)
    }

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
    fn parse_iso_format_without_colon_in_offset() {
        // time::Iso8601::DEFAULT принимает +0000 (без двоеточия) — формат поддерживается.
        let ms = parse_iso_to_ms("2026-01-01T00:00:00+0000").unwrap();
        assert_eq!(ms, 1_767_225_600_000);
        let ms2 = parse_iso_to_ms("2026-01-01T00:00:00.500+0000").unwrap();
        assert_eq!(ms2, 1_767_225_600_500);
    }

    #[test]
    fn parse_iso_non_zero_offset() {
        // +05:30 → UTC 2026-01-01T00:00:00Z
        assert_eq!(
            parse_iso_to_ms("2026-01-01T05:30:00+05:30").unwrap(),
            1_767_225_600_000
        );
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
        // a и b в одном 5h-окне, c — через 5h (другое окно).
        let mut a = TranscriptBuilder::new();
        a.add_user();
        a.add_assistant(100, 10, 10, 4, 2, None);
        let stats_a = parse_transcript(a.path()).unwrap();

        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 20, 20, 6, 3, None);
        let stats_b = parse_transcript(b.path()).unwrap();

        let mut c = TranscriptBuilder::new();
        c.add_user();
        c.advance(5 * 3600 * 1000); // перешагнуть в следующее 5h-окно
        c.add_assistant(100, 30, 30, 8, 5, None);
        let stats_c = parse_transcript(c.path()).unwrap();

        let left = merge_stats(
            merge_stats(stats_a.clone(), stats_b.clone()),
            stats_c.clone(),
        );
        let right = merge_stats(stats_a, merge_stats(stats_b, stats_c));
        assert_eq!(left.tokens_in_total, right.tokens_in_total);
        assert_eq!(left.tokens_out_total, right.tokens_out_total);
        assert_eq!(left.tokens_cache_read_total, right.tokens_cache_read_total);
        assert_eq!(
            left.tokens_cache_creation_total,
            right.tokens_cache_creation_total
        );
        assert_eq!(left.messages, right.messages);
        // Значения: 10+20+30, 10+20+30, 4+6+8, 2+3+5
        assert_eq!(left.tokens_in_total, 60);
        assert_eq!(left.tokens_out_total, 60);
        assert_eq!(left.tokens_cache_read_total, 18);
        assert_eq!(left.tokens_cache_creation_total, 10);
        // c в другом 5h-окне → 2 блока с обеих сторон.
        assert_eq!(left.blocks.len(), 2);
        assert_eq!(right.blocks.len(), 2);
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
            blocks: vec![
                block,
                BillingBlock {
                    started_at_ms: FIVE_HOURS_MS,
                    ends_at_ms: 2 * FIVE_HOURS_MS,
                },
            ],
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
    fn parse_assistant_with_content_array_counts_tokens() {
        // Real CC transcripts include "content":[...] in assistant messages.
        // Verifies sonic_rs deserialization doesn't choke on the content array,
        // which would silently drop the line and zero out token counts.
        let b = TranscriptBuilder::new();
        b.raw(concat!(
            r#"{"type":"assistant","timestamp":"2026-01-01T00:00:01Z","message":"#,
            r#"{"usage":{"input_tokens":42,"output_tokens":17},"#,
            r#""content":[{"type":"text","text":"hello world"},{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"ls"}}]}}"#
        ));
        let stats = parse_transcript(b.path()).unwrap();
        assert_eq!(stats.tokens_in_total, 42);
        assert_eq!(stats.tokens_out_total, 17);
    }

    #[test]
    fn parses_assistant_skill_invocations_into_skill_names() {
        let mut b = TranscriptBuilder::new();
        b.add_assistant_with_tool_uses(
            "2026-04-29T10:00:00Z",
            &[
                ("skill_brainstorming", "{}"),
                ("read_file", "{}"),
                ("skill_executing-plans", "{}"),
            ],
        );
        b.add_assistant_with_tool_uses("2026-04-29T10:01:00Z", &[("skill_brainstorming", "{}")]);
        let stats = parse_transcript(b.path()).unwrap();
        assert_eq!(
            stats.skill_names.len(),
            2,
            "dedup'd: {:?}",
            stats.skill_names
        );
        assert!(stats.skill_names.contains(&"brainstorming".to_string()));
        assert!(stats.skill_names.contains(&"executing-plans".to_string()));
        let mut sorted = stats.skill_names.clone();
        sorted.sort();
        assert_eq!(stats.skill_names, sorted);
    }

    #[test]
    fn merge_stats_unions_skill_names_sorted_dedup() {
        let a = TranscriptStats {
            skill_names: vec!["alpha".into(), "gamma".into()],
            ..Default::default()
        };
        let b = TranscriptStats {
            skill_names: vec!["beta".into(), "alpha".into()],
            ..Default::default()
        };
        let merged = merge_stats(a, b);
        assert_eq!(
            merged.skill_names,
            vec!["alpha".to_string(), "beta".into(), "gamma".into()]
        );
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
