#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn format_tokens(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    #[allow(clippy::cast_precision_loss)]
    if n < 1_000_000 {
        return compact(n as f64 / 1_000.0, "k");
    }
    #[allow(clippy::cast_precision_loss)]
    compact(n as f64 / 1_000_000.0, "M")
}

fn compact(value: f64, suffix: &str) -> String {
    let s = format!("{value:.1}");
    let trimmed = s.strip_suffix(".0").unwrap_or(&s);
    format!("{trimmed}{suffix}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn under_thousand() {
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn round_thousand_no_trailing_zero() {
        assert_eq!(format_tokens(1_000), "1k");
    }

    #[test]
    fn fractional_thousand() {
        assert_eq!(format_tokens(1_500), "1.5k");
    }

    #[test]
    fn near_million() {
        assert_eq!(format_tokens(999_500), "999.5k");
    }

    #[test]
    fn round_million_no_trailing_zero() {
        assert_eq!(format_tokens(1_000_000), "1M");
    }

    #[test]
    fn fractional_million() {
        assert_eq!(format_tokens(1_234_567), "1.2M");
    }

    #[test]
    fn large_million() {
        assert_eq!(format_tokens(50_500_000), "50.5M");
    }
}
