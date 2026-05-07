# Task 2 — JSONL types: TranscriptEntry, TranscriptStats, BillingBlock, CacheFile

**Files:**
- Modify: `src/cache/jsonl_types.rs` (заменяет T1 stub полным набором типов)

## Goal

Pure-data layer кэша. Только struct'ы + serde + bincode. Никакой логики, никаких IO.

Контракт T2:
1. `TranscriptEntry` / `MessagePayload` / `Usage` / `ThinkingMeta` — `Deserialize` для read JSONL.
2. `TranscriptStats` / `MessageStats` / `BillingBlock` — `Serialize + Deserialize` (read JSONL → bincode dump → bincode read обратно).
3. `CacheMeta` / `CacheFile` — обёртка для bincode IO с `format_version`.
4. `FORMAT_VERSION = 1` — константа, увеличиваем при breaking schema change.
5. Bincode round-trip работает: `decode(encode(x)) == x` для всех Serialize-типов.
6. Mismatch `format_version` детектится: `read_cache` (T4) проверит, но даже на уровне типов мы проверяем что mismatch виден через `CacheMeta.format_version`.

## Inputs

- T1 закрыт; `src/cache/mod.rs` re-exports работают; пустые stubs компилируются.

---

- [ ] **Step 1: Заменить `src/cache/jsonl_types.rs` полным набором типов**

Replace `/Users/igor/mp/startup/cchud/src/cache/jsonl_types.rs` (T1 stub) полностью:

