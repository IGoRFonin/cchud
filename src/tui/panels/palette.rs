//! Widget palette (top-right) — 60 widgets + filter + categories.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{App, EditField, Pane};
use crate::tui::widget_meta::{ALL_KINDS, WidgetCategory, WidgetMeta};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Palette;
    let in_filter_edit = matches!(app.editing_field, Some(EditField::PaletteFilter));

    let filter_str = if in_filter_edit {
        format!("/ {}_", app.palette_filter)
    } else if app.palette_filter.is_empty() {
        "press / to filter".into()
    } else {
        format!("/ {}", app.palette_filter)
    };

    let filtered: Vec<&WidgetMeta> = filter_meta(&app.palette_filter);

    let mut lines = Vec::with_capacity(filtered.len() + 6);
    lines.push(Line::from(Span::styled(
        filter_str,
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));

    let mut current_cat: Option<WidgetCategory> = None;
    for (i, m) in filtered.iter().enumerate() {
        if Some(m.category) != current_cat {
            lines.push(Line::from(Span::styled(
                format!("── {} ──", m.category.label()),
                Style::default().fg(Color::DarkGray),
            )));
            current_cat = Some(m.category);
        }
        let mark = if i == app.palette_cursor && focused {
            "▶ "
        } else {
            "  "
        };
        let style = if i == app.palette_cursor && focused {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::raw(mark),
            Span::styled(m.display, style),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Widgets ({}/60)", filtered.len()))
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn filter_meta(filter: &str) -> Vec<&'static WidgetMeta> {
    let needle = filter.trim().to_lowercase();
    if needle.is_empty() {
        return ALL_KINDS.iter().collect();
    }
    ALL_KINDS
        .iter()
        .filter(|m| {
            m.display.to_lowercase().contains(&needle)
                || m.kebab_type.contains(&needle)
                || m.category.label().to_lowercase().contains(&needle)
        })
        .collect()
}
