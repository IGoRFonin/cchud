//! Top-level draw orchestration.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use crate::tui::app::{App, MessageKind, Mode, Screen};
use crate::tui::overlays;
use crate::tui::panels;
use crate::tui::screens;

/// Левый отступ для status bar и Preview-панели — единый визуальный gutter
/// который выравнивает читаемые тексты с границей панелей.
pub const LEFT_GUTTER: u16 = 2;

/// Стандартная рамка панели — rounded углы, dim-серая обводка
/// (yellow + bold когда `focused`), боковые отступы 1 col, заголовок
/// в формате ` {title} ` слева. Используется во всех 4 panel'ах.
pub fn panel_block<'a>(title: impl Into<std::borrow::Cow<'a, str>>, focused: bool) -> Block<'a> {
    let title = title.into().into_owned();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .title(format!(" {title} "))
        .border_style(if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
}

/// Рамка для оверлеев (Help/Themes/Modal). Тот же стиль, что у `panel_block`,
/// но всегда без focus-выделения (overlay сам по себе уже вывод во фронт).
pub fn overlay_block<'a>(title: impl Into<std::borrow::Cow<'a, str>>) -> Block<'a> {
    let title = title.into().into_owned();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .title(format!(" {title} "))
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
}

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let main = outer[0];
    let status = outer[1];

    match app.screen {
        Screen::Home => screens::home::render(frame, main, app),
        Screen::EditLines => render_edit_lines(frame, main, app),
        Screen::ChoosePreset => screens::choose_preset::render(frame, main, app),
    }

    // Status bar.
    render_status_bar(frame, status, app);

    // Overlays поверх всего.
    match app.mode {
        Mode::Edit => {}
        Mode::ThemesOverlay => overlays::themes::render(frame, area, app),
        Mode::HelpOverlay => overlays::help::render(frame, area),
        Mode::ConfirmQuit => overlays::modal::render_confirm_quit(frame, area),
        Mode::ConfirmReturnHome => overlays::modal::render_confirm_return_home(frame, area),
        Mode::PresetNamePrompt => {
            overlays::modal::render_preset_name_prompt(frame, area, &app.preset_name_buffer);
        }
    }
}

fn render_edit_lines(frame: &mut Frame<'_>, main: ratatui::layout::Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    if app.palette_visible {
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(rows[0]);
        panels::lines::render(frame, top[0], app);
        panels::palette::render(frame, top[1], app);
    } else {
        panels::lines::render(frame, rows[0], app);
    }
    panels::settings::render(frame, bottom[0], app);
    panels::preview::render(frame, bottom[1], app);
}


fn render_status_bar(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(LEFT_GUTTER), Constraint::Min(0)])
        .split(area);
    let inner = cols[1];

    let dirty_glyph = if app.dirty() { "● " } else { "" };
    let msg = match &app.status_message {
        Some((m, MessageKind::Info)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Green))
        }
        Some((m, MessageKind::Warn)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Yellow))
        }
        Some((m, MessageKind::Error)) => {
            Line::from(m.clone()).style(Style::default().fg(Color::Red))
        }
        None => {
            // Persistent hint with the most useful keys. `Tab` is the discovery
            // anchor — without it users get stuck in the Lines panel.
            let dim = Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM);
            let key = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled(dirty_glyph, Style::default().fg(Color::Yellow)),
                Span::styled("Tab", key),
                Span::styled(" switch panel  ", dim),
                Span::styled("?", key),
                Span::styled(" help  ", dim),
                Span::styled("^S", key),
                Span::styled(" save  ", dim),
                Span::styled("q", key),
                Span::styled(" quit", dim),
            ])
        }
    };
    frame.render_widget(Paragraph::new(msg), inner);
}
