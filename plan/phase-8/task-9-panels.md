# Task 9 — `tui::panels` (Lines / Palette / Settings / Preview)

**Цель:** Четыре render-функции, по одной на панель. Каждая принимает `frame: &mut Frame, area: Rect, app: &App` и рендерит через ratatui Block + Paragraph + List/Table. Preview использует `Renderer::for_preview` + `Renderer::compose_line` + `style_map::to_span` (T2/T3) — **no ANSI rendering, никаких escape-последовательностей в TUI**.

**Files:**
- Modify: `src/tui/panels/lines.rs`
- Modify: `src/tui/panels/palette.rs`
- Modify: `src/tui/panels/settings.rs`
- Modify: `src/tui/panels/preview.rs`

---

- [ ] **Step 1: `panels/lines.rs`**

```rust
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
    // Round-trip через serde чтобы получить kebab `type`.
    let item = WidgetItem { kind: cfg.clone(), style: WidgetStyleOverride::default() };
    if let Ok(v) = serde_json::to_value(&item) {
        if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
            // Fallback display name из ALL_KINDS если найдём.
            return ALL_KINDS.iter().find(|m| m.kebab_type == t).map_or(t.to_string(), |m| m.display.to_string());
        }
    }
    "<unknown>".into()
}
```

- [ ] **Step 2: `panels/palette.rs`**

```rust
//! Widget palette (top-right) — 60 widgets + filter + categories.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{App, EditField, Pane};
use crate::tui::widget_meta::{ALL_KINDS, WidgetCategory, WidgetMeta};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Palette;
    let in_filter_edit = matches!(app.editing_field, Some(EditField::PaletteFilter));

    let filter_str = if in_filter_edit {
        format!("/ {}_", app.palette_filter)
    } else if app.palette_filter.is_empty() {
        "press / to filter".into()
    } else {
        format!("/ {}", app.palette_filter)
    };

    let filtered: Vec<&WidgetMeta> = filter_meta(&app.palette_filter);

    let mut lines = Vec::with_capacity(filtered.len() + 6);
    lines.push(Line::from(Span::styled(filter_str, Style::default().fg(Color::Cyan))));
    lines.push(Line::from(""));

    let mut current_cat: Option<WidgetCategory> = None;
    for (i, m) in filtered.iter().enumerate() {
        if Some(m.category) != current_cat {
            lines.push(Line::from(Span::styled(format!("── {} ──", m.category.label()),
                Style::default().fg(Color::DarkGray))));
            current_cat = Some(m.category);
        }
        let mark = if i == app.palette_cursor && focused { "▶ " } else { "  " };
        let style = if i == app.palette_cursor && focused {
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![Span::raw(mark), Span::styled(m.display, style)]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Widgets ({}/60)", filtered.len()))
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn filter_meta(filter: &str) -> Vec<&'static WidgetMeta> {
    let needle = filter.trim().to_lowercase();
    if needle.is_empty() {
        return ALL_KINDS.iter().collect();
    }
    ALL_KINDS
        .iter()
        .filter(|m| {
            m.display.to_lowercase().contains(&needle)
                || m.kebab_type.contains(&needle)
                || m.category.label().to_lowercase().contains(&needle)
        })
        .collect()
}
```

- [ ] **Step 3: `panels/settings.rs`**

Минимально: показывает текущий widget kind, color/background_color/bold (tri-state) + custom params (через kind matching). Для embedded inputs дёргает `widgets_ui::*`. Пример скелета:

```rust
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

    // Custom params section.
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
```

- [ ] **Step 4: `panels/preview.rs`** — единая точка интеграции `compose_line` + `style_map::to_span`

```rust
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
        for (i, line_widgets) in lines.iter().enumerate() {
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
            // Skip empty rendered lines except first.
            if i + 1 < lines.len() {
                // No-op; ratatui Paragraph join via newline anyway.
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });
    frame.render_widget(Paragraph::new(tui_lines).block(block), area);
}
```

- [ ] **Step 5: Build + lints**

```bash
cargo build --features tui --locked
cargo build --locked --no-default-features
cargo clippy --features tui --all-targets --locked -- -D warnings
```

Expected: всё PASS. Полный rendering упражняется в T14 snapshot тестах; здесь проверяем только compile-clean.

- [ ] **Step 6: Commit**

```bash
git add src/tui/panels/
git commit -m "$(cat <<'EOF'
feat(phase-8): T9 panels — lines/palette/settings/preview

- panels::lines: line list + nested widgets + selected_line/widget cursors + footer hints
- panels::palette: filter "/", category headers, 60 widgets, palette_cursor highlight
- panels::settings: color picker (FG/BG) + tri-bool Bold + custom params per WidgetConfig kind
- panels::preview: Renderer::for_preview + compose_line + style_map::to_span (no ANSI in TUI)
- Border yellow when focused; sample renders against `app.sample_payload`

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
