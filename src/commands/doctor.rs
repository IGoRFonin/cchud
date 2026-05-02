//! `cchud doctor` — environment health check (Phase 9 Task 5).
//!
//! 9 проверок: version, binary path, platform target, color level,
//! hyperlinks, cache dir, claude settings, cchud config, gh CLI.
//!
//! Exit codes:
//!   0 — все checks PASS or SKIP
//!   1 — хоть одна FAIL
//!   2 — нет FAIL, но есть WARN
//!
//! Реализация — Task 5.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::process::ExitCode;

#[must_use]
#[allow(dead_code)] // wired in T6
pub fn run(_args: &[String]) -> ExitCode {
    eprintln!("cchud doctor: not yet implemented (Phase 9 Task 5)");
    ExitCode::from(2)
}