```rust
//! JSONL transcript types and bincode cache layout — Phase 6 Task 2.
//!
//! Двунаправленные типы:
//! - `TranscriptEntry`/`MessagePayload`/`Usage`/`ThinkingMeta` — read-only,
//!   соответствуют формату записей CC в `~/.claude/projects/.../transcript.jsonl`.
//!   Все поля `Option<...>`, потому что CC изменяет shape между версиями.
//! - `TranscriptStats`/`MessageStats`/`BillingBlock` — модель агрегатов,
//!   которую виджеты читают и которая сериализуется в bincode-кэш.
//! - `CacheMeta`/`CacheFile` — обёртка для disk format с `format_version`.
//!
//! `FORMAT_VERSION` увеличиваем при любом breaking change в layout
//! `TranscriptStats`/`MessageStats`/`BillingBlock`/`CacheMeta`. Mismatch →
//! silent reset кэша (T4).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};

/// Текущая версия on-disk схемы. T4 при mismatch делает full rebuild.
pub const FORMAT_VERSION: u32 = 1;

// ───────────────────── Wire types (read JSONL) ─────────────────────

#[derive(Debug, Deserialize)]
pub struct TranscriptEntry {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub timestamp: Option<String>,
    pub message: Option<MessagePayload>,
    pub thinking: Option<ThinkingMeta>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    pub usage: Option<Usage>,
    /// Не парсим в Phase 6 — Phase 7 (Skills widget) пройдётся по content.
    /// `serde_json::Value` — heap, но мы не вытаскиваем его за пределы парсера.
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ThinkingMeta {
    pub effort: Option<String>,
}

// ───────────────────── Aggregate model (read by widgets, on-disk via bincode) ─────────────────────

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptStats {
    pub session_started_at_ms: Option<u64>,
    pub session_last_at_ms: Option<u64>,
    pub messages: u32,

    pub tokens_in_total: u64,
    pub tokens_out_total: u64,
    pub tokens_cache_read_total: u64,
    pub tokens_cache_creation_total: u64,

    pub last_assistant: Option<MessageStats>,
    pub blocks: Vec<BillingBlock>,
    pub last_thinking_effort: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageStats {
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BillingBlock {
    pub started_at_ms: u64,
    pub ends_at_ms: u64,
}

// ───────────────────── Cache file layout ─────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheMeta {
    pub format_version: u32,
    pub source_size: u64,
    pub source_mtime_ns: u128,
    pub last_parsed_offset: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub meta: CacheMeta,
    pub stats: TranscriptStats,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn format_version_constant_is_one() {
        assert_eq!(FORMAT_VERSION, 1);
    }

    #[test]
    fn transcript_stats_default_is_empty() {
        let s = TranscriptStats::default();
        assert_eq!(s.messages, 0);
        assert_eq!(s.tokens_in_total, 0);
        assert!(s.last_assistant.is_none());
        assert!(s.blocks.is_empty());
        assert!(s.last_thinking_effort.is_none());
    }

    #[test]
    fn bincode_roundtrip_transcript_stats() {
        let original = TranscriptStats {
            session_started_at_ms: Some(1_000),
            session_last_at_ms: Some(5_000),
            messages: 4,
            tokens_in_total: 100,
            tokens_out_total: 200,
            tokens_cache_read_total: 300,
            tokens_cache_creation_total: 400,
            last_assistant: Some(MessageStats {
                started_at_ms: 1_000,
                completed_at_ms: 5_000,
                tokens_in: 50,
                tokens_out: 100,
            }),
            blocks: vec![BillingBlock {
                started_at_ms: 0,
                ends_at_ms: 18_000_000,
            }],
            last_thinking_effort: Some("high".into()),
        };
        let bytes = bincode::serialize(&original).unwrap();
        let back: TranscriptStats = bincode::deserialize(&bytes).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn bincode_roundtrip_cache_file() {
        let original = CacheFile {
            meta: CacheMeta {
                format_version: FORMAT_VERSION,
                source_size: 1024,
                source_mtime_ns: 9_999_999_999,
                last_parsed_offset: 512,
            },
            stats: TranscriptStats::default(),
        };
        let bytes = bincode::serialize(&original).unwrap();
        let back: CacheFile = bincode::deserialize(&bytes).unwrap();
        assert_eq!(back.meta, original.meta);
        assert_eq!(back.stats, original.stats);
    }

    #[test]
    fn cache_meta_format_version_mismatch_detectable() {
        let bytes = bincode::serialize(&CacheMeta {
            format_version: 999,
            source_size: 0,
            source_mtime_ns: 0,
            last_parsed_offset: 0,
        })
        .unwrap();
        let back: CacheMeta = bincode::deserialize(&bytes).unwrap();
        assert_ne!(
            back.format_version, FORMAT_VERSION,
            "mismatch must remain visible after roundtrip"
        );
    }

    #[test]
    fn transcript_entry_parses_minimal_user() {
        let json = r#"{"type":"user","timestamp":"2026-01-01T00:00:00Z"}"#;
        let entry: TranscriptEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.kind.as_deref(), Some("user"));
        assert_eq!(entry.timestamp.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert!(entry.message.is_none());
    }

    #[test]
    fn transcript_entry_parses_assistant_with_usage() {
        let json = r#"{
            "type":"assistant",
            "timestamp":"2026-01-01T00:01:00Z",
            "message":{
                "usage":{
                    "input_tokens":100,
                    "output_tokens":50,
                    "cache_read_input_tokens":200,
                    "cache_creation_input_tokens":10
                }
            },
            "thinking":{"effort":"high"}
        }"#;
        let entry: TranscriptEntry = serde_json::from_str(json).unwrap();
        let msg = entry.message.unwrap();
        let usage = msg.usage.unwrap();
        assert_eq!(usage.input_tokens, Some(100));
        assert_eq!(usage.output_tokens, Some(50));
        assert_eq!(usage.cache_read_input_tokens, Some(200));
        assert_eq!(usage.cache_creation_input_tokens, Some(10));
        assert_eq!(entry.thinking.unwrap().effort.as_deref(), Some("high"));
    }

    #[test]
    fn transcript_entry_unknown_fields_ignored() {
        let json = r#"{"type":"system","timestamp":"2026-01-01T00:00:00Z","extra":"ignored"}"#;
        let entry: TranscriptEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.kind.as_deref(), Some("system"));
    }

    #[test]
    fn usage_fields_all_optional() {
        let json = r#"{}"#;
        let u: Usage = serde_json::from_str(json).unwrap();
        assert!(u.input_tokens.is_none());
        assert!(u.output_tokens.is_none());
    }

    #[test]
    fn message_stats_copy_semantics() {
        let m = MessageStats {
            started_at_ms: 1,
            completed_at_ms: 2,
            tokens_in: 3,
            tokens_out: 4,
        };
        let n = m; // Copy
        assert_eq!(m.tokens_in, n.tokens_in);
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/cache/jsonl_types.rs`.

