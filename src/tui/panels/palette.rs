//! Widget palette (top-right) — 61 widgets + filter + categories.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::{App, Pane, PaletteMode};
use crate::tui::ui::panel_block;
use crate::tui::widget_meta::{ALL_KINDS, WidgetCategory, WidgetMeta};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Palette;
    let filter_str = if app.palette_filter.is_empty() {
        "type to filter…".to_string()
    } else {
        format!("filter: {}_", app.palette_filter)
    };

    let filtered: Vec<&WidgetMeta> = filter_meta(&app.palette_filter);

    let mut lines = Vec::with_capacity(filtered.len() + 6);
    lines.push(Line::from(Span::styled(
        filter_str,
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));

    let mut current_cat: Option<WidgetCategory> = None;
    // Row index (in `lines`) of the row corresponding to `palette_cursor`.
    let mut cursor_row: u16 = 0;
    for (i, m) in filtered.iter().enumerate() {
        if Some(m.category) != current_cat {
            lines.push(Line::from(Span::styled(
                format!("── {} ──", m.category.label()),
                Style::default().fg(Color::DarkGray),
            )));
            current_cat = Some(m.category);
        }
        let mark = if i == app.palette_cursor && focused {
            "▶ "
        } else {
            "  "
        };
        let style = if i == app.palette_cursor && focused {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow)
        } else {
            Style::default()
        };
        if i == app.palette_cursor {
            // saturate to u16: lines.len() is bounded by ~80, fits.
            cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        }
        lines.push(Line::from(vec![
            Span::raw(mark),
            Span::styled(m.display, style),
        ]));
    }

    // Visible height = area minus borders (top+bottom).
    let visible = area.height.saturating_sub(2);
    // Reserve top 2 rows (filter + blank) as anchor; scroll only the body.
    let anchor: u16 = 2;
    let total = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let scroll = compute_scroll(cursor_row, total, visible, anchor);

    let title = match app.palette_mode {
        PaletteMode::Add => format!("Widgets ({}/61) · add", filtered.len()),
        PaletteMode::Replace => format!("Widgets ({}/61) · replace", filtered.len()),
    };
    let block = panel_block(title, focused);
    frame.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll, 0)),
        area,
    );
}

/// Returns vertical scroll offset that keeps `cursor_row` visible.
/// `anchor` is the count of leading rows that should stay pinned at the top
/// (filter line + blank). When the cursor is inside the anchor we don't scroll.
fn compute_scroll(cursor_row: u16, total: u16, visible: u16, anchor: u16) -> u16 {
    if visible == 0 || total <= visible {
        return 0;
    }
    if cursor_row < anchor + visible.saturating_sub(anchor) {
        // Cursor fits in the initial viewport — no scroll yet.
        if cursor_row < visible {
            return 0;
        }
    }
    // Center cursor in the body (below anchor).
    let body = visible.saturating_sub(anchor);
    let half = body / 2;
    let want = cursor_row.saturating_sub(anchor + half);
    let max = total.saturating_sub(visible);
    want.min(max)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_zero_when_total_fits() {
        assert_eq!(compute_scroll(0, 10, 20, 2), 0);
        assert_eq!(compute_scroll(9, 10, 20, 2), 0);
    }

    #[test]
    fn scroll_zero_when_cursor_in_initial_viewport() {
        assert_eq!(compute_scroll(0, 80, 20, 2), 0);
        assert_eq!(compute_scroll(15, 80, 20, 2), 0);
    }

    #[test]
    fn scroll_advances_when_cursor_below_viewport() {
        // cursor_row=40, anchor=2, body=18, half=9 → want = 40 - 11 = 29
        // max = 80 - 20 = 60 → 29
        assert_eq!(compute_scroll(40, 80, 20, 2), 29);
    }

    #[test]
    fn scroll_clamps_to_max_at_bottom() {
        // cursor near end → want > max → clamped to max
        assert_eq!(compute_scroll(79, 80, 20, 2), 60);
    }
}
