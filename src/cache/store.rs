//! Phase 6 Task 4 will populate `load_or_build_incremental`, `read_cache`,
//! `write_cache_best_effort`, `cache_path_for`.
//!
//! T1 stub: minimal placeholder so cache/mod.rs re-exports compile.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use crate::cache::jsonl_types::TranscriptStats;

#[allow(dead_code)]
#[must_use]
pub const fn load_or_build_incremental(_path: &Path) -> Option<TranscriptStats> {
    // T4 will replace this with real impl.
    None
}
