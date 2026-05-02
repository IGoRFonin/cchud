//! Widget settings panel (bottom-left) — color/background/bold + custom params.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{App, Pane};

const TIMEOUT_MIN_MS: u64 = 50;
const TIMEOUT_MAX_MS: u64 = 5000;
use crate::tui::widgets_ui::{color_picker, list_editor, number_input, tri_bool};
use crate::types::config::WidgetConfig;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Settings;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Widget Settings")
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
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
    color_picker::render(
        frame,
        chunks[0],
        "FG",
        item.style.color.as_deref(),
        app.color_fg_cursor,
        focused && cursor == 0,
    );
    color_picker::render(
        frame,
        chunks[1],
        "BG",
        item.style.background_color.as_deref(),
        app.color_bg_cursor,
        focused && cursor == 1,
    );
    tri_bool::render(
        frame,
        chunks[2],
        "Bold",
        item.style.bold,
        focused && cursor == 2,
    );

    render_kind_params(frame, chunks[3], &item.kind, cursor, focused);
}

fn render_kind_params(
    frame: &mut Frame<'_>,
    area: Rect,
    kind: &WidgetConfig,
    cursor: usize,
    focused: bool,
) {
    match kind {
        WidgetConfig::CustomText { params } => {
            frame.render_widget(Paragraph::new(format!("Text:    {}", params.text)), area);
        }
        WidgetConfig::CustomSymbol { params } => {
            frame.render_widget(Paragraph::new(format!("Symbol:  {}", params.symbol)), area);
        }
        WidgetConfig::Link { params } => {
            let mut lines = vec![Line::from(format!("URL:    {}", params.url))];
            if let Some(l) = &params.label {
                lines.push(Line::from(format!("Label:  {l}")));
            }
            frame.render_widget(Paragraph::new(lines), area);
        }
        WidgetConfig::CustomCommand { params } => {
            render_custom_command(frame, area, params, cursor, focused);
        }
        WidgetConfig::ContextBar { params } => {
            number_input::render(
                frame,
                area,
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
                area,
            );
        }
    }
}

fn render_custom_command(
    frame: &mut Frame<'_>,
    area: Rect,
    params: &crate::types::config::CustomCommandParams,
    cursor: usize,
    focused: bool,
) {
    let custom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(format!("Command: {}", params.command)),
        custom_chunks[0],
    );
    number_input::render(
        frame,
        custom_chunks[1],
        "Timeout ms:",
        &params.timeout_ms.to_string(),
        TIMEOUT_MIN_MS,
        TIMEOUT_MAX_MS,
        focused && cursor == 3,
    );
    list_editor::render(
        frame,
        custom_chunks[2],
        "Args:",
        &params.args,
        cursor.saturating_sub(4),
        focused,
    );
}
