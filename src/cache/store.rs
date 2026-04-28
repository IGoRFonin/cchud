//! Disk-IO для transcript-кэша — Phase 6 Task 4.
//!
//! Public surface:
//! - [`load_or_build_incremental`] — единственный entry point
//!   виджетов Phase 6 (через `RenderContext::transcript()`).
//!
//! Стратегия:
//! - Hit-path: `cached.format_version == FORMAT_VERSION`
//!   `&& cached.last_parsed_offset <= src.size`
//!   `&& cached.source_mtime_ns <= src.mtime_ns`
//!   → [`parse_from_offset`] + [`merge_stats`].
//! - Miss-path: любая из проверок не прошла → full rebuild + write cache.
//!
//! Best-effort writes: [`write_cache_best_effort`] игнорирует все IO-ошибки;
//! следующий рендер пересчитает. Никаких panic / unwrap / expect.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::cache::jsonl_types::{CacheFile, CacheMeta, FORMAT_VERSION, TranscriptStats};
use crate::cache::parser::{merge_stats, parse_from_offset};

/// Главный API. None если транскрипт недоступен / битый / IO error.
#[must_use]
pub fn load_or_build_incremental(transcript_path: &Path) -> Option<TranscriptStats> {
    let cache_path = cache_path_for(transcript_path);
    let meta = fs::metadata(transcript_path).ok()?;
    let src_size = meta.len();
    let src_mtime_ns = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();

    if let Some(cached) = read_cache(&cache_path) {
        let m = &cached.meta;
        let incremental_ok = m.format_version == FORMAT_VERSION
            && m.last_parsed_offset <= src_size
            && m.source_mtime_ns <= src_mtime_ns;

        if incremental_ok {
            if let Some((tail, new_offset)) =
                parse_from_offset(transcript_path, m.last_parsed_offset)
            {
                let merged = merge_stats(cached.stats, tail);
                let _ = write_cache_best_effort(
                    &cache_path,
                    &CacheFile {
                        meta: CacheMeta {
                            format_version: FORMAT_VERSION,
                            source_size: src_size,
                            source_mtime_ns: src_mtime_ns,
                            last_parsed_offset: new_offset,
                        },
                        stats: merged.clone(),
                    },
                );
                return Some(merged);
            }
        }
    }

    // Cold path: full rebuild.
    let (stats, offset) = parse_from_offset(transcript_path, 0)?;
    let _ = write_cache_best_effort(
        &cache_path,
        &CacheFile {
            meta: CacheMeta {
                format_version: FORMAT_VERSION,
                source_size: src_size,
                source_mtime_ns: src_mtime_ns,
                last_parsed_offset: offset,
            },
            stats: stats.clone(),
        },
    );
    Some(stats)
}

/// `<cache_dir>/cchud/transcript-<hex16>.bincode`.
/// `SipHash24` от канонического абсолютного пути транскрипта.
#[must_use]
pub fn cache_path_for(transcript: &Path) -> PathBuf {
    let abs = fs::canonicalize(transcript).unwrap_or_else(|_| transcript.to_path_buf());
    let mut h = siphasher::sip::SipHasher24::new();
    h.write(abs.to_string_lossy().as_bytes());
    let hex = format!("{:016x}", h.finish());
    let dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.join("cchud").join(format!("transcript-{hex}.bincode"))
}

fn read_cache(path: &Path) -> Option<CacheFile> {
    let f = File::open(path).ok()?;
    let mut reader = BufReader::new(f);
    bincode::deserialize_from(&mut reader).ok()
}

/// Атомарно (best-effort): пишем во временный файл рядом и rename.
/// Любая ошибка → silent ignore (`Result::Err` игнорируется выше).
fn write_cache_best_effort(path: &Path, file: &CacheFile) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = with_extension_appended(path, ".tmp");
    {
        let f = File::create(&tmp)?;
        let mut writer = BufWriter::new(f);
        bincode::serialize_into(&mut writer, file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }
    fs::rename(&tmp, path)
}

