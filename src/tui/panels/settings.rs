//! Widget settings panel (bottom-left) — color/background/bold + custom params.
//!
//! Все строки настройки строятся как `Vec<Line>` и рендерятся одним `Paragraph`
//! со скроллом. Это даёт корректный overflow-handling, когда у виджета много
//! параметров (например `CustomCommand` с длинным `args`) или когда раскрыт
//! color-picker (18+ строк).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::{App, ColorField, EditField, Pane, SettingsField};
use crate::tui::ui::panel_block;

const TIMEOUT_MIN_MS: u64 = 50;
const TIMEOUT_MAX_MS: u64 = 5000;
use crate::tui::widgets_ui::color_picker::{
    self, EXCALIDRAW_PALETTE, GRID_COLS, GRID_LEN, GRID_ROWS, IDX_CUSTOM, IDX_DEFAULT,
};
use crate::tui::widgets_ui::tri_bool;
use crate::types::config::WidgetConfig;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Pane::Settings;
    let block = panel_block("Widget Settings", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(item) = app.current_widget() else {
        let lines = vec![
            Line::from(Span::styled(
                "(no widget selected)",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Pick a widget in the Lines pane (←→ between lines, ↑↓ between widgets).",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Press Tab to focus this panel, Enter on a row to edit.",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    };

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    render_hint(frame, body[0], app);

    let cursor = app.settings_field_cursor;
    let (lines, cursor_row) = build_lines(app, &item.kind, cursor, focused);

    let total = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let visible = body[1].height;
    let scroll = compute_scroll(cursor_row, total, visible);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), body[1]);
}

fn render_hint(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let txt = if app.editing_field.is_some() {
        "edit · ↑↓ navigate · Enter apply · Esc cancel"
    } else if app.focus == Pane::Settings {
        "↑↓ select · Enter edit · Space toggle bold/raw · Esc back to Lines"
    } else {
        "Tab to focus · then ↑↓ select · Enter to edit"
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            txt,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )),
        area,
    );
}

fn compute_scroll(cursor_row: u16, total: u16, visible: u16) -> u16 {
    if visible == 0 || total <= visible {
        return 0;
    }
    let max = total.saturating_sub(visible);
    if cursor_row >= visible {
        cursor_row
            .saturating_sub(visible.saturating_sub(1))
            .min(max)
    } else {
        0
    }
}

