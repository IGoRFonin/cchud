//! Render layer — Phase 4.
//!
//! Pipeline: Widget produces text → wrapped into `Segment { text, style, hyperlink }`
//! → `Renderer::render(segments)` joins them. `Plain` and `Powerline` are
//! both held in a single enum (no boxed dispatch).
//!
//! Submodules wired incrementally per plan tasks (3 → 11).

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Ansi256(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorLevel {
    #[default]
    None,
    /// 256 colors. Also the liberal fallback for old 16-color terminals.
    Ansi256,
    /// 24-bit truecolor.
    TrueColor,
}

pub mod color_sanitize;

impl ColorLevel {
    /// Auto-detect via `supports-color` on stdout. Returns `None` on non-TTY.
    #[must_use]
    pub fn detect() -> Self {
        match supports_color::on(supports_color::Stream::Stdout) {
            None => Self::None,
            Some(s) if s.has_16m => Self::TrueColor,
            Some(_) => Self::Ansi256, // covers Has16 and Has256
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_level_default_is_none() {
        assert_eq!(ColorLevel::default(), ColorLevel::None);
    }

    #[test]
    fn color_eq_rgb_values() {
        assert_eq!(Color::Rgb(1, 2, 3), Color::Rgb(1, 2, 3));
        assert_ne!(Color::Rgb(1, 2, 3), Color::Rgb(3, 2, 1));
    }

    #[test]
    fn color_distinguishes_rgb_and_ansi() {
        assert_ne!(Color::Rgb(255, 0, 0), Color::Ansi256(196));
    }
}
