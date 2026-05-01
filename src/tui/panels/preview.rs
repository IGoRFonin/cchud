//! Live preview (bottom-right) — рендерит editable Settings на sample payload.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::render::{RenderState, Renderer, Segment};
use crate::tui::app::{App, Pane};
use crate::tui::style_map;
use crate::widgets::{RenderContext, build_widgets};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Preview;
    let ctx = RenderContext::new(&app.sample_payload, &app.editable);
    let renderer = Renderer::for_preview(&app.editable);
    let lines = build_widgets(&app.editable);
    let mut state = RenderState::default();

    let mut tui_lines: Vec<Line<'_>> = Vec::with_capacity(lines.len().max(1));

    if lines.is_empty() {
        tui_lines.push(Line::from("(empty — add a widget from the palette)"));
    } else {
        for line_widgets in lines.iter() {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });
    frame.render_widget(Paragraph::new(tui_lines).block(block), area);
}
