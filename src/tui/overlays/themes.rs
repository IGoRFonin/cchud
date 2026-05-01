//! Themes overlay (t) — 5 builtins + 9 globals.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::app::App;
use crate::tui::overlays::modal::centered;

pub static BUILTIN_THEMES: &[&str] = &["default", "dracula", "solarized-dark", "nord", "gruvbox-dark"];

pub static GLOBAL_FIELDS: &[&str] = &[
    "global_bold",
    "inherit_separator_colors",
    "override_background_color",
    "override_foreground_color",
    "minimalist_mode",
    "flex_mode",
    "compact_threshold",
    "auto_align",
    "continue_theme_across_lines",
];

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let popup = centered(80, 80, area);
    frame.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(popup);

    let theme_label = app.editable.theme.theme_name.as_deref().unwrap_or("default");
    let mut left_lines: Vec<Line<'_>> = vec![Line::from("Builtin themes (t to close)")];
    for (i, name) in BUILTIN_THEMES.iter().enumerate() {
        let mark = if *name == theme_label { "▶ " } else { "  " };
        let highlight = if i == app.theme_field_cursor.min(BUILTIN_THEMES.len() - 1) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        left_lines.push(Line::from(format!("{mark}{name}")).style(highlight));
    }
    let left_block = Block::default().borders(Borders::ALL).title("Themes");
    frame.render_widget(Paragraph::new(left_lines).block(left_block), chunks[0]);

    let mut right_lines: Vec<Line<'_>> = vec![Line::from("Global theme settings")];
    let theme = &app.editable.theme;
    let values: [(&str, String); 9] = [
        ("global_bold",                bool_glyph(theme.global_bold)),
        ("inherit_separator_colors",   bool_glyph(theme.inherit_separator_colors)),
        ("override_background_color",  theme.override_background_color.clone().unwrap_or_else(|| "(none)".into())),
        ("override_foreground_color",  theme.override_foreground_color.clone().unwrap_or_else(|| "(none)".into())),
        ("minimalist_mode",            bool_glyph(theme.minimalist_mode)),
        ("flex_mode",                  format!("{:?}", theme.flex_mode)),
        ("compact_threshold",          theme.compact_threshold.to_string()),
        ("auto_align",                 bool_glyph(theme.auto_align)),
        ("continue_theme_across_lines",bool_glyph(theme.continue_theme_across_lines)),
    ];
    for (i, (k, v)) in values.iter().enumerate() {
        let mark = if i + BUILTIN_THEMES.len() == app.theme_field_cursor { "▶ " } else { "  " };
        right_lines.push(Line::from(format!("{mark}{k:<28}: {v}")));
    }
    let right_block = Block::default().borders(Borders::ALL).title("Globals");
    frame.render_widget(Paragraph::new(right_lines).block(right_block), chunks[1]);
}

fn bool_glyph(b: bool) -> String {
    (if b { "✓" } else { "✗" }).to_string()
}
