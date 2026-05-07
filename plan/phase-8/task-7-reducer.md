# Task 7 — `tui::reducer` (pure handle_key + ≥15 unit-тестов)

**Цель:** Pure reducer `handle_key(&mut App, KeyEvent) -> ReducerEffect`. Без файлового/терминал IO. Полный switch на `app.mode × key.code`. Покрытие тестами — по 1+ на каждый transition. Это сердце TUI логики; чем больше unit-тестов здесь, тем меньше bugs в event loop.

**Files:**
- Modify: `src/tui/effects.rs` — `ReducerEffect`
- Modify: `src/tui/reducer.rs` — `handle_key` + ≥15 unit-тестов

---

- [ ] **Step 1: `ReducerEffect` enum**

В `src/tui/effects.rs`:

```rust
//! Reducer effects — Phase 8 Task 7.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReducerEffect {
    /// Никаких side effects, продолжаем event loop.
    None,
    /// User просит выход без save (нет dirty или подтвердил discard).
    Quit,
    /// User в `ConfirmQuit` нажал `s` — event loop делает save + exit.
    RequestSaveAndQuit,
    /// User в `ConfirmQuit` нажал `d` — event loop discard'ит и exit'ит.
    RequestDiscardAndQuit,
    /// Hint event loop'у что preview надо перерисовать. Сейчас preview lazy
    /// (на каждом draw frame) — оставлено как hint для future оптимизаций.
    RebuildPreview,
}
```

- [ ] **Step 2: `handle_key` skeleton**

В `src/tui/reducer.rs`:

