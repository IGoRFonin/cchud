//! Help overlay (?).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};

use crate::tui::overlays::modal::centered;
use crate::tui::ui::overlay_block;

const KEYBINDINGS: &[(&str, &str)] = &[
    ("Tab / Shift+Tab", "cycle focus across panels"),
    ("Lines: ↑ ↓", "walk widgets across lines"),
    ("Lines: ← →", "switch between lines"),
    ("Lines: Enter", "replace selected widget (open palette)"),
    ("Lines: a", "add widget to current line (open palette)"),
    ("Lines: l", "add new line"),
    ("Lines: d / Delete", "delete widget · or empty line"),
    ("Lines: r", "toggle raw value (drop label/icon prefix)"),
    ("Lines: Alt+↑ / ↓", "reorder widget within line"),
    ("Palette: type", "filter by name / category"),
    ("Palette: ↑ ↓", "navigate (auto-scroll)"),
    ("Palette: Enter", "apply (add or replace) and close"),
    ("Palette: Esc", "close palette without changes"),
    ("Settings: ↑ ↓", "select field (FG/BG/Bold/params)"),
    ("Settings: Enter", "open color picker · edit text/number"),
    ("Settings: Space", "toggle tri-state Bold"),
    ("Settings: Esc", "back to Lines · cancel edit"),
    ("Edit-mode: Enter", "apply"),
    ("Edit-mode: Esc", "cancel"),
    ("t", "toggle Themes overlay"),
    ("?", "this help (any key closes)"),
    ("Ctrl + S", "save and quit"),
    ("q / Ctrl + C", "quit (confirms if unsaved)"),
];

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered(60, 80, area);
    frame.render_widget(Clear, popup);

    let mut lines: Vec<Line<'_>> = Vec::with_capacity(KEYBINDINGS.len() + 3);
    lines.push(Line::from(Span::styled(
        "cchud configure  ·  keybindings",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
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

    let block = overlay_block("Help (? to close)");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
