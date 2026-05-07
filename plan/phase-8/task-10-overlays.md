# Task 10 — `tui::overlays` + `tui::ui::draw` (themes / help / modal + orchestration)

**Цель:** Три overlay-функции (themes / help / modal) + top-level `ui::draw(frame, &app)` который собирает 4 панели + status bar в layout. Overlays рисуются ПОВЕРХ панелей в зависимости от `app.mode`.

**Files:**
- Modify: `src/tui/overlays/themes.rs`
- Modify: `src/tui/overlays/help.rs`
- Modify: `src/tui/overlays/modal.rs`
- Modify: `src/tui/ui.rs`

---

- [ ] **Step 1: `overlays/help.rs`**

```rust
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
        lines.push(Line::from(format!("  {:<18} {}", k, desc)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "(any key closes)",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default().borders(Borders::ALL).title("Help (? to close)");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
```

`centered` импортируется из `tui::overlays::modal::centered` (определяется в Step 3).

- [ ] **Step 2: `overlays/themes.rs`**

```rust
//! Themes overlay (t) — 5 builtins + 9 globals.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::tui::app::App;

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
```

`centered` функция используется и в `help.rs` — выделить её в `tui::overlays::modal::centered` (см. Step 3) и импортировать в обоих файлах через `use crate::tui::overlays::modal::centered;`.

- [ ] **Step 3: `overlays/modal.rs`**

```rust
//! Modal helpers — Phase 8 Task 10.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

#[must_use]
pub fn centered(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Confirm-quit modal — `[s] Save · [d] Discard · [c] Cancel`.
pub fn render_confirm_quit(frame: &mut Frame<'_>, area: Rect) {
    let popup = centered(50, 20, area);
    frame.render_widget(Clear, popup);
    let lines = vec![
        Line::from(Span::styled(
            "Unsaved changes",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
        )),
        Line::from("You have unsaved changes."),
        Line::from(""),
        Line::from("[s] Save and quit"),
        Line::from("[d] Discard and quit"),
        Line::from("[c] Cancel (Esc)"),
    ];
    let block = Block::default().borders(Borders::ALL).title("Unsaved changes");
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}
```

Также добавить `use crate::tui::overlays::modal::centered;` в `themes.rs` (вверх файла, сразу после `use ratatui::widgets::...`) и удалить локальную копию `centered` из `themes.rs`.

- [ ] **Step 4: `tui::ui::draw` orchestration**

В `src/tui/ui.rs`:

```rust
//! Top-level draw orchestration — Phase 8 Task 10.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::tui::app::{App, Mode, MessageKind};
use crate::tui::overlays;
use crate::tui::panels;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let main = outer[0];
    let status = outer[1];

    // 4-panel grid.
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    panels::lines::render(frame, top[0], app);
    panels::palette::render(frame, top[1], app);
    panels::settings::render(frame, bottom[0], app);
    panels::preview::render(frame, bottom[1], app);

    // Status bar.
    render_status_bar(frame, status, app);

    // Overlays поверх всего.
    match app.mode {
        Mode::Edit => {}
        Mode::ThemesOverlay => overlays::themes::render(frame, area, app),
        Mode::HelpOverlay => overlays::help::render(frame, area),
        Mode::ConfirmQuit => overlays::modal::render_confirm_quit(frame, area),
    }
}

fn render_status_bar(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let dirty_glyph = if app.dirty() { "● " } else { "  " };
    let mode = format!("{dirty_glyph}{:?}  focus={:?}", app.mode, app.focus);
    let msg = match &app.status_message {
        Some((m, MessageKind::Info)) => Line::from(m.clone()).style(Style::default().fg(Color::Green)),
        Some((m, MessageKind::Warn)) => Line::from(m.clone()).style(Style::default().fg(Color::Yellow)),
        Some((m, MessageKind::Error)) => Line::from(m.clone()).style(Style::default().fg(Color::Red)),
        None => Line::from(format!("{mode}  ·  ? for help")).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
    };
    frame.render_widget(Paragraph::new(msg), area);
}
```

- [ ] **Step 5: Build + lints**

```bash
cargo build --features tui --locked
cargo build --locked --no-default-features
cargo clippy --features tui --locked --all-targets -- -D warnings
```

Expected: всё PASS.

- [ ] **Step 6: Commit**

```bash
git add src/tui/overlays/ src/tui/ui.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T10 overlays + ui::draw — themes/help/modal + orchestration

- overlays::help: full keybindings cheatsheet (12 entries, ? to close)
- overlays::themes: side-by-side 5 builtins + 9 globals
- overlays::modal: centered() helper + render_confirm_quit (s/d/c)
- ui::draw: 4-panel grid (Lines/Palette top, Settings/Preview bottom) + status bar
- Status bar: dirty glyph + mode/focus + message (Info/Warn/Error)
- Overlays render last, on top of panels

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
