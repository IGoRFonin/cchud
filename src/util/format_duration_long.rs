//! Duration formatters — short ("4h32m") and long ("5d 14h") variants.

#![deny(clippy::unwrap_used, clippy::expect_used)]

const MIN: i64 = 60;
const HOUR: i64 = 60 * MIN;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;

/// Short form: `< 1m`, `Nm`, `NhMm`, `Nd Mh`. Negative seconds → `< 1m`.
#[must_use]
pub fn format_short(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < MIN {
        return "< 1m".to_string();
    }
    if s < HOUR {
        return format!("{}m", s / MIN);
    }
    if s < DAY {
        return format!("{}h{}m", s / HOUR, (s % HOUR) / MIN);
    }
    format!("{}d {}h", s / DAY, (s % DAY) / HOUR)
}

/// Long form: `Nm`, `NhMm`, `Nd Mh`, `Nw Md`.
#[must_use]
pub fn format_long(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < HOUR {
        return format!("{}m", (s / MIN).max(0));
    }
    if s < DAY {
        return format!("{}h{}m", s / HOUR, (s % HOUR) / MIN);
    }
    if s < WEEK {
        return format!("{}d {}h", s / DAY, (s % DAY) / HOUR);
    }
    format!("{}w {}d", s / WEEK, (s % WEEK) / DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    // format_short: < 1m / Nm / NhMm / Nd Mh
    #[test]
    fn short_under_1_minute() {
        assert_eq!(format_short(0), "< 1m");
        assert_eq!(format_short(45), "< 1m");
    }

    #[test]
    fn short_minutes() {
        assert_eq!(format_short(60), "1m");
        assert_eq!(format_short(42 * 60), "42m");
    }

    #[test]
    fn short_hours_and_minutes() {
        assert_eq!(format_short(4 * 3600 + 32 * 60), "4h32m");
        assert_eq!(format_short(23 * 3600 + 59 * 60), "23h59m");
    }

    #[test]
    fn short_days_and_hours() {
        assert_eq!(format_short(24 * 3600), "1d 0h");
        assert_eq!(format_short(24 * 3600 + 5 * 3600), "1d 5h");
    }

    #[test]
    fn short_negative_clamped_to_under_1m() {
        assert_eq!(format_short(-100), "< 1m");
    }

    // format_long: Nm / NhMm / Nd Mh / Nw Md
    #[test]
    fn long_minutes() {
        assert_eq!(format_long(42 * 60), "42m");
    }

    #[test]
    fn long_hours_and_minutes() {
        assert_eq!(format_long(4 * 3600 + 32 * 60), "4h32m");
    }

    #[test]
    fn long_days_and_hours() {
        assert_eq!(format_long(5 * 86400 + 14 * 3600), "5d 14h");
    }

    #[test]
    fn long_weeks_and_days() {
        assert_eq!(format_long(8 * 86400), "1w 1d");
        assert_eq!(format_long(14 * 86400 + 3 * 86400), "2w 3d");
    }
}