```rust
//! Pure reducer — Phase 8 Task 7.
//!
//! `handle_key(&mut App, KeyEvent) -> ReducerEffect` — единственная mutating-точка
//! для App'а. Никакого file/terminal IO. Save/Quit/Discard поднимаются как
//! `ReducerEffect` и обрабатываются в event loop (T12).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, EditField, Mode, Pane, SettingsField};
use crate::tui::effects::ReducerEffect;
use crate::tui::widget_meta::{ALL_KINDS, WidgetMeta};
use crate::types::config::{Line, WidgetItem, WidgetStyleOverride};

/// Pure key dispatcher. Возвращает effect для event loop.
pub fn handle_key(app: &mut App, key: KeyEvent) -> ReducerEffect {
    // Overlays перехватывают первыми.
    match app.mode {
        Mode::HelpOverlay => return handle_help(app, key),
        Mode::ThemesOverlay => return handle_themes(app, key),
        Mode::ConfirmQuit => return handle_confirm_quit(app, key),
        Mode::Edit => {}
    }

    // Edit-mode внутри poll-строки (PaletteFilter / Text / Number / ColorHex).
    if app.editing_field.is_some() {
        return handle_editing(app, key);
    }

    // Глобальные key-биндинги в Edit (focus-independent).
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), m) if !m.contains(KeyModifiers::SHIFT) => return quit_or_confirm(app),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => return quit_or_confirm(app),
        (KeyCode::Char('s'), m) if m.contains(KeyModifiers::CONTROL) => {
            return ReducerEffect::RequestSaveAndQuit;
        }
        (KeyCode::Char('?'), _) => {
            app.mode = Mode::HelpOverlay;
            return ReducerEffect::None;
        }
        (KeyCode::Char('t'), _) => {
            app.mode = Mode::ThemesOverlay;
            return ReducerEffect::None;
        }
        (KeyCode::Tab, _) => {
            cycle_focus(app, false);
            return ReducerEffect::None;
        }
        (KeyCode::BackTab, _) => {
            cycle_focus(app, true);
            return ReducerEffect::None;
        }
        _ => {}
    }

    // Pane-specific.
    match app.focus {
        Pane::Lines => handle_lines(app, key),
        Pane::Palette => handle_palette(app, key),
        Pane::Settings => handle_settings(app, key),
        Pane::Preview => ReducerEffect::None,
    }
}

fn cycle_focus(app: &mut App, backward: bool) {
    let order = [Pane::Lines, Pane::Palette, Pane::Settings, Pane::Preview];
    let idx = order.iter().position(|p| *p == app.focus).unwrap_or(0);
    let next = if backward {
        (idx + order.len() - 1) % order.len()
    } else {
        (idx + 1) % order.len()
    };
    app.focus = order[next];
}

fn quit_or_confirm(app: &mut App) -> ReducerEffect {
    if app.dirty() {
        app.mode = Mode::ConfirmQuit;
        ReducerEffect::None
    } else {
        ReducerEffect::Quit
    }
}

fn handle_help(app: &mut App, _key: KeyEvent) -> ReducerEffect {
    // Любая клавиша закрывает help.
    app.mode = Mode::Edit;
    ReducerEffect::None
}

fn handle_confirm_quit(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Char('s') => ReducerEffect::RequestSaveAndQuit,
        KeyCode::Char('d') => ReducerEffect::RequestDiscardAndQuit,
        KeyCode::Char('c') | KeyCode::Esc => {
            app.mode = Mode::Edit;
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

fn handle_themes(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Char('t') | KeyCode::Esc => {
            app.mode = Mode::Edit;
            ReducerEffect::None
        }
        KeyCode::Up => {
            app.theme_field_cursor = app.theme_field_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            app.theme_field_cursor = app.theme_field_cursor.saturating_add(1);
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

fn handle_lines(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match (key.code, key.modifiers) {
        (KeyCode::Up, m) if m.is_empty() => {
            app.selected_line = app.selected_line.saturating_sub(1);
            sync_selected_widget(app);
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Down, m) if m.is_empty() => {
            let max = app.editable.lines.len().saturating_sub(1);
            if app.selected_line < max {
                app.selected_line += 1;
                sync_selected_widget(app);
            }
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Char('a'), _) => {
            app.editable.lines.push(Line::default());
            app.selected_line = app.editable.lines.len().saturating_sub(1);
            app.selected_widget = None;
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Char('d') | KeyCode::Delete, _) => {
            // Удаляет выбранный widget; если пустая линия — удаляет линию.
            if let Some(idx) = app.selected_widget {
                if let Some(line) = app.editable.lines.get_mut(app.selected_line) {
                    if idx < line.widgets.len() {
                        line.widgets.remove(idx);
                    }
                    if line.widgets.is_empty() {
                        app.selected_widget = None;
                    } else if idx >= line.widgets.len() {
                        app.selected_widget = Some(line.widgets.len() - 1);
                    }
                }
            } else if !app.editable.lines.is_empty() {
                app.editable.lines.remove(app.selected_line);
                app.selected_line = app.selected_line.saturating_sub(1);
                sync_selected_widget(app);
            }
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Up, m) if m.contains(KeyModifiers::ALT) => {
            move_widget(app, true);
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Down, m) if m.contains(KeyModifiers::ALT) => {
            move_widget(app, false);
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Left, _) => {
            app.selected_widget = match app.selected_widget {
                Some(i) if i > 0 => Some(i - 1),
                _ => app.selected_widget,
            };
            ReducerEffect::None
        }
        (KeyCode::Right, _) => {
            if let Some(line) = app.editable.lines.get(app.selected_line) {
                let max = line.widgets.len().saturating_sub(1);
                app.selected_widget = match app.selected_widget {
                    Some(i) if i < max => Some(i + 1),
                    None if !line.widgets.is_empty() => Some(0),
                    _ => app.selected_widget,
                };
            }
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

fn move_widget(app: &mut App, up: bool) {
    let Some(idx) = app.selected_widget else { return };
    let Some(line) = app.editable.lines.get_mut(app.selected_line) else { return };
    if up && idx > 0 {
        line.widgets.swap(idx, idx - 1);
        app.selected_widget = Some(idx - 1);
    } else if !up && idx + 1 < line.widgets.len() {
        line.widgets.swap(idx, idx + 1);
        app.selected_widget = Some(idx + 1);
    }
}

fn sync_selected_widget(app: &mut App) {
    let len = app
        .editable
        .lines
        .get(app.selected_line)
        .map_or(0, |l| l.widgets.len());
    app.selected_widget = if len == 0 { None } else { Some(0) };
}

fn handle_palette(app: &mut App, key: KeyEvent) -> ReducerEffect {
    let filtered: Vec<&WidgetMeta> = filter_meta(&app.palette_filter);
    match key.code {
        KeyCode::Char('/') => {
            app.editing_field = Some(EditField::PaletteFilter);
            ReducerEffect::None
        }
        KeyCode::Up => {
            app.palette_cursor = app.palette_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            let max = filtered.len().saturating_sub(1);
            if app.palette_cursor < max {
                app.palette_cursor += 1;
            }
            ReducerEffect::None
        }
        KeyCode::Enter => {
            if let Some(meta) = filtered.get(app.palette_cursor) {
                let item = WidgetItem {
                    kind: (meta.factory)(),
                    style: WidgetStyleOverride::default(),
                };
                if app.editable.lines.is_empty() {
                    app.editable.lines.push(Line::default());
                    app.selected_line = 0;
                }
                if let Some(line) = app.editable.lines.get_mut(app.selected_line) {
                    line.widgets.push(item);
                    app.selected_widget = Some(line.widgets.len() - 1);
                }
                return ReducerEffect::RebuildPreview;
            }
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
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

fn handle_settings(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Up => {
            app.settings_field_cursor = app.settings_field_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            app.settings_field_cursor = app.settings_field_cursor.saturating_add(1);
            ReducerEffect::None
        }
        KeyCode::Char(' ') => {
            // Tri-state bold: None → Some(true) → Some(false) → None
            if let Some(item) = app.current_widget_mut() {
                item.style.bold = match item.style.bold {
                    None => Some(true),
                    Some(true) => Some(false),
                    Some(false) => None,
                };
                return ReducerEffect::RebuildPreview;
            }
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

fn handle_editing(app: &mut App, key: KeyEvent) -> ReducerEffect {
    let Some(field) = app.editing_field.as_mut() else { return ReducerEffect::None };
    match key.code {
        KeyCode::Esc => {
            app.editing_field = None;
            ReducerEffect::None
        }
        KeyCode::Char(c) => {
            match field {
                EditField::PaletteFilter => {
                    app.palette_filter.push(c);
                    app.palette_cursor = 0;
                    ReducerEffect::None
                }
                EditField::Text { buffer, cursor, .. } => {
                    buffer.insert(*cursor, c);
                    *cursor += c.len_utf8();
                    ReducerEffect::None
                }
                EditField::Number { buffer, .. } => {
                    if c.is_ascii_digit() {
                        buffer.push(c);
                    }
                    ReducerEffect::None
                }
                EditField::ColorHex { buffer, cursor, .. } => {
                    if c == '#' || c.is_ascii_hexdigit() {
                        buffer.insert(*cursor, c);
                        *cursor += 1;
                    }
                    ReducerEffect::None
                }
            }
        }
        KeyCode::Backspace => {
            match field {
                EditField::PaletteFilter => {
                    app.palette_filter.pop();
                    app.palette_cursor = 0;
                }
                EditField::Text { buffer, cursor, .. } => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        buffer.remove(*cursor);
                    }
                }
                EditField::Number { buffer, .. } => {
                    buffer.pop();
                }
                EditField::ColorHex { buffer, cursor, .. } => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        buffer.remove(*cursor);
                    }
                }
            }
            ReducerEffect::None
        }
        KeyCode::Enter => {
            commit_editing(app);
            ReducerEffect::RebuildPreview
        }
        _ => ReducerEffect::None,
    }
}

fn commit_editing(app: &mut App) {
    let Some(field) = app.editing_field.take() else { return };
    match field {
        EditField::PaletteFilter => {
            // Filter уже применился inline; Enter просто закрывает edit-mode.
        }
        EditField::Text { field, buffer, .. } => apply_text(app, field, buffer),
        EditField::Number { field, buffer } => apply_number(app, field, &buffer),
        EditField::ColorHex { field, buffer, .. } => apply_color_hex(app, field, &buffer),
    }
}

fn apply_text(app: &mut App, field: SettingsField, buffer: String) {
    use crate::types::config::WidgetConfig;
    let Some(item) = app.current_widget_mut() else { return };
    match (field, &mut item.kind) {
        (SettingsField::CustomTextText, WidgetConfig::CustomText { params }) => params.text = buffer,
        (SettingsField::CustomSymbolSymbol, WidgetConfig::CustomSymbol { params }) => params.symbol = buffer,
        (SettingsField::LinkUrl, WidgetConfig::Link { params }) => params.url = buffer,
        (SettingsField::LinkLabel, WidgetConfig::Link { params }) => params.label = if buffer.is_empty() { None } else { Some(buffer) },
        (SettingsField::CustomCommandCommand, WidgetConfig::CustomCommand { params }) => params.command = buffer,
        (SettingsField::CustomCommandArgs(idx), WidgetConfig::CustomCommand { params }) => {
            if let Some(slot) = params.args.get_mut(idx) {
                *slot = buffer;
            }
        }
        _ => {}
    }
}

fn apply_number(app: &mut App, field: SettingsField, buffer: &str) {
    use crate::types::config::WidgetConfig;
    let Ok(n) = buffer.parse::<u64>() else { return };
    let Some(item) = app.current_widget_mut() else { return };
    match (field, &mut item.kind) {
        (SettingsField::CustomCommandTimeoutMs, WidgetConfig::CustomCommand { params }) => {
            if (50..=5000).contains(&n) {
                params.timeout_ms = n;
            }
        }
        (SettingsField::ContextBarWidth, WidgetConfig::ContextBar { params }) => {
            if (1..=80).contains(&n) {
                #[allow(clippy::cast_possible_truncation)]
                {
                    params.width = n as u32;
                }
            }
        }
        _ => {}
    }
}

fn apply_color_hex(app: &mut App, field: ColorField_alias, buffer: &str) {
    let Some(item) = app.current_widget_mut() else { return };
    let v = if buffer.is_empty() { None } else { Some(buffer.to_string()) };
    match field {
        ColorField_alias::Foreground => item.style.color = v,
        ColorField_alias::Background => item.style.background_color = v,
    }
}
// Локальный alias — чтобы не тащить полный path в импорты
use crate::tui::app::ColorField as ColorField_alias;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::tui::sample;
    use crate::types::config::{Settings, WidgetConfig};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn key_mod(code: KeyCode, m: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, m)
    }

    fn make_app() -> App {
        let (p, f) = sample::payload();
        // Один widget для удобства тестов.
        let json = r#"{"lines":[{"widgets":[{"type":"model"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        App::new(s, p, f)
    }

    fn empty_app() -> App {
        let (p, f) = sample::payload();
        App::new(Settings::default(), p, f)
    }

    #[test]
    fn tab_cycles_focus_forward() {
        let mut app = make_app();
        assert_eq!(app.focus, Pane::Lines);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Palette);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Settings);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Preview);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Lines);
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::BackTab));
        assert_eq!(app.focus, Pane::Preview);
    }

    #[test]
    fn arrow_in_lines_pane_changes_selected_line() {
        let mut app = make_app();
        let json = r#"{"lines":[{"widgets":[{"type":"model"}]},{"widgets":[]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_line, 1);
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.selected_line, 0);
    }

    #[test]
    fn down_in_palette_pane_advances_cursor() {
        let mut app = make_app();
        app.focus = Pane::Palette;
        let before = app.palette_cursor;
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.palette_cursor, before + 1);
    }

    #[test]
    fn enter_in_palette_adds_widget_to_selected_line() {
        let mut app = make_app();
        app.focus = Pane::Palette;
        let before = app.editable.lines[0].widgets.len();
        handle_key(&mut app, key(KeyCode::Enter));
        let after = app.editable.lines[0].widgets.len();
        assert_eq!(after, before + 1);
    }

    #[test]
    fn delete_in_lines_removes_selected_widget() {
        let mut app = make_app();
        assert_eq!(app.editable.lines[0].widgets.len(), 1);
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.editable.lines[0].widgets.len(), 0);
    }

    #[test]
    fn alt_up_swaps_widgets() {
        let mut app = make_app();
        let json = r#"{"lines":[{"widgets":[{"type":"model"},{"type":"version"}]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        app.selected_widget = Some(1);
        handle_key(&mut app, key_mod(KeyCode::Up, KeyModifiers::ALT));
        assert!(matches!(app.editable.lines[0].widgets[0].kind, WidgetConfig::Version));
        assert!(matches!(app.editable.lines[0].widgets[1].kind, WidgetConfig::Model { .. }));
        assert_eq!(app.selected_widget, Some(0));
    }

    #[test]
    fn slash_in_palette_enters_filter_edit_mode() {
        let mut app = make_app();
        app.focus = Pane::Palette;
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert!(matches!(app.editing_field, Some(EditField::PaletteFilter)));
    }

    #[test]
    fn typing_in_palette_filter_updates_buffer_and_resets_cursor() {
        let mut app = make_app();
        app.focus = Pane::Palette;
        app.editing_field = Some(EditField::PaletteFilter);
        app.palette_cursor = 5;
        handle_key(&mut app, key(KeyCode::Char('g')));
        handle_key(&mut app, key(KeyCode::Char('i')));
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.palette_filter, "git");
        assert_eq!(app.palette_cursor, 0);
    }

    #[test]
    fn space_toggles_tri_state_bold_in_settings() {
        let mut app = make_app();
        app.focus = Pane::Settings;
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, None);
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, Some(true));
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, Some(false));
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, None);
    }

    #[test]
    fn t_opens_themes_overlay_and_t_again_closes() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.mode, Mode::ThemesOverlay);
        handle_key(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.mode, Mode::Edit);
    }

    #[test]
    fn question_mark_opens_help_any_key_closes() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Char('?')));
        assert_eq!(app.mode, Mode::HelpOverlay);
        handle_key(&mut app, key(KeyCode::Char('x')));
        assert_eq!(app.mode, Mode::Edit);
    }

    #[test]
    fn q_with_no_dirty_returns_quit() {
        let mut app = empty_app();
        let eff = handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(eff, ReducerEffect::Quit);
    }

    #[test]
    fn q_with_dirty_opens_confirm_modal() {
        let mut app = empty_app();
        app.editable.lines.push(Line::default()); // make dirty
        let eff = handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(eff, ReducerEffect::None);
        assert_eq!(app.mode, Mode::ConfirmQuit);
    }

    #[test]
    fn confirm_quit_s_returns_request_save() {
        let mut app = make_app();
        app.mode = Mode::ConfirmQuit;
        let eff = handle_key(&mut app, key(KeyCode::Char('s')));
        assert_eq!(eff, ReducerEffect::RequestSaveAndQuit);
    }

    #[test]
    fn confirm_quit_d_returns_request_discard() {
        let mut app = make_app();
        app.mode = Mode::ConfirmQuit;
        let eff = handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(eff, ReducerEffect::RequestDiscardAndQuit);
    }

    #[test]
    fn confirm_quit_c_or_esc_cancels_to_edit() {
        let mut app = make_app();
        app.mode = Mode::ConfirmQuit;
        handle_key(&mut app, key(KeyCode::Char('c')));
        assert_eq!(app.mode, Mode::Edit);
        app.mode = Mode::ConfirmQuit;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Edit);
    }

    #[test]
    fn esc_in_editing_field_exits_edit_without_apply() {
        let mut app = make_app();
        app.editing_field = Some(EditField::Number {
            field: SettingsField::ContextBarWidth,
            buffer: "999".into(), // out of range
        });
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(app.editing_field.is_none());
    }

    #[test]
    fn ctrl_s_returns_request_save_directly() {
        let mut app = make_app();
        let eff = handle_key(&mut app, key_mod(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(eff, ReducerEffect::RequestSaveAndQuit);
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --features tui --locked tui::reducer
```

