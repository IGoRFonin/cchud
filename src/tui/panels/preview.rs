//! Live preview (bottom-right) — рендерит editable Settings на sample payload.
//!
//! Mock Claude Code prompt-box сверху + rendered statusline снизу — чтобы превью
//! выглядел так же, как реальный CC (paritет с ccstatusline).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use crate::render::{RenderState, Renderer, Segment};
use crate::tui::app::{App, Pane};
use crate::tui::style_map;
use crate::tui::ui::panel_block;
use crate::widgets::{RenderContext, build_widgets};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Preview;
    let block = panel_block("Preview", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: mock CC input box (3 rows) + blank gap (1 row) + statusline (rest).
    // Если высоты не хватает — input всё равно занимает 3 строки, statusline схлопывается.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    render_mock_input(frame, chunks[0]);
    render_statusline(frame, chunks[2], app);
}

/// Mock Claude Code text-input prompt — rounded box + `>` prompt + dim hint.
fn render_mock_input(frame: &mut Frame<'_>, area: Rect) {
    let mock_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .border_style(Style::default().fg(Color::DarkGray));
    let dim = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);
    let prompt = Style::default().fg(Color::Gray);
    let line = Line::from(vec![
        Span::styled("> ", prompt),
        Span::styled("ctrl+s saves config", dim),
    ]);
    frame.render_widget(Paragraph::new(line).block(mock_block), area);
}

/// Render the actual configured statusline на sample payload.
fn render_statusline(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let ctx = RenderContext::new(&app.sample_payload, &app.editable);
    let renderer = Renderer::for_preview(&app.editable);
    let lines = build_widgets(&app.editable);
    let mut state = RenderState::default();

    let mut tui_lines: Vec<Line<'_>> = Vec::with_capacity(lines.len().max(1));

    if lines.is_empty() {
        tui_lines.push(Line::from("(empty — add a widget from the palette)"));
    } else {
        for line_widgets in &lines {
            let segments: Vec<Segment> = line_widgets
                .iter()
                .filter_map(|(w, ovr)| {
                    let text = w.render(&ctx)?;
                    let is_align = w.id() == "align-right";
                    let style = crate::render::apply_widget_style(
                        w.default_style(),
                        None,
                        ovr,
                        &app.editable.theme,
                    );
                    Some(Segment {
                        text,
                        style,
                        hyperlink: w.hyperlink(&ctx),
                        align_marker: is_align,
                    })
                })
                .collect();
            let composed = renderer.compose_line(&segments, &mut state, &app.editable.theme);
            let spans: Vec<_> = composed.iter().map(style_map::to_span).collect();
            tui_lines.push(Line::from(spans));
            if !app.editable.theme.continue_theme_across_lines {
                state.reset();
            }
        }
    }

    frame.render_widget(Paragraph::new(tui_lines), area);
}
