//! Cross-widget helper modules used across multiple widget files.
//!
//! Наполняются по мере роста потребностей кластеров:
//! - [`model_context_size`] — Task 4 (`ContextPercentageUsable` lookup)
//! - [`duration`] — Task 5 (`SessionClock` formatter)
//! - [`ascii_bar`] — Task 4 (`ContextBar` formatter)

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod ansi;
pub mod ascii_bar;
pub mod duration;
pub mod format_duration_long;   // Phase 7
pub mod format_memory;          // Phase 7
pub mod format_tokens;
pub mod model_context_size;
pub mod now;
