//! Widget settings panel (bottom-left) — color/background/bold + custom params.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{App, Pane};
use crate::tui::widgets_ui::{color_picker, list_editor, number_input, tri_bool};
use crate::types::config::WidgetConfig;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Settings;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Widget Settings")
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(item) = app.current_widget() else {
        let hint = Paragraph::new("(no widget selected — add one from the palette →)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner);
        return;
    };

    let constraints = vec![
        Constraint::Length(3), // color
        Constraint::Length(3), // bg color
        Constraint::Length(1), // bold
        Constraint::Min(0),    // custom params
    ];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let cursor = app.settings_field_cursor;
    color_picker::render(frame, chunks[0], "FG", item.style.color.as_deref(), cursor, focused && cursor == 0);
    color_picker::render(frame, chunks[1], "BG", item.style.background_color.as_deref(), cursor, focused && cursor == 1);
    tri_bool::render(frame, chunks[2], "Bold", item.style.bold, focused && cursor == 2);

    match &item.kind {
        WidgetConfig::CustomText { params } => {
            frame.render_widget(Paragraph::new(format!("Text:    {}", params.text)), chunks[3]);
        }
        WidgetConfig::CustomSymbol { params } => {
            frame.render_widget(Paragraph::new(format!("Symbol:  {}", params.symbol)), chunks[3]);
        }
        WidgetConfig::Link { params } => {
            let mut lines = vec![Line::from(format!("URL:    {}", params.url))];
            if let Some(l) = &params.label {
                lines.push(Line::from(format!("Label:  {l}")));
            }
            frame.render_widget(Paragraph::new(lines), chunks[3]);
        }
        WidgetConfig::CustomCommand { params } => {
            let custom_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(chunks[3]);
            frame.render_widget(Paragraph::new(format!("Command: {}", params.command)), custom_chunks[0]);
            number_input::render(
                frame,
                custom_chunks[1],
                "Timeout ms:",
                &params.timeout_ms.to_string(),
                50,
                5000,
                focused && cursor == 3,
            );
            list_editor::render(frame, custom_chunks[2], "Args:", &params.args, cursor.saturating_sub(4), focused);
        }
        WidgetConfig::ContextBar { params } => {
            number_input::render(
                frame,
                chunks[3],
                "Width:",
                &params.width.to_string(),
                1,
                80,
                focused && cursor == 3,
            );
        }
        _ => {
            frame.render_widget(
                Paragraph::new("(no custom params)").style(Style::default().fg(Color::DarkGray)),
                chunks[3],
            );
        }
    }
}
