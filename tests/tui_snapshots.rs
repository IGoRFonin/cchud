//! Phase 8 TUI snapshot suite — `ratatui::backend::TestBackend(80, 24)`.

#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use cchud::tui::app::{App, ColorField, EditField, Mode, Pane};
use cchud::tui::sample;
use cchud::tui::ui;
use cchud::types::config::{Line, Settings};

fn fresh_app() -> App {
    let (p, f) = sample::payload();
    let json = r#"{"lines":[{"widgets":[{"type":"model"}]}]}"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    App::new(s, p, f)
}

fn render_to_buffer(app: &App) -> String {
    let backend = TestBackend::new(80, 24);
    let mut term = Terminal::new(backend).unwrap();
    term.draw(|f| ui::draw(f, app)).unwrap();
    format!("{:?}", term.backend().buffer())
}

#[test]
fn initial_state() {
    let app = fresh_app();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn palette_filtered_by_git() {
    let mut app = fresh_app();
    app.focus = Pane::Palette;
    app.palette_filter = "git".into();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn settings_with_color_picker_open() {
    let mut app = fresh_app();
    app.focus = Pane::Settings;
    app.settings_field_cursor = 0;
    app.editing_field = Some(EditField::ColorHex {
        field: ColorField::Foreground,
        buffer: "#aabbcc".into(),
        cursor: 7,
    });
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn themes_overlay_open() {
    let mut app = fresh_app();
    app.mode = Mode::ThemesOverlay;
    app.theme_field_cursor = 1; // Dracula
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn help_overlay_open() {
    let mut app = fresh_app();
    app.mode = Mode::HelpOverlay;
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn confirm_quit_modal_when_dirty() {
    let mut app = fresh_app();
    app.editable.lines.push(Line::default());
    app.mode = Mode::ConfirmQuit;
    insta::assert_snapshot!(render_to_buffer(&app));
}
