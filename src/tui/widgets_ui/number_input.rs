//! Numeric input + min/max validation — Phase 8 Task 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    buffer: &str,
    min: u64,
    max: u64,
    focused: bool,
) {
    let valid = buffer
        .parse::<u64>()
        .map_or(buffer.is_empty(), |n| (min..=max).contains(&n));
    let bg = if !valid {
        Style::default().bg(Color::Red)
    } else if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let caret = if focused { "_" } else { " " };
    let text = format!("{label} {buffer}{caret}  (range {min}–{max})");
    frame.render_widget(Paragraph::new(text).style(bg), area);
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_compiles() {
        // Render-helper смок-тест на полноценном `Frame` потребует ratatui::TestBackend
        // — покрытие в `tests/tui_snapshots.rs` (T14). Здесь — sanity-check на module link.
        let _ = super::render;
    }
}
