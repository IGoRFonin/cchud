//! Unix-epoch ms helper.
//!
//! Используется `RenderContext::new` для дефолтного `now_ms`. Тесты
//! инжектируют свой `now_ms` через explicit field-init синтаксис, не вызывая
//! этот хелпер.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Текущее время в Unix-ms. На системах с broken clock возвращает 0.
#[must_use]
pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn unix_now_ms_returns_positive() {
        // Тест прогоняется в 2026 — таймстамп явно > 1 января 1970.
        assert!(unix_now_ms() > 1_700_000_000_000);
    }

    #[test]
    fn unix_now_ms_monotonic_within_call() {
        let a = unix_now_ms();
        let b = unix_now_ms();
        assert!(b >= a);
    }
}
