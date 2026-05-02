//! Tri-state bool — Phase 8 Task 8. None / Some(true) / Some(false).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

#[must_use]
pub const fn glyph(value: Option<bool>) -> &'static str {
    match value {
        None => "[ ]",
        Some(true) => "[✓]",
        Some(false) => "[✗]",
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, value: Option<bool>, focused: bool) {
    let style = if focused {
        Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        Style::default()
    };
    let text = format!("{label} {}", glyph(value));
    frame.render_widget(Paragraph::new(text).style(style), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_for_three_states() {
        assert_eq!(glyph(None), "[ ]");
        assert_eq!(glyph(Some(true)), "[✓]");
        assert_eq!(glyph(Some(false)), "[✗]");
    }
}
