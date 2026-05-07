# Task 8 — `tui::widgets_ui` (input, tri_bool, color_picker, number_input, list_editor)

**Цель:** Reusable embedded UI-виджеты, рисуемые из `panels::settings` (T9). Все принимают `&mut Frame, area, &App` и читают/мутируют `App.editing_field` через reducer (T7) — сами они только render-helpers, не event handlers (события идут в reducer).

**Files:**
- Modify: `src/tui/widgets_ui/input.rs` — single-line text input
- Modify: `src/tui/widgets_ui/tri_bool.rs` — tri-state checkbox
- Modify: `src/tui/widgets_ui/color_picker.rs` — dropdown 16 named + Default + Custom hex
- Modify: `src/tui/widgets_ui/number_input.rs` — numeric input + min/max highlight
- Modify: `src/tui/widgets_ui/list_editor.rs` — embedded sub-list (CustomCommand.args)

---

- [ ] **Step 1: `widgets_ui/input.rs` — single-line input**

```rust
//! Single-line text input — Phase 8 Task 8.
//!
//! Render-helper: рисует buffer + cursor caret. Event handling — в reducer (T7).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, buffer: &str, cursor: usize, focused: bool) {
    let (left, mid, right) = split_at_cursor(buffer, cursor);
    let line = Line::from(vec![
        Span::raw(format!("{label} ")),
        Span::raw(left.to_string()),
        Span::styled(
            mid.to_string(),
            Style::default().add_modifier(Modifier::REVERSED),
        ),
        Span::raw(right.to_string()),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(ratatui::style::Color::Yellow)
        } else {
            Style::default()
        });
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn split_at_cursor(s: &str, cursor: usize) -> (&str, &str, &str) {
    let (left, rest) = s.split_at(cursor.min(s.len()));
    if rest.is_empty() {
        (left, " ", "")
    } else {
        // Take 1 char width as caret highlight.
        let head_len = rest
            .chars()
            .next()
            .map_or(0, |c| c.len_utf8());
        let (mid, tail) = rest.split_at(head_len);
        (left, mid, tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_at_cursor_handles_end_of_string() {
        let (l, m, r) = split_at_cursor("abc", 3);
        assert_eq!(l, "abc");
        assert_eq!(m, " ");
        assert_eq!(r, "");
    }

    #[test]
    fn split_at_cursor_highlights_one_char() {
        let (l, m, r) = split_at_cursor("hello", 2);
        assert_eq!(l, "he");
        assert_eq!(m, "l");
        assert_eq!(r, "lo");
    }
}
```

- [ ] **Step 2: `widgets_ui/tri_bool.rs` — tri-state checkbox**

```rust
//! Tri-state bool — Phase 8 Task 8. None / Some(true) / Some(false).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

#[must_use]
pub const fn glyph(value: Option<bool>) -> &'static str {
    match value {
        None => "[ ]",
        Some(true) => "[✓]",
        Some(false) => "[ ✗]",
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, value: Option<bool>, focused: bool) {
    let style = if focused {
        Style::default().fg(ratatui::style::Color::Yellow)
    } else {
        Style::default()
    };
    let text = format!("{label} {}", glyph(value));
    frame.render_widget(Paragraph::new(text).style(style), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_for_three_states() {
        assert_eq!(glyph(None), "[ ]");
        assert_eq!(glyph(Some(true)), "[✓]");
        assert_eq!(glyph(Some(false)), "[ ✗]");
    }
}
```

- [ ] **Step 3: `widgets_ui/color_picker.rs` — 16 named + Default + Custom hex**

