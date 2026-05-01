//! Help overlay (?) — Phase 8 Task 10.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::overlays::modal::centered;

const KEYBINDINGS: &[(&str, &str)] = &[
    ("Tab / Shift+Tab",  "cycle focus across 4 panels"),
    ("↑ ↓ ← →",          "navigate within active panel"),
    ("Enter",            "add widget (palette) · commit edit (settings)"),
    ("Space",            "toggle tri-state bool (settings)"),
    ("a",                "add line (Lines pane)"),
    ("d / Delete",       "delete selected widget · or empty line"),
    ("Alt + ↑ / ↓",      "reorder widget within line"),
    ("/",                "filter palette by name / category"),
    ("t",                "toggle Themes overlay"),
    ("?",                "this help (any key closes)"),
    ("Ctrl + S",         "save and quit"),
    ("q / Ctrl + C",     "quit (confirms if unsaved)"),
];

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered(60, 80, area);
    frame.render_widget(Clear, popup);

    let mut lines: Vec<Line<'_>> = Vec::with_capacity(KEYBINDINGS.len() + 3);
    lines.push(Line::from(Span::styled(
        "cchud configure  ·  keybindings",
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
    )));
    lines.push(Line::from(""));
    for (k, desc) in KEYBINDINGS {
        lines.push(Line::from(format!("  {k:<18} {desc}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "(any key closes)",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default().borders(Borders::ALL).title("Help (? to close)");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
