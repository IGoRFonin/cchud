//! Single-line text input — Phase 8 Task 8.
//!
//! Render-helper: рисует buffer + cursor caret. Event handling — в reducer (T7).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, buffer: &str, cursor: usize, focused: bool) {
    let (left, mid, right) = split_at_cursor(buffer, cursor);
    let line = Line::from(vec![
        Span::raw(format!("{label} ")),
        Span::raw(left.to_string()),
        Span::styled(
            mid.to_string(),
            Style::default().add_modifier(Modifier::REVERSED),
        ),
        Span::raw(right.to_string()),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            Style::default()
        });
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn split_at_cursor(s: &str, cursor: usize) -> (&str, &str, &str) {
    let (left, rest) = s.split_at(cursor.min(s.len()));
    if rest.is_empty() {
        (left, " ", "")
    } else {
        // Take 1 char width as caret highlight.
        let head_len = rest
            .chars()
            .next()
            .map_or(0, char::len_utf8);
        let (mid, tail) = rest.split_at(head_len);
        (left, mid, tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_at_cursor_handles_end_of_string() {
        let (l, m, r) = split_at_cursor("abc", 3);
        assert_eq!(l, "abc");
        assert_eq!(m, " ");
        assert_eq!(r, "");
    }

    #[test]
    fn split_at_cursor_highlights_one_char() {
        let (l, m, r) = split_at_cursor("hello", 2);
        assert_eq!(l, "he");
        assert_eq!(m, "l");
        assert_eq!(r, "lo");
    }
}
