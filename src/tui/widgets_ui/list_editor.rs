//! Embedded sub-list editor (CustomCommand.args).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    items: &[String],
    cursor: usize,
    focused: bool,
) {
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