```rust
//! Color picker — Phase 8 Task 8. 16 ANSI named + Default + Custom hex.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color as RColor, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub static NAMED_COLORS: &[(&str, &str)] = &[
    ("Default (none)", ""),
    ("Black",          "#000000"),
    ("Red",            "#cd0000"),
    ("Green",          "#00cd00"),
    ("Yellow",         "#cdcd00"),
    ("Blue",           "#0000ee"),
    ("Magenta",        "#cd00cd"),
    ("Cyan",           "#00cdcd"),
    ("White",          "#e5e5e5"),
    ("Bright Black",   "#7f7f7f"),
    ("Bright Red",     "#ff0000"),
    ("Bright Green",   "#00ff00"),
    ("Bright Yellow",  "#ffff00"),
    ("Bright Blue",    "#5c5cff"),
    ("Bright Magenta", "#ff00ff"),
    ("Bright Cyan",    "#00ffff"),
    ("Bright White",   "#ffffff"),
    ("Custom hex…",    ""),
];

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, current: Option<&str>, cursor: usize, focused: bool) {
    let mut lines = Vec::with_capacity(NAMED_COLORS.len() + 1);
    lines.push(Line::from(format!("{label} = {}", current.unwrap_or("(none)"))));
    for (i, (name, hex)) in NAMED_COLORS.iter().enumerate() {
        let mark = if i == cursor && focused { "▶ " } else { "  " };
        let swatch = if hex.is_empty() {
            Span::raw("  ")
        } else if let Some(c) = parse_hex(hex) {
            Span::styled("██", Style::default().fg(c))
        } else {
            Span::raw("  ")
        };
        lines.push(Line::from(vec![
            Span::raw(mark),
            swatch,
            Span::raw(format!(" {name}")),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn parse_hex(s: &str) -> Option<RColor> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(RColor::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_colors_has_18_entries() {
        // 16 ANSI + Default + Custom hex
        assert_eq!(NAMED_COLORS.len(), 18);
    }

    #[test]
    fn parse_hex_valid_returns_rgb() {
        assert_eq!(parse_hex("#aabbcc"), Some(RColor::Rgb(0xaa, 0xbb, 0xcc)));
    }

    #[test]
    fn parse_hex_invalid_returns_none() {
        assert!(parse_hex("zzz").is_none());
        assert!(parse_hex("#xx").is_none());
    }
}
```

- [ ] **Step 4: `widgets_ui/number_input.rs`**

```rust
//! Numeric input + min/max validation — Phase 8 Task 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, buffer: &str, min: u64, max: u64, focused: bool) {
    let valid = match buffer.parse::<u64>() {
        Ok(n) => (min..=max).contains(&n),
        Err(_) => buffer.is_empty(),
    };
    let bg = if !valid {
        Style::default().bg(Color::Red)
    } else if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let text = format!("{label} {buffer}_  (range {min}–{max})");
    frame.render_widget(Paragraph::new(text).style(bg), area);
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_compiles() {
        // Render-helper смок-тест на полноценном `Frame` потребует ratatui::TestBackend
        // — покрытие в `tests/tui_snapshots.rs` (T14). Здесь — sanity-check на module link.
        let _ = super::render;
    }
}
```

- [ ] **Step 5: `widgets_ui/list_editor.rs`**

```rust
//! Embedded sub-list editor (CustomCommand.args) — Phase 8 Task 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame<'_>, area: Rect, label: &str, items: &[String], cursor: usize, focused: bool) {
    let mut lines = Vec::with_capacity(items.len() + 1);
    lines.push(Line::from(format!("{label}  ({} items)", items.len())));
    for (i, item) in items.iter().enumerate() {
        let mark = if i == cursor && focused { "▶ " } else { "  " };
        lines.push(Line::from(format!("{mark}{i:>2}: {item}")));
    }
    if items.is_empty() {
        lines.push(Line::from("  (empty — press Enter to add)"));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_compiles() {
        let _ = super::render;
    }
}
```

- [ ] **Step 6: Run tests**

```bash
cargo test --features tui --locked tui::widgets_ui
```

Expected: PASS — все unit-тесты в widgets_ui.

- [ ] **Step 7: Verify --no-default-features build + lints**

```bash
cargo build --locked --no-default-features
cargo clippy --locked --features tui --all-targets -- -D warnings
```

Expected: оба PASS.

- [ ] **Step 8: Commit**

```bash
git add src/tui/widgets_ui/
git commit -m "$(cat <<'EOF'
feat(phase-8): T8 widgets_ui — input/tri_bool/color_picker/number_input/list_editor

- input.rs: single-line buffer + cursor caret highlight (REVERSED modifier)
- tri_bool.rs: glyphs [ ] / [✓] / [ ✗]
- color_picker.rs: 16 ANSI named + Default + Custom hex; swatch preview
- number_input.rs: red bg on invalid range + min/max display
- list_editor.rs: indexed list + cursor + empty-state hint

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
