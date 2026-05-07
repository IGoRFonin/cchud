//! Phase 8 TUI snapshot suite — `ratatui::backend::TestBackend(80, 24)`.

#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use cchud::tui::app::{App, ColorField, EditField, Mode, Pane, Screen};
use cchud::tui::sample;
use cchud::tui::ui;
use cchud::types::config::{Line, Settings};

fn fresh_app() -> App {
    let (p, f) = sample::payload();
    let json = r#"{"lines":[{"widgets":[{"type":"model"}]}]}"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    let mut app = App::new(s, p, f);
    // Legacy snapshots assume EditLines screen; Home/ChoosePreset get new explicit tests.
    app.screen = Screen::EditLines;
    app
}

fn home_app() -> App {
    let (p, f) = sample::payload();
    App::new(Settings::default(), p, f)
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
    app.palette_visible = true;
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

#[test]
fn settings_named_color_picker_open_shows_swatches() {
    use cchud::tui::app::EditField;
    let mut app = fresh_app();
    app.focus = Pane::Settings;
    app.settings_field_cursor = 0;
    app.color_fg_cursor = 2; // Slate (row 0, col 2 in 5×3 Excalidraw grid)
    app.editing_field = Some(EditField::ColorPicker {
        field: ColorField::Foreground,
    });
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn home_initial() {
    let app = home_app();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn choose_preset_first_builtin_selected() {
    let mut app = home_app();
    app.presets = cchud::tui::presets::list_all();
    app.screen = Screen::ChoosePreset;
    app.preset_cursor = 0;
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn confirm_return_home_modal() {
    let mut app = fresh_app();
    app.mode = Mode::ConfirmReturnHome;
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn confirm_install_modal_over_home() {
    let mut app = home_app();
    app.home_cursor = 2;
    app.mode = Mode::ConfirmInstall;
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn preset_name_prompt_with_partial_input() {
    let mut app = fresh_app();
    app.mode = Mode::PresetNamePrompt;
    app.preset_name_buffer = "my-laptop".to_string();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn settings_scrolls_when_custom_command_has_many_args() {
    let (p, f) = sample::payload();
    // CustomCommand с 12-ю args — не помещается в высоту панели.
    let json = r#"{
        "lines":[{"widgets":[{
            "type":"custom-command",
            "command":"echo",
            "timeoutMs":1000,
            "args":["a","b","c","d","e","f","g","h","i","j","k","l"]
        }]}]
    }"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    let mut app = App::new(s, p, f);
    app.screen = Screen::EditLines;
    app.focus = Pane::Settings;
    // Курсор на последнем arg — должен прокрутиться.
    app.settings_field_cursor = 5 + 11;
    insta::assert_snapshot!(render_to_buffer(&app));
}
