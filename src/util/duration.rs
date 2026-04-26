//! Duration formatter — `HH:MM:SS` (≥ 1h) или `MM:SS` (< 1h).
//!
//! Используется `widgets::session::SessionClock`. Phase 4/8 могут
//! получить colored variant; для Phase 3 — plain string.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn format_duration(total_ms: u64) -> String {
    let total_secs = total_ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs / 60) % 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn zero_duration() {
        assert_eq!(format_duration(0), "00:00");
    }

    #[test]
    fn under_minute() {
        assert_eq!(format_duration(5_000), "00:05");
        assert_eq!(format_duration(59_999), "00:59");
    }

    #[test]
    fn under_hour() {
        // 90 sec = 1m30s
        assert_eq!(format_duration(90_000), "01:30");
        // 59m59s
        assert_eq!(format_duration(3_599_000), "59:59");
    }

    #[test]
    fn one_hour_threshold() {
        // 3600 sec = 1h00m00s
        assert_eq!(format_duration(3_600_000), "01:00:00");
    }

    #[test]
    fn multiple_hours() {
        // 2h05m07s = 7507 sec = 7_507_000 ms
        assert_eq!(format_duration(7_507_000), "02:05:07");
    }

    #[test]
    fn drops_sub_second_remainder() {
        // 1499 ms < 1.5 sec → 0 sec floor
        assert_eq!(format_duration(1_499), "00:01");
        // Нет, 1499 / 1000 = 1, потому "00:01". Корректно.
    }

    #[test]
    fn double_digit_hours() {
        // 100h00m00s = 360_000 sec = 360_000_000 ms
        assert_eq!(format_duration(360_000_000), "100:00:00");
    }
}
