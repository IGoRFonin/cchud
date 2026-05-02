//! cchud Style/Color → ratatui Style/Color/Span — Phase 8 Task 3.
//!
//! Hot path остаётся под `Style::render(&str, ColorLevel) -> String` (ANSI emit).
//! TUI preview мапит [`StyledSegment`] → [`ratatui::text::Span`] — без ANSI escape, без OSC 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::style::{Color as RColor, Modifier, Style as RStyle};
use ratatui::text::Span;

use crate::render::{Color, Style, StyledSegment};

/// Converts cchud `Color` → ratatui `Color`. RGB → `Rgb(r,g,b)`; Ansi256 → `Indexed(n)`.
#[must_use]
pub const fn to_ratatui_color(c: Color) -> RColor {
    match c {
        Color::Rgb(r, g, b) => RColor::Rgb(r, g, b),
        Color::Ansi256(n) => RColor::Indexed(n),
    }
}

/// Converts cchud `Style` → ratatui `Style`.
///
/// fg/bg + bold/italic/dim/underline. `Style::adapt(level)` НЕ вызывается — preview
/// forces `TrueColor` via `Renderer::for_preview` (T2). None fg/bg → None in ratatui.
#[must_use]
pub fn to_ratatui_style(s: Style) -> RStyle {
    let mut out = RStyle::default();
    if let Some(c) = s.fg {
        out = out.fg(to_ratatui_color(c));
    }
    if let Some(c) = s.bg {
        out = out.bg(to_ratatui_color(c));
    }
    let mut mods = Modifier::empty();
    if s.bold {
        mods.insert(Modifier::BOLD);
    }
    if s.italic {
        mods.insert(Modifier::ITALIC);
    }
    if s.dim {
        mods.insert(Modifier::DIM);
    }
    if s.underline {
        mods.insert(Modifier::UNDERLINED);
    }
    out.add_modifier(mods)
}

/// `StyledSegment` → ratatui `Span` (owned). Hyperlink игнорируется — TUI не
/// рендерит OSC 8.
#[must_use]
pub fn to_span(seg: &StyledSegment) -> Span<'static> {
    Span::styled(seg.text.clone(), to_ratatui_style(seg.style))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::render::{Color, Style, StyledSegment};

    #[test]
    fn to_ratatui_color_maps_rgb() {
        assert_eq!(to_ratatui_color(Color::Rgb(1, 2, 3)), RColor::Rgb(1, 2, 3));
    }

    #[test]
    fn to_ratatui_color_maps_ansi256() {
        assert_eq!(to_ratatui_color(Color::Ansi256(196)), RColor::Indexed(196));
    }

    #[test]
    fn to_ratatui_style_carries_fg_bg_and_modifiers() {
        let s = Style::none()
            .fg(Color::Rgb(10, 20, 30))
            .bg(Color::Rgb(40, 50, 60))
            .bold()
            .italic();
        let r = to_ratatui_style(s);
        assert_eq!(r.fg, Some(RColor::Rgb(10, 20, 30)));
        assert_eq!(r.bg, Some(RColor::Rgb(40, 50, 60)));
        assert!(r.add_modifier.contains(Modifier::BOLD));
        assert!(r.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn to_ratatui_style_none_yields_default() {
        let r = to_ratatui_style(Style::none());
        assert_eq!(r.fg, None);
        assert_eq!(r.bg, None);
        assert!(r.add_modifier.is_empty());
    }

    #[test]
    fn to_span_carries_text_and_style_drops_hyperlink() {
        let seg = StyledSegment {
            text: "hello".into(),
            style: Style::none().bold(),
            hyperlink: Some("https://x.com".into()),
        };
        let span = to_span(&seg);
        assert_eq!(span.content, "hello");
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn to_span_owned_is_static_lifetime() {
        // Compile-time check: Span<'static> can outlive the segment ref.
        let seg = StyledSegment::plain("x");
        let span: Span<'static> = to_span(&seg);
        drop(seg);
        assert_eq!(span.content, "x");
    }
}
