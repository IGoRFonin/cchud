//! Transcript JSONL cache.
//!
//! Контракт:
//! - `load_or_build_incremental(path)` — single entry point. Возвращает
//!   `Option<TranscriptStats>`; None если транскрипт недоступен / битый /
//!   IO error. Никогда не панкует.
//! - Внутренняя структура (`parser`, `store`) — pub(crate); внешние
//!   потребители (transcript-виджеты) обращаются только через `RenderContext::transcript()`.
//! - `cache/fixture.rs` — `#[cfg(test)]` helper `TranscriptBuilder` для
//!   интеграционных тестов парсера и store.
//!
//! Production-код в этом модуле должен соблюдать lint
//! `unwrap_used = "deny"` / `expect_used = "deny"` — error-path → `Option::None`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod jsonl_types;
mod parser;
pub mod rate_limits;
mod store;

#[cfg(test)]
pub mod fixture;

use std::path::PathBuf;

#[cfg(test)]
pub use jsonl_types::BillingBlock;
pub use jsonl_types::{MessageStats, TranscriptStats};
pub use store::load_or_build_incremental;

/// Корень cchud-кэша. Тесты могут переопределить через `CCHUD_CACHE_DIR`.
///
/// Production: `<dirs::cache_dir()>/cchud` (Linux: `~/.cache/cchud`,
/// macOS: `~/Library/Caches/cchud`). Fallback `/tmp/cchud` если `cache_dir`
/// недоступен.
#[must_use]
pub fn cache_root() -> PathBuf {
    if let Ok(p) = std::env::var("CCHUD_CACHE_DIR") {
        return PathBuf::from(p);
    }
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("cchud")
}
