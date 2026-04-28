//! Phase 6 Task 2 will populate this file with `TranscriptEntry`, `Usage`,
//! `TranscriptStats`, `MessageStats`, `BillingBlock`, `CacheMeta`, `CacheFile`.
//!
//! T1 stub: minimal placeholder so `cache/mod.rs` re-exports compile.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: u32 = 1;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct TranscriptStats {}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MessageStats;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BillingBlock;

#[derive(Serialize, Deserialize)]
pub struct CacheMeta {
    pub format_version: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CacheFile {
    pub meta: CacheMeta,
    pub stats: TranscriptStats,
}
