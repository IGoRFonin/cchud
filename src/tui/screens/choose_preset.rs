//! Choose Preset screen — preset list (left 40%) + live preview (right 60%).
#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::App;
use crate::tui::panels::preview;
use crate::tui::presets::{Preset, PresetSource};
use crate::tui::ui::panel_block;

/// Развёрнутый список с заголовками групп. `is_header == true` —
/// курсор пропускает.
struct ListItem {
    label: String,
    is_header: bool,
    preset_index: Option<usize>,
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_list(frame, cols[0], app);
    render_preview(frame, cols[1], app);
}

fn render_list(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let block = panel_block("Presets", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items = build_list(&app.presets);
    let active = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let header = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal = Style::default();

    let lines: Vec<Line<'_>> = items
        .iter()
        .map(|item| {
            if item.is_header {
                Line::from(Span::styled(item.label.clone(), header))
            } else {
                let is_selected = item.preset_index == Some(app.preset_cursor);
                let prefix = if is_selected { "  ▶ " } else { "    " };
                let style = if is_selected { active } else { normal };
                Line::from(Span::styled(format!("{prefix}{}", item.label), style))
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let preview_settings = app.presets.get(app.preset_cursor).map_or_else(
        || app.editable.clone(),
        |p| {
            let mut s = app.editable.clone();
            s.lines.clone_from(&p.data.lines);
            s.theme.clone_from(&p.data.theme);
            s
        },
    );
    preview::render_with_settings(frame, area, &preview_settings, &app.sample_payload, false);
}

fn build_list(presets: &[Preset]) -> Vec<ListItem> {
    let mut out: Vec<ListItem> = Vec::new();
    out.push(ListItem {
        label: "Built-in".to_string(),
        is_header: true,
        preset_index: None,
    });
    for (i, p) in presets.iter().enumerate() {
        if matches!(p.source, PresetSource::Builtin) {
            out.push(ListItem {
                label: p.name.clone(),
                is_header: false,
                preset_index: Some(i),
            });
        }
    }
    let user_present = presets
        .iter()
        .any(|p| matches!(p.source, PresetSource::UserSaved(_)));
    if user_present {
        out.push(ListItem {
            label: "User-saved".to_string(),
            is_header: true,
            preset_index: None,
        });
        for (i, p) in presets.iter().enumerate() {
            if matches!(p.source, PresetSource::UserSaved(_)) {
                out.push(ListItem {
                    label: p.name.clone(),
                    is_header: false,
                    preset_index: Some(i),
                });
            }
        }
    }
    out
}
