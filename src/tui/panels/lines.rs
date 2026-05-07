//! Lines panel (top-left) — список линий + текущие виджеты.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::{App, Pane};
use crate::tui::ui::panel_block;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Lines;
    let block = panel_block("Lines", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Закреплённая подсказка сверху + прокручиваемое тело.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    let hint_area = chunks[0];
    let body_area = chunks[1];

    let hint = Span::styled(
        "↑↓ widgets · ←→ lines · Enter replace · a add · l line · d del · r raw · Alt+↑↓ reorder",
        Style::default().fg(Color::DarkGray),
    );
    frame.render_widget(Paragraph::new(Line::from(hint)), hint_area);

    let mut rows: Vec<Line<'_>> = Vec::with_capacity(app.editable.lines.len() * 4);
    let mut cursor_row: u16 = 0;
    for (li, line) in app.editable.lines.iter().enumerate() {
        if li == app.selected_line && app.selected_widget.is_none() {
            cursor_row = u16::try_from(rows.len()).unwrap_or(u16::MAX);
        }
        let line_mark = if li == app.selected_line {
            "▶ "
        } else {
            "  "
        };
        rows.push(Line::from(vec![
            Span::raw(line_mark),
            Span::styled(
                format!("Line {} ({} widgets)", li + 1, line.widgets.len()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        for (wi, item) in line.widgets.iter().enumerate() {
            if li == app.selected_line && app.selected_widget == Some(wi) {
                cursor_row = u16::try_from(rows.len()).unwrap_or(u16::MAX);
            }
            let widget_mark = if li == app.selected_line && app.selected_widget == Some(wi) {
                "    ▶ "
            } else {
                "      "
            };
            rows.push(Line::from(format!(
                "{widget_mark}{}",
                debug_kebab(&item.kind)
            )));
        }
    }

    let total = u16::try_from(rows.len()).unwrap_or(u16::MAX);
    let visible = body_area.height;
    let scroll = compute_scroll(cursor_row, total, visible);
    frame.render_widget(Paragraph::new(rows).scroll((scroll, 0)), body_area);
}

fn compute_scroll(cursor_row: u16, total: u16, visible: u16) -> u16 {
    if visible == 0 || total <= visible {
        return 0;
    }
    let max = total.saturating_sub(visible);
    if cursor_row >= visible {
        cursor_row
            .saturating_sub(visible.saturating_sub(1))
            .min(max)
    } else {
        0
    }
}

fn debug_kebab(cfg: &crate::types::config::WidgetConfig) -> String {
    use crate::tui::widget_meta::ALL_KINDS;
    use std::mem::discriminant;
    let d = discriminant(cfg);
    ALL_KINDS
        .iter()
        .find(|m| discriminant(&(m.factory)()) == d)
        .map_or("<unknown>".to_string(), |m| m.display.to_string())
}
