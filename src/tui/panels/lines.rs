//! Lines panel (top-left) — список линий + текущие виджеты.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{App, Pane};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Lines;
    let mut text = Vec::with_capacity(app.editable.lines.len() * 4 + 1);
    for (li, line) in app.editable.lines.iter().enumerate() {
        let line_mark = if li == app.selected_line { "▶ " } else { "  " };
        text.push(Line::from(vec![
            Span::raw(line_mark),
            Span::styled(format!("Line {} ({} widgets)", li + 1, line.widgets.len()),
                Style::default().add_modifier(Modifier::BOLD)),
        ]));
        for (wi, item) in line.widgets.iter().enumerate() {
            let widget_mark = if li == app.selected_line && app.selected_widget == Some(wi) {
                "    ▶ "
            } else {
                "      "
            };
            text.push(Line::from(format!("{widget_mark}{}", debug_kebab(&item.kind))));
        }
    }
    text.push(Line::from(""));
    text.push(Line::from(Span::styled("  + a: add line · d: delete · Alt+↑↓: reorder",
        Style::default().fg(Color::DarkGray))));

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Lines")
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn debug_kebab(cfg: &crate::types::config::WidgetConfig) -> String {
    use crate::tui::widget_meta::ALL_KINDS;
    use crate::types::config::{WidgetItem, WidgetStyleOverride};
    let item = WidgetItem { kind: cfg.clone(), style: WidgetStyleOverride::default() };
    if let Ok(v) = serde_json::to_value(&item) {
        if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
            return ALL_KINDS.iter().find(|m| m.kebab_type == t).map_or(t.to_string(), |m| m.display.to_string());
        }
    }
    "<unknown>".into()
}
