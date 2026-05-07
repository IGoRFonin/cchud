//! Test fixture helpers.
//!
//! `TranscriptBuilder` создаёт tempfile-`.jsonl` транскрипты для тестов
//! парсера, store, виджетов. Каждый `.add_user`/`.add_assistant` пишет
//! одну JSONL-строку с типичной CC-формой записи.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::new_without_default,
    clippy::doc_markdown
)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Lightweight transcript-фикстура.
pub struct TranscriptBuilder {
    #[allow(dead_code)]
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

    /// Добавляет assistant-entry с tool_use блоками. Каждый блок: (tool_name, input_json_str).
    pub fn add_assistant_with_tool_uses(
        &mut self,
        ts: &str,
        tool_uses: &[(&str, &str)],
    ) -> &mut Self {
        let content: Vec<serde_json::Value> = tool_uses
            .iter()
            .map(|(name, input_json)| {
                serde_json::json!({
                    "type": "tool_use",
                    "name": name,
                    "input": serde_json::from_str::<serde_json::Value>(input_json)
                        .unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();
        let entry = serde_json::json!({
            "type": "assistant",
            "timestamp": ts,
            "message": { "content": content }
        });
        self.raw(&entry.to_string());
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
    use time::OffsetDateTime;
    let dt = OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000)
        .expect("ts within OffsetDateTime range");
    let sub_ms = ms % 1000;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        dt.year(),
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        sub_ms,
    )
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
