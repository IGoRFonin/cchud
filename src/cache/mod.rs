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
mod store;

#[cfg(test)]
pub mod fixture;

#[cfg(test)]
pub use jsonl_types::BillingBlock;
pub use jsonl_types::{MessageStats, TranscriptStats};
pub use store::load_or_build_incremental;