fn with_extension_appended(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::fixture::TranscriptBuilder;

    fn force_cache_into_tempdir() -> tempfile::TempDir {
        // Этот helper не подменяет dirs::cache_dir(), но через `cache_path_for`
        // уникальный hash на каждый транскрипт-tempdir обеспечивает изоляцию.
        // Тесты T4 чистят cache_path_for(transcript_path) явно.
        tempfile::tempdir().unwrap()
    }

    fn cleanup_cache(transcript: &Path) {
        let _ = fs::remove_file(cache_path_for(transcript));
    }

    #[test]
    fn cache_path_includes_hex16() {
        let p = Path::new("/tmp/test-transcript.jsonl");
        let cp = cache_path_for(p);
        let name = cp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("transcript-"));
        assert!(name.ends_with(".bincode"));
        let hex = name
            .strip_prefix("transcript-")
            .unwrap()
            .strip_suffix(".bincode")
            .unwrap();
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cache_path_stable_for_same_input() {
        let p = Path::new("/tmp/stable-transcript.jsonl");
        assert_eq!(cache_path_for(p), cache_path_for(p));
    }

    #[test]
    fn cache_path_differs_for_different_inputs() {
        let a = cache_path_for(Path::new("/tmp/a.jsonl"));
        let b = cache_path_for(Path::new("/tmp/b.jsonl"));
        assert_ne!(a, b);
    }

    #[test]
    fn first_run_parses_full_and_writes_cache() {
        let _td = force_cache_into_tempdir();
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 7, 3, 0, 0, None);
        cleanup_cache(b.path());
        let stats = load_or_build_incremental(b.path()).unwrap();
        assert_eq!(stats.tokens_in_total, 7);
        assert!(cache_path_for(b.path()).exists());
        cleanup_cache(b.path());
    }

    #[test]
    fn second_run_no_changes_returns_same_stats() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 11, 22, 0, 0, None);
        cleanup_cache(b.path());
        let first = load_or_build_incremental(b.path()).unwrap();
        let second = load_or_build_incremental(b.path()).unwrap();
        assert_eq!(first.tokens_in_total, second.tokens_in_total);
        assert_eq!(first.tokens_out_total, second.tokens_out_total);
        cleanup_cache(b.path());
    }

    #[test]
    fn append_triggers_tail_merge() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 10, 10, 0, 0, None);
        cleanup_cache(b.path());
        let first = load_or_build_incremental(b.path()).unwrap();
        // Append second пара.
        b.add_user();
        b.add_assistant(100, 5, 5, 0, 0, None);
        // Чтобы mtime поменялся стабильно — touch.
        std::thread::sleep(std::time::Duration::from_millis(20));
        let _ = filetime::set_file_mtime(b.path(), filetime::FileTime::now());
        let second = load_or_build_incremental(b.path()).unwrap();
        assert_eq!(second.tokens_in_total, first.tokens_in_total + 5);
        assert_eq!(second.tokens_out_total, first.tokens_out_total + 5);
        cleanup_cache(b.path());
    }

    #[test]
    fn truncate_triggers_full_rebuild() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 100, 100, 0, 0, None);
        cleanup_cache(b.path());
        let _ = load_or_build_incremental(b.path()).unwrap();
        // Truncate to single small entry — size меньше last_parsed_offset.
        b.overwrite(
            r#"{"type":"user","timestamp":"2026-01-01T00:00:00Z"}
"#,
        );
        let after = load_or_build_incremental(b.path()).unwrap();
        // После truncate stats должен соответствовать только новому контенту.
        assert_eq!(after.messages, 1);
        assert_eq!(after.tokens_in_total, 0);
        cleanup_cache(b.path());
    }

    #[test]
    fn corrupt_cache_triggers_silent_reset() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 50, 25, 0, 0, None);
        cleanup_cache(b.path());
        // Записываем мусор в кэш-файл руками, минуя bincode.
        let cp = cache_path_for(b.path());
        if let Some(parent) = cp.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&cp, b"\x00\x01\x02 not bincode").unwrap();
        let stats = load_or_build_incremental(b.path()).unwrap();
        assert_eq!(stats.tokens_in_total, 50);
        cleanup_cache(b.path());
    }

    #[test]
    fn format_version_mismatch_triggers_silent_reset() {
        let mut b = TranscriptBuilder::new();
        b.add_user();
        b.add_assistant(100, 99, 0, 0, 0, None);
        cleanup_cache(b.path());
        // Создаём корректный bincode с form_version=999.
        let cp = cache_path_for(b.path());
        if let Some(parent) = cp.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let bogus = CacheFile {
            meta: CacheMeta {
                format_version: 999,
                source_size: 0,
                source_mtime_ns: 0,
                last_parsed_offset: 0,
            },
            stats: TranscriptStats::default(),
        };
        let bytes = bincode::serialize(&bogus).unwrap();
        fs::write(&cp, bytes).unwrap();
        let stats = load_or_build_incremental(b.path()).unwrap();
        // Reset → пересчитали с нуля, видим реальные tokens.
        assert_eq!(stats.tokens_in_total, 99);
        cleanup_cache(b.path());
    }

    #[test]
    fn nonexistent_transcript_returns_none() {
        let path = std::env::temp_dir().join("__cchud-does-not-exist.jsonl");
        let _ = fs::remove_file(&path);
        assert!(load_or_build_incremental(&path).is_none());
    }
}
