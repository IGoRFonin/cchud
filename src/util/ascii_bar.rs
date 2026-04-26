//! ASCII progress bar formatter.
//!
//! Bar shape: `[████░░░░░░]` — filled блоки `█` (U+2588) + empty `░`
//! (U+2591) внутри `[]`. Width = total filled+empty inside brackets.
//! `pct` clamped to [0.0, 100.0]; `width == 0` → "[]".

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn render(pct: f64, width: u32) -> String {
    if width == 0 {
        return "[]".to_string();
    }
    let clamped = pct.clamp(0.0, 100.0);
    let filled_f = (clamped / 100.0) * f64::from(width);
    // Round-half-to-even, но `round()` достаточно для UI.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let filled = filled_f.round() as u32;
    let filled = filled.min(width);
    let empty = width - filled;
    let mut s = String::with_capacity(width as usize * 3 + 2);
    s.push('[');
    for _ in 0..filled {
        s.push('█');
    }
    for _ in 0..empty {
        s.push('░');
    }
    s.push(']');
    s
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn width_zero_renders_empty_brackets() {
        assert_eq!(render(50.0, 0), "[]");
    }

    #[test]
    fn width_one_zero_pct() {
        assert_eq!(render(0.0, 1), "[░]");
    }

    #[test]
    fn width_one_full_pct() {
        assert_eq!(render(100.0, 1), "[█]");
    }

    #[test]
    fn width_ten_zero_pct() {
        assert_eq!(render(0.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn width_ten_fifty_pct() {
        assert_eq!(render(50.0, 10), "[█████░░░░░]");
    }

    #[test]
    fn width_ten_full_pct() {
        assert_eq!(render(100.0, 10), "[██████████]");
    }

    #[test]
    fn pct_above_100_clamps_to_full() {
        assert_eq!(render(150.0, 10), "[██████████]");
    }

    #[test]
    fn pct_below_0_clamps_to_empty() {
        assert_eq!(render(-25.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn rounds_to_nearest() {
        // 27% of 10 = 2.7 → round to 3 filled
        assert_eq!(render(27.0, 10), "[███░░░░░░░]");
    }
}
