//! JSONL transcript types and bincode cache layout.
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
//! silent reset кэша.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};

/// Текущая версия on-disk схемы. При mismatch делается full rebuild.
pub const FORMAT_VERSION: u32 = 3;

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
    /// Не парсим — будущий Skills widget пройдётся по content.
    #[serde(default)]
    #[allow(dead_code)]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
#[allow(clippy::struct_field_names)]
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
    pub skill_names: Vec<String>,

    /// Число пауз > 300s между соседними записями транскрипта (любого kind).
    /// Сбрасывается на /clear (контекст пуст → prior misses нерелевантны).
    pub cache_misses: u32,
    /// Первый ts любого entry в текущем chunk (для boundary-gap при merge).
    pub first_event_ts_ms: Option<u64>,
    /// Последний ts любого entry (regardless of kind) — для boundary-gap при merge.
    pub last_event_ts_ms: Option<u64>,
    /// ts последнего /clear, обнаруженного в этом chunk; None если clear не было.
    /// При merge: если `tail.last_clear_ts_ms.is_some()` → `prev.cache_misses` отбрасывается.
    pub last_clear_ts_ms: Option<u64>,
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
    /// Stored for diagnostic purposes; not used in cache invalidation logic.
    pub source_size: u64,
    pub source_mtime_ns: u128,
    pub last_parsed_offset: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheFile {
    pub meta: CacheMeta,
    pub stats: TranscriptStats,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn format_version_constant_is_three() {
        assert_eq!(FORMAT_VERSION, 3);
    }

    #[test]
    fn transcript_stats_default_has_empty_skill_names() {
        let s = TranscriptStats::default();
        assert!(s.skill_names.is_empty());
    }

    #[test]
    fn bincode_roundtrip_includes_skill_names() {
        let original = TranscriptStats {
            skill_names: vec!["brainstorming".into(), "executing-plans".into()],
            ..TranscriptStats::default()
        };
        let bytes = bincode::serialize(&original).unwrap();
        let back: TranscriptStats = bincode::deserialize(&bytes).unwrap();
        assert_eq!(back.skill_names, original.skill_names);
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
            skill_names: vec!["tdd".into()],
            cache_misses: 7,
            first_event_ts_ms: Some(1_000),
            last_event_ts_ms: Some(5_000),
            last_clear_ts_ms: Some(2_500),
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
        assert_eq!(back, original);
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
        let json = r"{}";
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
