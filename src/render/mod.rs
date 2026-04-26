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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underline: bool,
}

impl Style {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            dim: false,
            underline: false,
        }
    }

    #[must_use]
    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }

    #[must_use]
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }

    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    #[must_use]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Downgrade colors to the given level. Returns a new style.
    #[must_use]
    pub fn adapt(self, level: ColorLevel) -> Self {
        Self {
            fg: self.fg.and_then(|c| color_sanitize::adapt_color(c, level)),
            bg: self.bg.and_then(|c| color_sanitize::adapt_color(c, level)),
            ..self
        }
    }

    /// Render `text` with this style. If `level == None`, returns `text` as-is.
    #[must_use]
    pub fn render(&self, text: &str, level: ColorLevel) -> String {
        if level == ColorLevel::None {
            return text.to_string();
        }
        let adapted = self.adapt(level);
        let mut style = anstyle::Style::new();
        if let Some(c) = adapted.fg {
            style = style.fg_color(Some(to_anstyle_color(c)));
        }
        if let Some(c) = adapted.bg {
            style = style.bg_color(Some(to_anstyle_color(c)));
        }
        let mut effects = anstyle::Effects::new();
        if adapted.bold {
            effects = effects.insert(anstyle::Effects::BOLD);
        }
        if adapted.italic {
            effects = effects.insert(anstyle::Effects::ITALIC);
        }
        if adapted.dim {
            effects = effects.insert(anstyle::Effects::DIMMED);
        }
        if adapted.underline {
            effects = effects.insert(anstyle::Effects::UNDERLINE);
        }
        style = style.effects(effects);
        format!("{style}{text}{style:#}")
    }
}

fn to_anstyle_color(c: Color) -> anstyle::Color {
    match c {
        Color::Rgb(r, g, b) => anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)),
        Color::Ansi256(n) => anstyle::Color::Ansi256(anstyle::Ansi256Color(n)),
    }
}

pub mod hyperlink;

#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub style: Style,
    pub hyperlink: Option<String>,
}

impl Segment {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::none(),
            hyperlink: None,
        }
    }

    #[must_use]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            hyperlink: None,
        }
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;

    #[test]
    fn plain_constructs_neutral_segment() {
        let s = Segment::plain("hi");
        assert_eq!(s.text, "hi");
        assert_eq!(s.style, Style::none());
        assert!(s.hyperlink.is_none());
    }

    #[test]
    fn styled_carries_provided_style() {
        let s = Segment::styled("hi", Style::none().bold());
        assert!(s.style.bold);
    }
}

#[cfg(test)]
mod style_tests {
    use super::*;

    #[test]
    fn none_level_returns_text_unchanged() {
        let s = Style::none().fg(Color::Rgb(255, 0, 0)).bold();
        assert_eq!(s.render("hello", ColorLevel::None), "hello");
    }

    #[test]
    fn truecolor_emits_rgb_escape() {
        let s = Style::none().fg(Color::Rgb(255, 0, 0));
        let out = s.render("x", ColorLevel::TrueColor);
        assert!(out.contains("\x1b["), "expected ANSI escape");
        assert!(out.contains("38;2;255;0;0"), "expected truecolor fg, got: {out:?}");
        assert!(out.ends_with("\x1b[0m"), "expected reset");
    }

    #[test]
    fn ansi256_level_downgrades_rgb() {
        let s = Style::none().fg(Color::Rgb(255, 0, 0));
        let out = s.render("x", ColorLevel::Ansi256);
        assert!(out.contains("38;5;196"), "expected 256-color red, got: {out:?}");
    }

    #[test]
    fn bold_emits_effect() {
        let s = Style::none().bold();
        let out = s.render("x", ColorLevel::TrueColor);
        assert!(out.contains("\x1b[1m") || out.contains(";1m"), "got: {out:?}");
    }

    #[test]
    fn empty_style_does_not_emit_escape_at_truecolor() {
        let s = Style::none();
        // anstyle treats no-op style as identity → no leading escape.
        assert_eq!(s.render("x", ColorLevel::TrueColor), "x");
    }

    #[test]
    fn adapt_strips_colors_at_none_level() {
        let s = Style::none()
            .fg(Color::Rgb(1, 2, 3))
            .bg(Color::Rgb(4, 5, 6))
            .bold();
        let a = s.adapt(ColorLevel::None);
        assert_eq!(a.fg, None);
        assert_eq!(a.bg, None);
        assert!(a.bold, "effects survive None-level");
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