Expected: ≥18 PASS (15+ unique transitions covered).

- [ ] **Step 4: Verify --no-default-features build**

```bash
cargo build --locked --no-default-features
```

Expected: PASS.

- [ ] **Step 5: Lints**

```bash
cargo clippy --locked --features tui --all-targets -- -D warnings
```

Expected: PASS. `clippy::too_many_lines` может ругаться — позволь через `#[allow(clippy::too_many_lines)]` на больших функциях `handle_key`/`handle_editing`/`apply_text`.

- [ ] **Step 6: Commit**

```bash
git add src/tui/effects.rs src/tui/reducer.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T7 reducer — pure handle_key + ReducerEffect + 18 unit-tests

- ReducerEffect { None, Quit, RequestSaveAndQuit, RequestDiscardAndQuit, RebuildPreview }
- handle_key dispatches by mode (Edit/ThemesOverlay/HelpOverlay/ConfirmQuit) and pane
- Tab/Shift+Tab cycles focus across 4 panes
- Lines pane: ↑↓ select, a add, d delete, Alt+↑↓ reorder, ←→ widget cursor
- Palette pane: / filter, ↑↓ navigate, Enter add to current line
- Settings pane: ↑↓ field, Space tri-state bold; commit_editing applies Text/Number/ColorHex
- ConfirmQuit modal: s save, d discard, c/Esc cancel
- t toggles ThemesOverlay; ? opens HelpOverlay (any key closes)
- q without dirty → Quit; with dirty → ConfirmQuit
- Ctrl+S direct save shortcut

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