> **Why `serde_json::Value` для `MessagePayload.content`:** Phase 6 не парсит content. В Phase 7 `Skills` widget пройдётся по этому полю. Мы не платим heap-аллок, потому что `content` всё равно отбрасывается после `apply_entry` в `parser.rs` (T3). Альтернатива — `Option<RawValue>`, но усложняет lifetime; `Value` проще и тестировать.

> **Why `time` crate здесь не нужен:** Этот файл — только pure data. Парсер ISO-8601 → `u64 ms` живёт в `parser.rs` (T3).

- [ ] **Step 2: Запустить тесты типов**

```bash
cargo test --locked cache::jsonl_types 2>&1 | tail -20
```

Expected: 9+ тестов passed.

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
grep -c 'pub struct TranscriptEntry' src/cache/jsonl_types.rs
grep -c 'pub struct TranscriptStats' src/cache/jsonl_types.rs
grep -c 'pub struct MessageStats' src/cache/jsonl_types.rs
grep -c 'pub struct BillingBlock' src/cache/jsonl_types.rs
grep -c 'pub struct CacheFile' src/cache/jsonl_types.rs
grep -c 'pub const FORMAT_VERSION' src/cache/jsonl_types.rs
```

Expected:
```
1
1
1
1
1
1
```

- [ ] **Step 5: Commit**

```bash
git add src/cache/jsonl_types.rs
git commit -m "feat(phase-6): T2 jsonl types — wire + aggregate + cache layout

Replaces T1 stub with full type system:
- Wire types (Deserialize): TranscriptEntry, MessagePayload, Usage,
  ThinkingMeta — match CC's transcript.jsonl entries.
- Aggregate model (Serialize+Deserialize): TranscriptStats, MessageStats,
  BillingBlock — read by widgets, persisted to bincode cache.
- Cache layout: CacheMeta, CacheFile with FORMAT_VERSION = 1; mismatch
  triggers silent reset in T4.

Pure data layer — no IO, no parsing logic. Bincode roundtrip verified
for all Serialize types. Wire types accept extra fields (CC may evolve
shape).

Task 2/10 of Phase 6. T3 implements parser on top of these types."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked cache::jsonl_types
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/cache/jsonl_types.rs` содержит 9 публичных типов: `TranscriptEntry`, `MessagePayload`, `Usage`, `ThinkingMeta`, `TranscriptStats`, `MessageStats`, `BillingBlock`, `CacheMeta`, `CacheFile`
- [ ] `FORMAT_VERSION = 1` объявлена как `pub const`
- [ ] `TranscriptStats`, `MessageStats`, `BillingBlock`, `CacheMeta` имеют `Serialize + Deserialize`; bincode round-trip покрыт unit-тестом
- [ ] `Usage` поля `Option<u64>` (CC может опускать поля)
- [ ] Wire-типы парсят минимальный user / полный assistant с usage / неизвестными полями (3 теста)
- [ ] `cargo test cache::jsonl_types` зелёный (≥9 тестов)
- [ ] Один commit `feat(phase-6): T2 jsonl types ...`

## Files touched

- `src/cache/jsonl_types.rs` (rewritten — заменяет T1 stub)

## Risks & rollback

- **bincode trailing-bytes ошибки**: bincode 1.x по умолчанию не толерирует trailing bytes. Митигация: при чтении на T4 будем использовать `bincode::deserialize_from(reader)`. Для round-trip теста используем `serialize` → `deserialize` через `Vec<u8>` — корректно.
- **CC меняет shape `Usage`**: новые поля → ignored (serde default). Удалённые поля → `Option::None`. Bincode mismatch → format_version bump, silent reset.
- **Rollback**: `git revert HEAD` восстанавливает T1 stub.
