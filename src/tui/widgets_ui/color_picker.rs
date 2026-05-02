//! Color picker — Phase 8 Task 8. 16 ANSI named + Default + Custom hex.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color as RColor, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub static NAMED_COLORS: &[(&str, &str)] = &[
    ("Default (none)", ""),
    ("Black", "#000000"),
    ("Red", "#cd0000"),
    ("Green", "#00cd00"),
    ("Yellow", "#cdcd00"),
    ("Blue", "#0000ee"),
    ("Magenta", "#cd00cd"),
    ("Cyan", "#00cdcd"),
    ("White", "#e5e5e5"),
    ("Bright Black", "#7f7f7f"),
    ("Bright Red", "#ff0000"),
    ("Bright Green", "#00ff00"),
    ("Bright Yellow", "#ffff00"),
    ("Bright Blue", "#5c5cff"),
    ("Bright Magenta", "#ff00ff"),
    ("Bright Cyan", "#00ffff"),
    ("Bright White", "#ffffff"),
    ("Custom hex…", ""),
];

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    current: Option<&str>,
    cursor: usize,
    focused: bool,
) {
    let mut lines = Vec::with_capacity(NAMED_COLORS.len() + 1);
    lines.push(Line::from(format!(
        "{label} = {}",
        current.unwrap_or("(none)")
    )));
    for (i, (name, hex)) in NAMED_COLORS.iter().enumerate() {
        let mark = if i == cursor && focused { "▶ " } else { "  " };
        let swatch = if hex.is_empty() {
            Span::raw("  ")
        } else if let Some(c) = parse_hex(hex) {
            Span::styled("██", Style::default().fg(c))
        } else {
            Span::raw("  ")
        };
        lines.push(Line::from(vec![
            Span::raw(mark),
            swatch,
            Span::raw(format!(" {name}")),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn parse_hex(s: &str) -> Option<RColor> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(RColor::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_colors_has_18_entries() {
        // 16 ANSI + Default + Custom hex
        assert_eq!(NAMED_COLORS.len(), 18);
    }

    #[test]
    fn parse_hex_valid_returns_rgb() {
        assert_eq!(parse_hex("#aabbcc"), Some(RColor::Rgb(0xaa, 0xbb, 0xcc)));
    }

    #[test]
    fn parse_hex_invalid_returns_none() {
        assert!(parse_hex("zzz").is_none());
        assert!(parse_hex("#xx").is_none());
    }
}
