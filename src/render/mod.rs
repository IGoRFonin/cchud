//! Render layer — Phase 4.
//!
//! Pipeline: Widget produces text → wrapped into `Segment { text, style, hyperlink }`
//! → `Renderer::render(segments)` joins them. `Plain` and `Powerline` are
//! both held in a single enum (no boxed dispatch).
//!
//! Submodules wired incrementally per plan tasks (3 → 11).

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Color {
    Rgb(u8, u8, u8),
    Ansi256(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorLevel {
    #[default]
    None,
    /// 256 colors. Also the liberal fallback for old 16-color terminals.
    Ansi256,
    /// 24-bit truecolor.
    TrueColor,
}

pub mod color_parse;
pub mod color_sanitize;

impl ColorLevel {
    /// Auto-detect via `supports-color` on stdout. Returns `None` on non-TTY.
    #[must_use]
    pub fn detect() -> Self {
        if let Ok(forced) = std::env::var("CCHUD_TEST_COLOR_LEVEL") {
            return match forced.as_str() {
                "ansi256" => Self::Ansi256,
                "true-color" | "truecolor" => Self::TrueColor,
                _ => Self::None,
            };
        }
        match supports_color::on(supports_color::Stream::Stdout) {
            None => Self::None,
            Some(s) if s.has_16m => Self::TrueColor,
            Some(_) => Self::Ansi256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[allow(clippy::struct_excessive_bools)]
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

const fn to_anstyle_color(c: Color) -> anstyle::Color {
    match c {
        Color::Rgb(r, g, b) => anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)),
        Color::Ansi256(n) => anstyle::Color::Ansi256(anstyle::Ansi256Color(n)),
    }
}

pub mod flex; // Phase 7
pub mod hyperlink;
pub mod plain;
pub mod powerline;
pub mod themes;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct RenderState {
    pub global_theme_index: usize,
    pub global_separator_index: usize,
}

impl RenderState {
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.global_theme_index = 0;
        self.global_separator_index = 0;
    }
}

/// Композирует финальный `Style` для одного виджета.
/// Порядок: widget default → `theme.widget_styles`[id] → per-widget override → theme globals.
#[allow(dead_code)]
#[must_use]
pub fn apply_widget_style(
    widget_default: Style,
    theme_widget_style: Option<Style>,
    per_widget_override: &crate::types::config::WidgetStyleOverride,
    theme_globals: &crate::types::config::ThemeConfig,
) -> Style {
    let mut style = theme_widget_style.unwrap_or(widget_default);

    if let Some(c) = color_parse::parse_color(per_widget_override.color.as_deref()) {
        style.fg = Some(c);
    }
    if let Some(c) = color_parse::parse_color(per_widget_override.background_color.as_deref()) {
        style.bg = Some(c);
    }
    if let Some(b) = per_widget_override.bold {
        style.bold = b;
    }

    if theme_globals.global_bold {
        style.bold = true;
    }
    if let Some(c) = color_parse::parse_color(theme_globals.override_foreground_color.as_deref()) {
        style.fg = Some(c);
    }
    if let Some(c) = color_parse::parse_color(theme_globals.override_background_color.as_deref()) {
        style.bg = Some(c);
    }

    style
}

use crate::types::config::Settings;

#[derive(Debug)]
pub enum Renderer {
    Plain(plain::Plain),
    Powerline(powerline::Powerline),
}

impl Renderer {
    #[must_use]
    pub fn from_settings(settings: &Settings) -> Self {
        use crate::types::config::ThemeKind;

        let level = settings
            .theme
            .color_level
            .unwrap_or_else(ColorLevel::detect);
        let hyperlinks = level != ColorLevel::None && hyperlink::supports_hyperlinks_detect();

        match settings.theme.kind {
            ThemeKind::Plain => Self::Plain(plain::Plain {
                separator: " | ".into(),
                level,
                hyperlinks,
            }),
            ThemeKind::Powerline => {
                let theme: themes::PowerlineTheme = settings
                    .theme
                    .custom
                    .clone()
                    .or_else(|| {
                        settings
                            .theme
                            .theme_name
                            .as_deref()
                            .and_then(themes::lookup)
                            .map(Into::into)
                    })
                    .unwrap_or_else(|| (&themes::DEFAULT).into());

                let mut p = powerline::Powerline::new(theme, level, hyperlinks);
                if let Some(sep) = settings
                    .theme
                    .separators
                    .first()
                    .and_then(|s| s.chars().next())
                {
                    p.separator_left = sep;
                }
                Self::Powerline(p)
            }
        }
    }

    #[must_use]
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &crate::types::config::ThemeConfig,
    ) -> String {
        match self {
            Self::Plain(p) => p.render_line(segments, state, theme),
            Self::Powerline(p) => p.render_line(segments, state, theme),
        }
    }
}

#[cfg(test)]
mod apply_style_tests {
    use super::*;
    use crate::types::config::{ThemeConfig, WidgetStyleOverride};

    fn theme() -> ThemeConfig {
        ThemeConfig::default()
    }

    #[test]
    fn returns_widget_default_when_no_overrides() {
        let dflt = Style::none().bold();
        let theme = theme();
        let ovr = WidgetStyleOverride::default();
        let s = apply_widget_style(dflt, None, &ovr, &theme);
        assert!(s.bold);
        assert!(s.fg.is_none());
    }

    #[test]
    fn theme_widget_style_overrides_default() {
        let dflt = Style::none().bold();
        let theme_style = Style::none().fg(Color::Rgb(255, 0, 0));
        let theme = theme();
        let ovr = WidgetStyleOverride::default();
        let s = apply_widget_style(dflt, Some(theme_style), &ovr, &theme);
        assert_eq!(s.fg, Some(Color::Rgb(255, 0, 0)));
        assert!(!s.bold, "theme style replaced widget default entirely");
    }

    #[test]
    fn per_widget_override_changes_color_and_bold() {
        let mut ovr = WidgetStyleOverride::default();
        ovr.color = Some("#fafafa".into());
        ovr.bold = Some(true);
        let s = apply_widget_style(Style::none(), None, &ovr, &theme());
        assert_eq!(s.fg, Some(Color::Rgb(0xfa, 0xfa, 0xfa)));
        assert!(s.bold);
    }

    #[test]
    fn global_bold_force_enables_after_override() {
        let mut theme = theme();
        theme.global_bold = true;
        let mut ovr = WidgetStyleOverride::default();
        ovr.bold = Some(false);
        let s = apply_widget_style(Style::none(), None, &ovr, &theme);
        assert!(
            s.bold,
            "global_bold force-enables after per-widget override"
        );
    }

    #[test]
    fn override_foreground_color_wins_over_per_widget() {
        let mut theme = theme();
        theme.override_foreground_color = Some("#aabbcc".into());
        let mut ovr = WidgetStyleOverride::default();
        ovr.color = Some("#000000".into());
        let s = apply_widget_style(Style::none(), None, &ovr, &theme);
        assert_eq!(s.fg, Some(Color::Rgb(0xaa, 0xbb, 0xcc)));
    }
}

#[cfg(test)]
mod renderer_tests {
    use super::*;
    use crate::types::config::Settings;

    #[test]
    fn from_settings_returns_plain_with_default_settings() {
        let s = Settings::default();
        let r = Renderer::from_settings(&s);
        assert!(matches!(r, Renderer::Plain(_)));
    }

    #[test]
    fn renderer_dispatches_render_to_plain() {
        use crate::types::config::ThemeConfig;
        let r = Renderer::Plain(plain::Plain {
            separator: ", ".into(),
            level: ColorLevel::None,
            hyperlinks: false,
        });
        let mut state = RenderState::default();
        let theme = ThemeConfig::default();
        assert_eq!(
            r.render_line(&[Segment::plain("a"), Segment::plain("b")], &mut state, &theme),
            "a, b"
        );
    }

    #[test]
    fn from_settings_returns_powerline_when_kind_is_powerline() {
        let json = r#"{"theme": {"kind": "powerline", "theme_name": "dracula"}}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let r = Renderer::from_settings(&s);
        assert!(matches!(r, Renderer::Powerline(_)));
    }

    #[test]
    fn powerline_falls_back_to_default_for_unknown_theme_name() {
        let json = r#"{"theme": {"kind": "powerline", "theme_name": "nope"}}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let r = Renderer::from_settings(&s);
        match r {
            Renderer::Powerline(p) => assert_eq!(p.theme.name, "default"),
            other => panic!("expected powerline, got {other:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub style: Style,
    pub hyperlink: Option<String>,
    /// Phase 7: marker для `auto_align` — сегмент действует как разделитель left/right.
    pub align_marker: bool,
}

impl Segment {
    #[must_use]
    #[allow(dead_code)]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::none(),
            hyperlink: None,
            align_marker: false,
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            hyperlink: None,
            align_marker: false,
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
        assert!(
            out.contains("38;2;255;0;0"),
            "expected truecolor fg, got: {out:?}"
        );
        assert!(out.ends_with("\x1b[0m"), "expected reset");
    }

    #[test]
    fn ansi256_level_downgrades_rgb() {
        let s = Style::none().fg(Color::Rgb(255, 0, 0));
        let out = s.render("x", ColorLevel::Ansi256);
        assert!(
            out.contains("38;5;196"),
            "expected 256-color red, got: {out:?}"
        );
    }

    #[test]
    fn bold_emits_effect() {
        let s = Style::none().bold();
        let out = s.render("x", ColorLevel::TrueColor);
        assert!(
            out.contains("\x1b[1m") || out.contains(";1m"),
            "got: {out:?}"
        );
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
    #[serial_test::serial]
    fn detect_honours_env_override() {
        // SAFETY: serial test guarded by name uniqueness; this var is only read here.
        unsafe { std::env::set_var("CCHUD_TEST_COLOR_LEVEL", "true-color") };
        assert_eq!(ColorLevel::detect(), ColorLevel::TrueColor);
        unsafe { std::env::set_var("CCHUD_TEST_COLOR_LEVEL", "none") };
        assert_eq!(ColorLevel::detect(), ColorLevel::None);
        unsafe { std::env::remove_var("CCHUD_TEST_COLOR_LEVEL") };
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
