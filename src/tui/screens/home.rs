//! Home screen — 4-item menu + live preview справа.
#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::App;
use crate::tui::panels::preview;
use crate::tui::ui::panel_block;

pub const ITEMS: &[&str] = &[
    "📝 Edit Lines",
    "🎨 Choose preset",
    "📦 Install to Claude Code",
    "🚪 Exit",
];

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let menu_height = u16::try_from(ITEMS.len()).unwrap_or(4) + 2;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(menu_height),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    render_menu(frame, rows[0], app);
    preview::render_with_settings(frame, rows[1], &app.editable, &app.sample_payload, false);
}

fn render_menu(frame: &mut Frame<'_>, area: Rect, app: &App) {
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
