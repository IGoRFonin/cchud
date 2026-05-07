//! Home screen — 4-item menu.
#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::App;
use crate::tui::ui::panel_block;

pub const ITEMS: &[&str] = &[
    "Edit Lines",
    "Choose preset",
    "Install to Claude Code",
    "Exit",
];

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let block = panel_block("cchud configurator", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let normal = Style::default();

    let lines: Vec<Line<'_>> = ITEMS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let prefix = if i == app.home_cursor { "▶ " } else { "  " };
            let style = if i == app.home_cursor { active } else { normal };
            Line::from(vec![Span::styled(format!("{prefix}{label}"), style)])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}
