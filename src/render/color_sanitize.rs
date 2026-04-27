//! Color downgrade ladder — Phase 4 Task 4.
//!
//! Single-step adapt: `Rgb → Ansi256` only when truecolor unavailable.
//! `Ansi256` always passes through. `None` level always returns `None`.
//!
//! `rgb_to_ansi256` follows the standard 6×6×6 cube + 24-step grayscale
//! mapping (matches upstream ccstatusline behavior byte-for-byte on grays
//! and primaries).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::{Color, ColorLevel};

#[must_use]
pub const fn adapt_color(color: Color, level: ColorLevel) -> Option<Color> {
    match (color, level) {
        (_, ColorLevel::None) => None,
        (Color::Rgb(_, _, _), ColorLevel::TrueColor) | (Color::Ansi256(_), _) => Some(color),
        (Color::Rgb(r, g, b), ColorLevel::Ansi256) => Some(Color::Ansi256(rgb_to_ansi256(r, g, b))),
    }
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Grayscale check: r==g==b → 24-step gray ramp (codes 232..=255).
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return 232 + ((r as u16 - 8) * 24 / 247) as u8;
    }
    // 6×6×6 color cube (codes 16..=231).
    16 + 36 * (r as u16 * 5 / 255) as u8
        + 6 * (g as u16 * 5 / 255) as u8
        + (b as u16 * 5 / 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_level_strips_color() {
        assert_eq!(adapt_color(Color::Rgb(255, 0, 0), ColorLevel::None), None);
        assert_eq!(adapt_color(Color::Ansi256(196), ColorLevel::None), None);
    }

    #[test]
    fn truecolor_passes_rgb() {
        assert_eq!(
            adapt_color(Color::Rgb(10, 20, 30), ColorLevel::TrueColor),
            Some(Color::Rgb(10, 20, 30))
        );
    }

    #[test]
    fn ansi256_passes_through_at_any_level() {
        assert_eq!(
            adapt_color(Color::Ansi256(42), ColorLevel::Ansi256),
            Some(Color::Ansi256(42))
        );
        assert_eq!(
            adapt_color(Color::Ansi256(42), ColorLevel::TrueColor),
            Some(Color::Ansi256(42))
        );
    }

    #[test]
    fn rgb_downgrades_to_ansi256() {
        // Pure red 255,0,0 → cube code 196.
        assert_eq!(
            adapt_color(Color::Rgb(255, 0, 0), ColorLevel::Ansi256),
            Some(Color::Ansi256(196))
        );
    }

    #[test]
    fn rgb_to_ansi256_grayscale_endpoints() {
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16); // black
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231); // white
        assert_eq!(rgb_to_ansi256(128, 128, 128), 243); // 232 + (120u16 * 24 / 247) = 243
    }

    #[test]
    fn rgb_to_ansi256_primaries() {
        assert_eq!(rgb_to_ansi256(255, 0, 0), 196);
        assert_eq!(rgb_to_ansi256(0, 255, 0), 46);
        assert_eq!(rgb_to_ansi256(0, 0, 255), 21);
    }
}
