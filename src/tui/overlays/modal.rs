//! Modal helpers.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};

use crate::tui::ui::overlay_block;

#[must_use]
pub fn centered(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Confirm-quit modal — `[s] Save · [d] Discard · [c] Cancel`.
pub fn render_confirm_quit(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered(50, 20, area);
    frame.render_widget(Clear, popup);
    let lines = vec![
        Line::from(Span::styled(
            "Unsaved changes",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )),
        Line::from("You have unsaved changes."),
        Line::from(""),
        Line::from("[s] Save and quit"),
        Line::from("[d] Discard and quit"),
        Line::from("[c] Cancel (Esc)"),
    ];
    let block = overlay_block("Unsaved changes");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
