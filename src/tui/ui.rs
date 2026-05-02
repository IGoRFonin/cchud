//! Top-level draw orchestration — Phase 8 Task 10.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::tui::app::{App, MessageKind, Mode};
use crate::tui::overlays;
use crate::tui::panels;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let main = outer[0];
    let status = outer[1];

    // 4-panel grid.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    panels::lines::render(frame, top[0], app);
    panels::palette::render(frame, top[1], app);
    panels::settings::render(frame, bottom[0], app);
    panels::preview::render(frame, bottom[1], app);

    // Status bar.
    render_status_bar(frame, status, app);

    // Overlays поверх всего.
    match app.mode {
        Mode::Edit => {}
        Mode::ThemesOverlay => overlays::themes::render(frame, area, app),
        Mode::HelpOverlay => overlays::help::render(frame, area),
        Mode::ConfirmQuit => overlays::modal::render_confirm_quit(frame, area),
    }
}

fn render_status_bar(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let dirty_glyph = if app.dirty() { "● " } else { "  " };
    let mode = format!("{dirty_glyph}{:?}  focus={:?}", app.mode, app.focus);
    let msg = match &app.status_message {
        Some((m, MessageKind::Info)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Green))
        }
        Some((m, MessageKind::Warn)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Yellow))
        }
        Some((m, MessageKind::Error)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Red))
        }
        None => Line::from(format!("{mode}  ·  ? for help")).style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ),
    };
    frame.render_widget(Paragraph::new(msg), area);
}