/// Build the entire settings body as a flat list of styled lines plus the
/// vertical row index of the visual cursor (so the outer Paragraph can scroll
/// to keep it in view). Each settings field becomes 1+ rows; expanded color
/// picker contributes ~18 rows.
fn build_lines<'a>(
    app: &'a App,
    kind: &'a WidgetConfig,
    cursor: usize,
    focused: bool,
) -> (Vec<Line<'a>>, u16) {
    let item = app.current_widget();
    let style_color = item.and_then(|i| i.style.color.as_deref());
    let style_bg = item.and_then(|i| i.style.background_color.as_deref());
    let style_bold = item.and_then(|i| i.style.bold);

    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut cursor_row: u16 = 0;

    // Row 0: Foreground / Text color.
    let fg_focused = focused && cursor == 0;
    let fg_expanded = is_editing_color(app, ColorField::Foreground);
    let fg_hex_buf = hex_buffer_for(app, ColorField::Foreground);
    if fg_focused && !fg_expanded {
        cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    push_color_section(
        &mut lines,
        &mut cursor_row,
        "Text color",
        style_color,
        app.color_fg_cursor,
        app.color_fg_shade,
        fg_focused,
        fg_expanded,
        fg_hex_buf,
    );

    // Row 1: Background.
    let bg_focused = focused && cursor == 1;
    let bg_expanded = is_editing_color(app, ColorField::Background);
    let bg_hex_buf = hex_buffer_for(app, ColorField::Background);
    if bg_focused && !bg_expanded {
        cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    push_color_section(
        &mut lines,
        &mut cursor_row,
        "Background",
        style_bg,
        app.color_bg_cursor,
        app.color_bg_shade,
        bg_focused,
        bg_expanded,
        bg_hex_buf,
    );

    // Row 2: Bold (tri-state).
    let bold_focused = focused && cursor == 2;
    if bold_focused {
        cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    lines.push(bold_line(style_bold, bold_focused));

    // Row 3 (only for widgets with inherent prefix/icon): Raw value toggle.
    let supports_raw = crate::widgets::widget_supports_raw_value(kind);
    if supports_raw {
        let raw_focused = focused && cursor == 3;
        if raw_focused {
            cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        }
        let raw_value = item.is_some_and(|i| i.raw_value);
        lines.push(raw_value_line(raw_value, raw_focused));
    }

    // Spacer before kind-specific params.
    lines.push(Line::from(""));

    push_kind_params(&mut lines, &mut cursor_row, app, kind, cursor, focused);

    (lines, cursor_row)
}

fn is_editing_color(app: &App, field: ColorField) -> bool {
    match &app.editing_field {
        Some(EditField::ColorPicker { field: f } | EditField::ColorHex { field: f, .. }) => {
            *f == field
        }
        _ => false,
    }
}

fn hex_buffer_for(app: &App, field: ColorField) -> Option<&str> {
    if let Some(EditField::ColorHex {
        field: f, buffer, ..
    }) = &app.editing_field
    {
        if *f == field {
            return Some(buffer.as_str());
        }
    }
    None
}

fn parse_hex_to_color(hex: &str) -> Option<Color> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// Returns ("Name (#hex)" string, optional swatch color).
fn describe_color(value: Option<&str>) -> (String, Option<Color>) {
    let hex = match value {
        None | Some("") => return ("(default)".to_string(), None),
        Some(h) => h,
    };
    let label = color_picker::label_for_hex(hex)
        .map_or_else(|| hex.to_string(), |n| format!("{n} ({hex})"));
    (label, parse_hex_to_color(hex))
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn push_color_section<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    label: &'a str,
    current: Option<&'a str>,
    picker_cursor: usize,
    active_shade: usize,
    focused: bool,
    expanded: bool,
    hex_buffer: Option<&'a str>,
) {
    if !expanded {
        lines.push(collapsed_color_line(label, current, focused));
        return;
    }

    // Expanded — render the picker (or hex sub-input) inline.
    if let Some(buf) = hex_buffer {
        // ColorHex sub-mode: single-line text input. Cursor sits on this row.
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        lines.push(Line::from(vec![
            Span::raw("▶ "),
            Span::styled(
                format!("{label}: {buf}_  (hex, Enter to apply)"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        return;
    }

    // Header row.
    let (current_label, swatch) = describe_color(current);
    let mut header: Vec<Span<'a>> = vec![
        Span::raw("▶ "),
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if let Some(c) = swatch {
        header.push(Span::styled("██ ", Style::default().fg(c)));
    }
    header.push(Span::styled(
        current_label,
        Style::default().fg(Color::Yellow),
    ));
    lines.push(Line::from(header));

    // 5×3 grid of round swatches with 2-char gap between cells.
    for row in 0..GRID_ROWS {
        let mut spans: Vec<Span<'a>> = vec![Span::raw("  ")];
        let mut row_has_cursor = false;
        for col in 0..GRID_COLS {
            if col > 0 {
                spans.push(Span::raw("  "));
            }
            let idx = row * GRID_COLS + col;
            let is_sel = idx == picker_cursor;
            if is_sel {
                row_has_cursor = true;
            }
            let entry = &EXCALIDRAW_PALETTE[idx];
            let swatch_color = parse_hex_to_color(entry.base());
            let (l, r) = if is_sel { ("[", "]") } else { (" ", " ") };
            let bracket_style = if is_sel {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(l, bracket_style));
            if let Some(c) = swatch_color {
                spans.push(Span::styled("●●", Style::default().fg(c)));
            } else {
                spans.push(Span::raw("●●"));
            }
            spans.push(Span::styled(r, bracket_style));
        }
        if row_has_cursor {
            *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        }
        lines.push(Line::from(spans));
    }

    // Shades row — only when cursor is on a grid cell. Подсветка активного shade'а.
    if picker_cursor < GRID_LEN {
        let entry = &EXCALIDRAW_PALETTE[picker_cursor];
        let mut spans: Vec<Span<'a>> = vec![Span::styled(
            " Shades ",
            Style::default().fg(Color::DarkGray),
        )];
        for (n, hex) in entry.shades.iter().enumerate() {
            if n > 0 {
                spans.push(Span::raw(" "));
            }
            let is_active = n == active_shade;
            let (l, r) = if is_active { ("[", "]") } else { (" ", " ") };
            let bracket_style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(l, bracket_style));
            if let Some(c) = parse_hex_to_color(hex) {
                spans.push(Span::styled("●", Style::default().fg(c)));
            } else {
                spans.push(Span::raw("●"));
            }
            spans.push(Span::styled(
                format!("{}", n + 1),
                if is_active {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ));
            spans.push(Span::styled(r, bracket_style));
        }
        lines.push(Line::from(spans));
    } else {
        lines.push(Line::from(""));
    }

    // Default (none) row.
    let default_sel = picker_cursor == IDX_DEFAULT;
    if default_sel {
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let mark = if default_sel { "  ▶ " } else { "    " };
    let style = if default_sel {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![
        Span::raw(mark),
        Span::styled("Default (none)", style),
    ]));

    // Custom hex row.
    let custom_sel = picker_cursor == IDX_CUSTOM;
    if custom_sel {
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let mark = if custom_sel { "  ▶ " } else { "    " };
    let style = if custom_sel {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![
        Span::raw(mark),
        Span::styled("Custom hex…", style),
    ]));
}

fn collapsed_color_line<'a>(label: &'a str, current: Option<&'a str>, focused: bool) -> Line<'a> {
    let mark = if focused { "▶ " } else { "  " };
    let label_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let (value_label, swatch) = describe_color(current);
    let mut spans: Vec<Span<'a>> = vec![
        Span::raw(mark),
        Span::styled(format!("{label}: "), label_style),
    ];
    if let Some(c) = swatch {
        spans.push(Span::styled("██ ", Style::default().fg(c)));
    }
    let value_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    spans.push(Span::styled(value_label, value_style));
    Line::from(spans)
}

fn raw_value_line(value: bool, focused: bool) -> Line<'static> {
    let mark = if focused { "▶ " } else { "  " };
    let label_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let value_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let glyph = if value { "[✓]" } else { "[ ]" };
    let label_value = if value { "on (drop prefix)" } else { "off" };
    Line::from(vec![
        Span::raw(mark),
        Span::styled("Raw value: ", label_style),
        Span::styled(glyph, value_style),
        Span::raw(" "),
        Span::styled(label_value, value_style),
    ])
}

fn bold_line(value: Option<bool>, focused: bool) -> Line<'static> {
    let mark = if focused { "▶ " } else { "  " };
    let label_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let value_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let glyph = tri_bool::glyph(value);
    let label_value = match value {
        None => "default",
        Some(true) => "on",
        Some(false) => "off",
    };
    Line::from(vec![
        Span::raw(mark),
        Span::styled("Bold: ", label_style),
        Span::styled(glyph, value_style),
        Span::raw(" "),
        Span::styled(label_value, value_style),
    ])
}

#[allow(clippy::too_many_lines)]
fn push_kind_params<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    app: &'a App,
    kind: &'a WidgetConfig,
    cursor: usize,
    focused: bool,
) {
    match kind {
        WidgetConfig::CustomText { params } => {
            push_text_field(
                lines,
                cursor_row,
                "Text",
                &params.text,
                app,
                SettingsField::CustomTextText,
                focused && cursor == 3,
            );
        }
        WidgetConfig::CustomSymbol { params } => {
            push_text_field(
                lines,
                cursor_row,
                "Symbol",
                &params.symbol,
                app,
                SettingsField::CustomSymbolSymbol,
                focused && cursor == 3,
            );
        }
        WidgetConfig::Link { params } => {
            push_text_field(
                lines,
                cursor_row,
                "URL",
                &params.url,
                app,
                SettingsField::LinkUrl,
                focused && cursor == 3,
            );
            push_text_field(
                lines,
                cursor_row,
                "Label",
                params.label.as_deref().unwrap_or(""),
                app,
                SettingsField::LinkLabel,
                focused && cursor == 4,
            );
        }
        WidgetConfig::CustomCommand { params } => {
            push_text_field(
                lines,
                cursor_row,
                "Command",
                &params.command,
                app,
                SettingsField::CustomCommandCommand,
                focused && cursor == 3,
            );
            push_number_field(
                lines,
                cursor_row,
                "Timeout ms",
                params.timeout_ms.to_string(),
                app,
                SettingsField::CustomCommandTimeoutMs,
                TIMEOUT_MIN_MS,
                TIMEOUT_MAX_MS,
                focused && cursor == 4,
            );
            push_args_list(lines, cursor_row, &params.args, cursor, focused);
        }
        WidgetConfig::ContextBar { params } => {
            push_number_field(
                lines,
                cursor_row,
                "Width",
                params.width.to_string(),
                app,
                SettingsField::ContextBarWidth,
                1,
                80,
                focused && cursor == 3,
            );
        }
        WidgetConfig::CurrentWorkingDir { params } => {
            let segments_value = params
                .segments
                .map_or_else(|| "(off)".to_string(), |n| n.to_string());
            push_number_field(
                lines,
                cursor_row,
                "Segments",
                segments_value,
                app,
                SettingsField::CurrentWorkingDirSegments,
                0,
                10,
                focused && cursor == 3,
            );
            push_bool_row(
                lines,
                cursor_row,
                "Abbreviate home",
                params.abbreviate_home,
                focused && cursor == 4,
            );
            push_bool_row(
                lines,
                cursor_row,
                "Fish style",
                params.fish_style,
                focused && cursor == 5,
            );
            push_text_field(
                lines,
                cursor_row,
                "Prefix",
                params.prefix.as_deref().unwrap_or(""),
                app,
                SettingsField::CurrentWorkingDirPrefix,
                focused && cursor == 6,
            );
        }
        _ => {
            lines.push(Line::from(Span::styled(
                "(no custom params)",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
}

fn editing_text_buffer(app: &App, field: SettingsField) -> Option<&str> {
    if let Some(EditField::Text {
        field: f, buffer, ..
    }) = &app.editing_field
    {
        if *f == field {
            return Some(buffer.as_str());
        }
    }
    None
}

fn editing_number_buffer(app: &App, field: SettingsField) -> Option<&str> {
    if let Some(EditField::Number { field: f, buffer }) = &app.editing_field {
        if *f == field {
            return Some(buffer.as_str());
        }
    }
    None
}

fn push_text_field<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    label: &'a str,
    value: &'a str,
    app: &'a App,
    field: SettingsField,
    focused: bool,
) {
    if focused {
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let mark = if focused { "▶ " } else { "  " };
    let line = editing_text_buffer(app, field).map_or_else(
        || {
            let label_style = if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            let value_style = if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::raw(mark),
                Span::styled(format!("{label}: "), label_style),
                Span::styled(value, value_style),
            ])
        },
        |buf| {
            Line::from(vec![
                Span::raw(mark),
                Span::styled(
                    format!("{label}: {buf}_"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        },
    );
    lines.push(line);
}

#[allow(clippy::too_many_arguments)]
fn push_number_field<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    label: &'a str,
    value: String,
    app: &'a App,
    field: SettingsField,
    min: u64,
    max: u64,
    focused: bool,
) {
    if focused {
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let mark = if focused { "▶ " } else { "  " };
    if let Some(buf) = editing_number_buffer(app, field) {
        let valid = buf
            .parse::<u64>()
            .map_or(buf.is_empty(), |n| (min..=max).contains(&n));
        let style = if valid {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::Red).fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::raw(mark),
            Span::styled(format!("{label}: {buf}_  ({min}–{max})"), style),
        ]));
        return;
    }
    let label_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let value_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![
        Span::raw(mark),
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value, value_style),
        Span::styled(
            format!("  ({min}–{max})"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
}

fn push_bool_row<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    label: &'a str,
    value: bool,
    focused: bool,
) {
    if focused {
        *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    }
    let mark = if focused { "▶ " } else { "  " };
    let label_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let value_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let glyph = if value { "[✓]" } else { "[ ]" };
    let label_value = if value { "on" } else { "off" };
    lines.push(Line::from(vec![
        Span::raw(mark),
        Span::styled(format!("{label}: "), label_style),
        Span::styled(glyph, value_style),
        Span::raw(" "),
        Span::styled(label_value, value_style),
    ]));
}

fn push_args_list<'a>(
    lines: &mut Vec<Line<'a>>,
    cursor_row: &mut u16,
    args: &'a [String],
    cursor: usize,
    focused: bool,
) {
    lines.push(Line::from(Span::styled(
        format!("Args ({} items):", args.len()),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    if args.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        )));
        return;
    }
    // CustomCommand args occupy cursor positions 5..5+args.len()
    for (i, arg) in args.iter().enumerate() {
        let arg_focused = focused && cursor == 5 + i;
        if arg_focused {
            *cursor_row = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        }
        let mark = if arg_focused { "▶ " } else { "  " };
        let style = if arg_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::raw(mark),
            Span::styled(format!("{i:>2}: {arg}"), style),
        ]));
    }
}
