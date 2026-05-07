//! Pure reducer.
//!
//! `handle_key(&mut App, KeyEvent) -> ReducerEffect` — единственная mutating-точка
//! для App'а. Никакого file/terminal IO. Save/Quit/Discard поднимаются как
//! `ReducerEffect` и обрабатываются в event loop.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{
    App, ColorField, EditField, MessageKind, Mode, Pane, PaletteMode, Screen, SettingsField,
};
use crate::tui::effects::ReducerEffect;
use crate::tui::widget_meta::{ALL_KINDS, WidgetMeta};
use crate::types::config::{Line, WidgetItem, WidgetStyleOverride};

/// Pure key dispatcher. Возвращает effect для event loop.
pub fn handle_key(app: &mut App, key: KeyEvent) -> ReducerEffect {
    // Modals — ConfirmQuit/ConfirmReturnHome/PresetNamePrompt — обрабатываются
    // независимо от screen. Overlays (Help/Themes) — только в EditLines.
    match app.mode {
        Mode::ConfirmQuit => return handle_confirm_quit(app, key),
        Mode::ConfirmReturnHome => return handle_confirm_return_home(app, key),
        Mode::PresetNamePrompt => return handle_preset_name_prompt(app, key),
        Mode::HelpOverlay if matches!(app.screen, Screen::EditLines) => {
            return handle_help(app, key);
        }
        Mode::ThemesOverlay if matches!(app.screen, Screen::EditLines) => {
            return handle_themes(app, key);
        }
        _ => {}
    }

    match app.screen {
        Screen::Home => handle_home(app, key),
        Screen::EditLines => handle_edit_lines(app, key),
        Screen::ChoosePreset => handle_choose_preset(app, key),
    }
}

fn handle_home(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.home_cursor = app.home_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.home_cursor = (app.home_cursor + 1).min(3);
            ReducerEffect::None
        }
        KeyCode::Enter => match app.home_cursor {
            0 => {
                app.screen = Screen::EditLines;
                app.mode = Mode::Edit;
                ReducerEffect::None
            }
            1 => {
                if app.presets.is_empty() {
                    app.presets = crate::tui::presets::list_all();
                }
                app.preset_cursor = 0;
                app.screen = Screen::ChoosePreset;
                ReducerEffect::None
            }
            2 => ReducerEffect::RunInstall,
            _ => quit_or_confirm(app),
        },
        KeyCode::Char('q') | KeyCode::Esc => quit_or_confirm(app),
        _ => ReducerEffect::None,
    }
}

fn handle_choose_preset(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.preset_cursor = app.preset_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.preset_cursor + 1 < app.presets.len() {
                app.preset_cursor += 1;
            }
            ReducerEffect::None
        }
        KeyCode::Enter => {
            if let Some(preset) = app.presets.get(app.preset_cursor).cloned() {
                crate::tui::presets::apply(app, &preset);
                app.status_message = Some((
                    format!("Applied: {}", preset.name),
                    MessageKind::Info,
                ));
            }
            app.screen = Screen::Home;
            ReducerEffect::None
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::Home;
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

#[allow(clippy::missing_const_for_fn)]
fn handle_confirm_return_home(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Char('s' | 'S') => ReducerEffect::RequestSaveAndReturnHome,
        KeyCode::Char('d' | 'D') => ReducerEffect::RequestDiscardAndReturnHome,
        KeyCode::Char('c' | 'C') | KeyCode::Esc => {
            app.mode = Mode::Edit;
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

#[allow(clippy::missing_const_for_fn)]
fn handle_preset_name_prompt(_app: &mut App, _key: KeyEvent) -> ReducerEffect {
    // Wired in T7.
    ReducerEffect::None
}

#[allow(clippy::too_many_lines)]
fn handle_edit_lines(app: &mut App, key: KeyEvent) -> ReducerEffect {
    // Edit-mode внутри poll-строки (PaletteFilter / Text / Number / ColorHex).
    if app.editing_field.is_some() {
        return handle_editing(app, key);
    }

    // Глобальные key-биндинги в Edit (focus-independent).
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), m) if !m.contains(KeyModifiers::SHIFT) => return quit_or_confirm(app),
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
            return quit_or_confirm(app);
        }
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
    // Preview is read-only — exclude from focus cycle.
    let mut order: Vec<Pane> = vec![Pane::Lines, Pane::Settings];
    if app.palette_visible {
        order.insert(1, Pane::Palette);
    }
    // If we somehow are on Preview, treat as Lines.
    let idx = order
        .iter()
        .position(|p| *p == app.focus)
        .unwrap_or(0);
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

#[allow(clippy::missing_const_for_fn)]
fn handle_help(app: &mut App, _key: KeyEvent) -> ReducerEffect {
    // Любая клавиша закрывает help.
    app.mode = Mode::Edit;
    ReducerEffect::None
}

#[allow(clippy::missing_const_for_fn)]
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

#[allow(clippy::missing_const_for_fn)]
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

#[allow(clippy::too_many_lines)]
fn handle_lines(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match (key.code, key.modifiers) {
        // Alt+↑↓ reorder — must match before plain ↑↓.
        (KeyCode::Up, m) if m.contains(KeyModifiers::ALT) => {
            move_widget(app, true);
            ReducerEffect::RebuildPreview
        }
        (KeyCode::Down, m) if m.contains(KeyModifiers::ALT) => {
            move_widget(app, false);
            ReducerEffect::RebuildPreview
        }
        // ↑/↓ — flat-walk виджетов через линии (соответствует визуальной раскладке).
        (KeyCode::Up, m) if m.is_empty() => {
            nav_widget_prev(app);
            ReducerEffect::None
        }
        (KeyCode::Down, m) if m.is_empty() => {
            nav_widget_next(app);
            ReducerEffect::None
        }
        // ←/→ — переход между линиями (preserving widget index).
        (KeyCode::Left, _) => {
            if app.selected_line > 0 {
                app.selected_line -= 1;
                sync_selected_widget(app);
            }
            ReducerEffect::None
        }
        (KeyCode::Right, _) => {
            if app.selected_line + 1 < app.editable.lines.len() {
                app.selected_line += 1;
                sync_selected_widget(app);
            }
            ReducerEffect::None
        }
        // Enter → открывает палитру для замены типа выбранного виджета.
        (KeyCode::Enter, _) => {
            if app.selected_widget.is_some() {
                open_palette(app, PaletteMode::Replace);
            }
            ReducerEffect::None
        }
        (KeyCode::Char('a'), _) => {
            // Открываем палитру для добавления нового виджета в текущую линию.
            if app.editable.lines.is_empty() {
                app.editable.lines.push(Line::default());
                app.selected_line = 0;
            }
            open_palette(app, PaletteMode::Add);
            ReducerEffect::None
        }
        (KeyCode::Char('l'), _) => {
            app.editable.lines.push(Line::default());
            app.selected_line = app.editable.lines.len().saturating_sub(1);
            app.selected_widget = None;
            ReducerEffect::RebuildPreview
        }
        // 'r' — toggle raw value на выбранном виджете прямо из Lines (если поддерживается).
        (KeyCode::Char('r'), m) if m.is_empty() => {
            if app
                .current_widget()
                .is_some_and(|i| crate::widgets::widget_supports_raw_value(&i.kind))
            {
                return toggle_raw_value(app);
            }
            ReducerEffect::None
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
                app.selected_line = app
                    .selected_line
                    .min(app.editable.lines.len().saturating_sub(1));
                sync_selected_widget(app);
            }
            ReducerEffect::RebuildPreview
        }
        // Esc на Lines pane → возврат на Home (или модал, если есть unsaved).
        (KeyCode::Esc, _) => {
            if app.dirty() {
                app.mode = Mode::ConfirmReturnHome;
            } else {
                app.screen = Screen::Home;
            }
            ReducerEffect::None
        }
        _ => ReducerEffect::None,
    }
}

fn nav_widget_prev(app: &mut App) {
    if let Some(w) = app.selected_widget {
        if w > 0 {
            app.selected_widget = Some(w - 1);
            return;
        }
    }
    // На первом виджете или на пустой линии → шагаем на предыдущую линию.
    if app.selected_line > 0 {
        app.selected_line -= 1;
        let len = app
            .editable
            .lines
            .get(app.selected_line)
            .map_or(0, |l| l.widgets.len());
        app.selected_widget = if len == 0 { None } else { Some(len - 1) };
    }
}

fn nav_widget_next(app: &mut App) {
    let line_len = app
        .editable
        .lines
        .get(app.selected_line)
        .map_or(0, |l| l.widgets.len());
    if let Some(w) = app.selected_widget {
        if w + 1 < line_len {
            app.selected_widget = Some(w + 1);
            return;
        }
    }
    // На последнем виджете или на пустой линии → следующая линия.
    if app.selected_line + 1 < app.editable.lines.len() {
        app.selected_line += 1;
        let len = app
            .editable
            .lines
            .get(app.selected_line)
            .map_or(0, |l| l.widgets.len());
        app.selected_widget = if len == 0 { None } else { Some(0) };
    }
}

fn move_widget(app: &mut App, up: bool) {
    let Some(idx) = app.selected_widget else {
        return;
    };
    let Some(line) = app.editable.lines.get_mut(app.selected_line) else {
        return;
    };
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
    app.selected_widget = if len == 0 {
        None
    } else {
        Some(app.selected_widget.unwrap_or(0).min(len - 1))
    };
}

fn open_palette(app: &mut App, mode: PaletteMode) {
    app.palette_visible = true;
    app.palette_mode = mode;
    app.palette_filter.clear();
    app.palette_cursor = 0;
    app.focus = Pane::Palette;
    app.editing_field = Some(EditField::PaletteFilter);
}

fn close_palette(app: &mut App) {
    app.palette_visible = false;
    app.editing_field = None;
    app.focus = Pane::Lines;
}

fn handle_palette(app: &mut App, key: KeyEvent) -> ReducerEffect {
    // Палитра — это собственный модальный режим. Любое нажатие сюда не доходит:
    // когда она открыта, `app.editing_field == PaletteFilter`, и события идут в
    // `handle_editing` → `handle_palette_filter_key`. Эта функция оставлена как
    // запасной обработчик, если палитра вдруг сфокусирована без edit-режима.
    match key.code {
        KeyCode::Esc => {
            close_palette(app);
            ReducerEffect::None
        }
        KeyCode::Up | KeyCode::Down | KeyCode::Enter | KeyCode::Backspace | KeyCode::Char(_) => {
            // Восстанавливаем фильтр-режим и пропускаем событие через него.
            app.editing_field = Some(EditField::PaletteFilter);
            handle_palette_filter_key(app, key)
        }
        _ => ReducerEffect::None,
    }
}

fn handle_palette_filter_key(app: &mut App, key: KeyEvent) -> ReducerEffect {
    match key.code {
        KeyCode::Esc => {
            close_palette(app);
            ReducerEffect::None
        }
        KeyCode::Up => {
            app.palette_cursor = app.palette_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            let max = filter_meta(&app.palette_filter).len().saturating_sub(1);
            if app.palette_cursor < max {
                app.palette_cursor += 1;
            }
            ReducerEffect::None
        }
        KeyCode::Backspace => {
            app.palette_filter.pop();
            let max = filter_meta(&app.palette_filter).len().saturating_sub(1);
            app.palette_cursor = app.palette_cursor.min(max);
            ReducerEffect::None
        }
        KeyCode::Char(c) => {
            app.palette_filter.push(c);
            let max = filter_meta(&app.palette_filter).len().saturating_sub(1);
            app.palette_cursor = app.palette_cursor.min(max);
            ReducerEffect::None
        }
        KeyCode::Enter => {
            apply_palette_pick(app);
            ReducerEffect::RebuildPreview
        }
        _ => ReducerEffect::None,
    }
}

fn apply_palette_pick(app: &mut App) {
    let filtered: Vec<&WidgetMeta> = filter_meta(&app.palette_filter);
    let Some(meta) = filtered.get(app.palette_cursor).copied() else {
        close_palette(app);
        return;
    };
    match app.palette_mode {
        PaletteMode::Add => {
            let item = WidgetItem {
                kind: (meta.factory)(),
                style: WidgetStyleOverride::default(),
                raw_value: false,
            };
            if app.editable.lines.is_empty() {
                app.editable.lines.push(Line::default());
                app.selected_line = 0;
            }
            if let Some(line) = app.editable.lines.get_mut(app.selected_line) {
                line.widgets.push(item);
                app.selected_widget = Some(line.widgets.len() - 1);
            }
        }
        PaletteMode::Replace => {
            if let Some(idx) = app.selected_widget {
                if let Some(line) = app.editable.lines.get_mut(app.selected_line) {
                    if let Some(slot) = line.widgets.get_mut(idx) {
                        slot.kind = (meta.factory)();
                    }
                }
            }
        }
    }
    close_palette(app);
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
    let max = current_widget_max_field(app);
    match key.code {
        KeyCode::Up => {
            app.settings_field_cursor = app.settings_field_cursor.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            if app.settings_field_cursor < max {
                app.settings_field_cursor += 1;
            }
            ReducerEffect::None
        }
        KeyCode::Esc => {
            // Esc → возврат фокуса в Lines.
            app.focus = Pane::Lines;
            ReducerEffect::None
        }
        KeyCode::Char(' ') => {
            use crate::types::config::WidgetConfig;
            // Tri-state bold: только когда курсор на bold-row.
            if app.settings_field_cursor == 2 {
                if let Some(item) = app.current_widget_mut() {
                    item.style.bold = match item.style.bold {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => None,
                    };
                    return ReducerEffect::RebuildPreview;
                }
            }
            // Raw value toggle (only on widgets that support it).
            if app.settings_field_cursor == 3
                && app
                    .current_widget()
                    .is_some_and(|i| crate::widgets::widget_supports_raw_value(&i.kind))
            {
                return toggle_raw_value(app);
            }
            // CurrentWorkingDir bool toggles.
            if matches!(
                app.current_widget().map(|i| &i.kind),
                Some(WidgetConfig::CurrentWorkingDir { .. })
            ) {
                match app.settings_field_cursor {
                    4 => return toggle_cwd_abbreviate_home(app),
                    5 => return toggle_cwd_fish_style(app),
                    _ => {}
                }
            }
            ReducerEffect::None
        }
        KeyCode::Enter => enter_field_edit(app),
        _ => ReducerEffect::None,
    }
}

#[must_use]
fn current_widget_max_field(app: &App) -> usize {
    use crate::types::config::WidgetConfig;
    let Some(item) = app.current_widget() else {
        return 2;
    };
    match &item.kind {
        WidgetConfig::CustomText { .. }
        | WidgetConfig::CustomSymbol { .. }
        | WidgetConfig::ContextBar { .. } => 3,
        WidgetConfig::Link { .. } => 4,
        // 0=FG 1=BG 2=Bold 3=Command 4=Timeout 5..=Args
        WidgetConfig::CustomCommand { params } => 4 + params.args.len(),
        // 3=Segments, 4=AbbreviateHome, 5=FishStyle, 6=Prefix
        WidgetConfig::CurrentWorkingDir { .. } => 6,
        // raw-supporting widgets get a single extra row (index 3 = Raw).
        kind if crate::widgets::widget_supports_raw_value(kind) => 3,
        _ => 2,
    }
}

#[allow(clippy::too_many_lines)]
fn enter_field_edit(app: &mut App) -> ReducerEffect {
    use crate::types::config::WidgetConfig;
    let cursor = app.settings_field_cursor;
    match cursor {
        0 => {
            sync_color_cursor(app, ColorField::Foreground);
            app.editing_field = Some(EditField::ColorPicker {
                field: ColorField::Foreground,
            });
            ReducerEffect::None
        }
        1 => {
            sync_color_cursor(app, ColorField::Background);
            app.editing_field = Some(EditField::ColorPicker {
                field: ColorField::Background,
            });
            ReducerEffect::None
        }
        2 => {
            // Enter на bold-row тоже как Space — tri-state toggle.
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
        n => {
            let Some(item) = app.current_widget() else {
                return ReducerEffect::None;
            };
            // Enter on Raw row toggles raw_value (mirrors Space).
            if n == 3 && crate::widgets::widget_supports_raw_value(&item.kind) {
                return toggle_raw_value(app);
            }
            match (&item.kind, n) {
                (WidgetConfig::CustomText { params }, 3) => {
                    start_text_edit(app, SettingsField::CustomTextText, params.text.clone());
                }
                (WidgetConfig::CustomSymbol { params }, 3) => {
                    start_text_edit(
                        app,
                        SettingsField::CustomSymbolSymbol,
                        params.symbol.clone(),
                    );
                }
                (WidgetConfig::Link { params }, 3) => {
                    start_text_edit(app, SettingsField::LinkUrl, params.url.clone());
                }
                (WidgetConfig::Link { params }, 4) => {
                    start_text_edit(
                        app,
                        SettingsField::LinkLabel,
                        params.label.clone().unwrap_or_default(),
                    );
                }
                (WidgetConfig::CustomCommand { params }, 3) => {
                    start_text_edit(
                        app,
                        SettingsField::CustomCommandCommand,
                        params.command.clone(),
                    );
                }
                (WidgetConfig::CustomCommand { params }, 4) => {
                    app.editing_field = Some(EditField::Number {
                        field: SettingsField::CustomCommandTimeoutMs,
                        buffer: params.timeout_ms.to_string(),
                    });
                }
                (WidgetConfig::CustomCommand { params }, k)
                    if k >= 5 && k - 5 < params.args.len() =>
                {
                    let idx = k - 5;
                    let buf = params.args[idx].clone();
                    start_text_edit(app, SettingsField::CustomCommandArgs(idx), buf);
                }
                (WidgetConfig::ContextBar { params }, 3) => {
                    app.editing_field = Some(EditField::Number {
                        field: SettingsField::ContextBarWidth,
                        buffer: params.width.to_string(),
                    });
                }
                (WidgetConfig::CurrentWorkingDir { params }, 3) => {
                    app.editing_field = Some(EditField::Number {
                        field: SettingsField::CurrentWorkingDirSegments,
                        buffer: params.segments.map(|n| n.to_string()).unwrap_or_default(),
                    });
                }
                (WidgetConfig::CurrentWorkingDir { .. }, 4) => {
                    return toggle_cwd_abbreviate_home(app);
                }
                (WidgetConfig::CurrentWorkingDir { .. }, 5) => {
                    return toggle_cwd_fish_style(app);
                }
                (WidgetConfig::CurrentWorkingDir { params }, 6) => {
                    start_text_edit(
                        app,
                        SettingsField::CurrentWorkingDirPrefix,
                        params.prefix.clone().unwrap_or_default(),
                    );
                }
                _ => {}
            }
            ReducerEffect::None
        }
    }
}

fn toggle_raw_value(app: &mut App) -> ReducerEffect {
    let Some(item) = app.current_widget_mut() else {
        return ReducerEffect::None;
    };
    item.raw_value = !item.raw_value;
    ReducerEffect::RebuildPreview
}

fn toggle_cwd_abbreviate_home(app: &mut App) -> ReducerEffect {
    use crate::types::config::WidgetConfig;
    let Some(item) = app.current_widget_mut() else {
        return ReducerEffect::None;
    };
    if let WidgetConfig::CurrentWorkingDir { params } = &mut item.kind {
        params.abbreviate_home = !params.abbreviate_home;
        if params.abbreviate_home {
            params.fish_style = false;
        }
    }
    ReducerEffect::RebuildPreview
}

fn toggle_cwd_fish_style(app: &mut App) -> ReducerEffect {
    use crate::types::config::WidgetConfig;
    let Some(item) = app.current_widget_mut() else {
        return ReducerEffect::None;
    };
    if let WidgetConfig::CurrentWorkingDir { params } = &mut item.kind {
        params.fish_style = !params.fish_style;
        if params.fish_style {
            params.abbreviate_home = false;
            params.segments = None;
        }
    }
    ReducerEffect::RebuildPreview
}

fn start_text_edit(app: &mut App, field: SettingsField, buffer: String) {
    let cursor = buffer.len();
    app.editing_field = Some(EditField::Text {
        field,
        buffer,
        cursor,
    });
}

fn sync_color_cursor(app: &mut App, field: ColorField) {
    use crate::tui::widgets_ui::color_picker::NAMED_COLORS;
    let Some(item) = app.current_widget() else {
        return;
    };
    let current = match field {
        ColorField::Foreground => item.style.color.as_deref(),
        ColorField::Background => item.style.background_color.as_deref(),
    };
    let idx = current.map_or(0, |hex| {
        NAMED_COLORS
            .iter()
            .position(|(_, h)| *h == hex)
            .unwrap_or(NAMED_COLORS.len() - 1)
    });
    match field {
        ColorField::Foreground => app.color_fg_cursor = idx,
        ColorField::Background => app.color_bg_cursor = idx,
    }
}

#[allow(clippy::too_many_lines)]
fn handle_editing(app: &mut App, key: KeyEvent) -> ReducerEffect {
    // Палитра-фильтр — отдельный модальный обработчик (Up/Down/Enter/Esc/типизация).
    if matches!(app.editing_field, Some(EditField::PaletteFilter)) {
        return handle_palette_filter_key(app, key);
    }
    // Color picker submode не имеет buffer — отдельная ветка с навигацией.
    if let Some(EditField::ColorPicker { field }) = app.editing_field {
        return handle_color_picker(app, key, field);
    }
    let Some(field) = app.editing_field.as_mut() else {
        return ReducerEffect::None;
    };
    match key.code {
        KeyCode::Esc => {
            app.editing_field = None;
            ReducerEffect::None
        }
        KeyCode::Char(c) => match field {
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
            // PaletteFilter / ColorPicker отфильтрованы ранними return'ами выше.
            EditField::PaletteFilter | EditField::ColorPicker { .. } => ReducerEffect::None,
        },
        KeyCode::Backspace => {
            match field {
                EditField::Text { buffer, cursor, .. }
                | EditField::ColorHex { buffer, cursor, .. } => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        buffer.remove(*cursor);
                    }
                }
                EditField::Number { buffer, .. } => {
                    buffer.pop();
                }
                EditField::PaletteFilter | EditField::ColorPicker { .. } => {}
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

fn handle_color_picker(app: &mut App, key: KeyEvent, field: ColorField) -> ReducerEffect {
    use crate::tui::widgets_ui::color_picker::NAMED_COLORS;
    let last = NAMED_COLORS.len() - 1;
    match key.code {
        KeyCode::Esc => {
            app.editing_field = None;
            ReducerEffect::None
        }
        KeyCode::Up => {
            let cur = match field {
                ColorField::Foreground => &mut app.color_fg_cursor,
                ColorField::Background => &mut app.color_bg_cursor,
            };
            *cur = cur.saturating_sub(1);
            ReducerEffect::None
        }
        KeyCode::Down => {
            let cur = match field {
                ColorField::Foreground => &mut app.color_fg_cursor,
                ColorField::Background => &mut app.color_bg_cursor,
            };
            *cur = (*cur + 1).min(last);
            ReducerEffect::None
        }
        KeyCode::Enter => {
            let idx = match field {
                ColorField::Foreground => app.color_fg_cursor,
                ColorField::Background => app.color_bg_cursor,
            };
            if idx == last {
                // "Custom hex…" → переход в hex-input.
                app.editing_field = Some(EditField::ColorHex {
                    field,
                    buffer: String::new(),
                    cursor: 0,
                });
                return ReducerEffect::None;
            }
            let value = if idx == 0 {
                None
            } else {
                Some(NAMED_COLORS[idx].1.to_string())
            };
            if let Some(item) = app.current_widget_mut() {
                match field {
                    ColorField::Foreground => item.style.color = value,
                    ColorField::Background => item.style.background_color = value,
                }
            }
            app.editing_field = None;
            ReducerEffect::RebuildPreview
        }
        _ => ReducerEffect::None,
    }
}

fn commit_editing(app: &mut App) {
    let Some(field) = app.editing_field.take() else {
        return;
    };
    match field {
        EditField::PaletteFilter | EditField::ColorPicker { .. } => {
            // ColorPicker и filter не используют commit_editing — обрабатываются inline.
        }
        EditField::Text { field, buffer, .. } => apply_text(app, field, buffer),
        EditField::Number { field, buffer } => apply_number(app, field, &buffer),
        EditField::ColorHex { field, buffer, .. } => apply_color_hex(app, field, &buffer),
    }
}

fn apply_text(app: &mut App, field: SettingsField, buffer: String) {
    use crate::types::config::WidgetConfig;
    let Some(item) = app.current_widget_mut() else {
        return;
    };
    match (field, &mut item.kind) {
        (SettingsField::CustomTextText, WidgetConfig::CustomText { params }) => {
            params.text = buffer;
        }
        (SettingsField::CustomSymbolSymbol, WidgetConfig::CustomSymbol { params }) => {
            params.symbol = buffer;
        }
        (SettingsField::LinkUrl, WidgetConfig::Link { params }) => {
            params.url = buffer;
        }
        (SettingsField::LinkLabel, WidgetConfig::Link { params }) => {
            params.label = if buffer.is_empty() {
                None
            } else {
                Some(buffer)
            };
        }
        (SettingsField::CustomCommandCommand, WidgetConfig::CustomCommand { params }) => {
            params.command = buffer;
        }
        (SettingsField::CustomCommandArgs(idx), WidgetConfig::CustomCommand { params }) => {
            if let Some(slot) = params.args.get_mut(idx) {
                *slot = buffer;
            }
        }
        (SettingsField::CurrentWorkingDirPrefix, WidgetConfig::CurrentWorkingDir { params }) => {
            params.prefix = if buffer.is_empty() { None } else { Some(buffer) };
        }
        _ => {}
    }
}

fn apply_number(app: &mut App, field: SettingsField, buffer: &str) {
    use crate::types::config::WidgetConfig;
    // Empty buffer for CWD segments → clear (None).
    if buffer.is_empty() && matches!(field, SettingsField::CurrentWorkingDirSegments) {
        if let Some(item) = app.current_widget_mut() {
            if let WidgetConfig::CurrentWorkingDir { params } = &mut item.kind {
                params.segments = None;
            }
        }
        return;
    }
    let Ok(n) = buffer.parse::<u64>() else { return };
    let Some(item) = app.current_widget_mut() else {
        return;
    };
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
        (
            SettingsField::CurrentWorkingDirSegments,
            WidgetConfig::CurrentWorkingDir { params },
        ) => {
            if n == 0 {
                params.segments = None;
            } else if (1..=10).contains(&n) {
                #[allow(clippy::cast_possible_truncation)]
                {
                    params.segments = Some(n as u32);
                }
                params.fish_style = false;
            }
        }
        _ => {}
    }
}

fn apply_color_hex(app: &mut App, field: ColorField, buffer: &str) {
    let Some(item) = app.current_widget_mut() else {
        return;
    };
    let v = if buffer.is_empty() {
        None
    } else {
        Some(buffer.to_string())
    };
    match field {
        ColorField::Foreground => item.style.color = v,
        ColorField::Background => item.style.background_color = v,
    }
}

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
        let mut app = App::new(s, p, f);
        // Legacy reducer tests assume EditLines screen; new screens are tested explicitly.
        app.screen = Screen::EditLines;
        app
    }

    fn home_app() -> App {
        let (p, f) = sample::payload();
        App::new(Settings::default(), p, f)
    }

    fn empty_app() -> App {
        let (p, f) = sample::payload();
        let mut app = App::new(Settings::default(), p, f);
        app.screen = Screen::EditLines;
        app
    }

    #[test]
    fn tab_cycles_focus_forward_skipping_preview() {
        let mut app = make_app();
        // Палитра скрыта — Tab пропускает её. Preview исключён из цикла.
        assert_eq!(app.focus, Pane::Lines);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Settings);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Lines);
    }

    #[test]
    fn tab_includes_palette_when_visible() {
        let mut app = make_app();
        app.palette_visible = true;
        assert_eq!(app.focus, Pane::Lines);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Palette);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Settings);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Pane::Lines);
    }

    #[test]
    fn shift_tab_cycles_focus_backward_skipping_preview() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::BackTab));
        // Backwards from Lines goes to Settings (Preview excluded).
        assert_eq!(app.focus, Pane::Settings);
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
        assert!(matches!(
            app.editable.lines[0].widgets[0].kind,
            WidgetConfig::Version
        ));
        assert!(matches!(
            app.editable.lines[0].widgets[1].kind,
            WidgetConfig::Model { .. }
        ));
        assert_eq!(app.selected_widget, Some(0));
    }

    #[test]
    fn opening_palette_starts_in_filter_edit_mode() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert!(matches!(app.editing_field, Some(EditField::PaletteFilter)));
    }

    #[test]
    fn typing_in_palette_filter_clamps_cursor_to_results() {
        let mut app = make_app();
        app.focus = Pane::Palette;
        app.editing_field = Some(EditField::PaletteFilter);
        app.palette_cursor = 5;
        // "model" yields exactly 1 result → cursor clamped from 5 to 0
        handle_key(&mut app, key(KeyCode::Char('m')));
        handle_key(&mut app, key(KeyCode::Char('o')));
        handle_key(&mut app, key(KeyCode::Char('d')));
        handle_key(&mut app, key(KeyCode::Char('e')));
        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.palette_filter, "model");
        assert_eq!(app.palette_cursor, 0);
    }

    #[test]
    fn space_toggles_tri_state_bold_in_settings() {
        let mut app = make_app();
        app.focus = Pane::Settings;
        // Bold-row = settings_field_cursor 2 (0=FG, 1=BG, 2=Bold).
        app.settings_field_cursor = 2;
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, None);
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, Some(true));
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, Some(false));
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.editable.lines[0].widgets[0].style.bold, None);
    }

    #[test]
    fn space_toggles_raw_value_for_supported_widget() {
        let (p, f) = sample::payload();
        let json = r#"{"lines":[{"widgets":[{"type":"context-length"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let mut app = App::new(s, p, f);
        app.screen = Screen::EditLines;
        app.focus = Pane::Settings;
        // Raw row = settings_field_cursor 3 (after FG/BG/Bold).
        app.settings_field_cursor = 3;
        assert!(!app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(!app.editable.lines[0].widgets[0].raw_value);
    }

    #[test]
    fn r_in_lines_toggles_raw_value_for_supported_widget() {
        let (p, f) = sample::payload();
        let json = r#"{"lines":[{"widgets":[{"type":"thinking-effort"}]}]}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        let mut app = App::new(s, p, f);
        app.screen = Screen::EditLines;
        assert_eq!(app.focus, Pane::Lines);
        app.selected_widget = Some(0);
        assert!(!app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert!(app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert!(!app.editable.lines[0].widgets[0].raw_value);
    }

    #[test]
    fn r_in_lines_noop_for_widget_without_raw_support() {
        let mut app = make_app();
        assert_eq!(app.focus, Pane::Lines);
        app.selected_widget = Some(0);
        assert!(!app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char('r')));
        assert!(!app.editable.lines[0].widgets[0].raw_value);
    }

    #[test]
    fn space_does_nothing_on_raw_row_for_widget_without_raw_support() {
        // Model has no inherent prefix → max field is 2; cursor 3 is out of range.
        let mut app = make_app();
        app.focus = Pane::Settings;
        app.settings_field_cursor = 3;
        assert!(!app.editable.lines[0].widgets[0].raw_value);
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(!app.editable.lines[0].widgets[0].raw_value);
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

    #[test]
    fn delete_only_line_leaves_valid_state() {
        let mut app = make_app();
        // Only one line, selected_widget = None (widget deleted first)
        app.selected_widget = None;
        assert_eq!(app.editable.lines.len(), 1);
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.editable.lines.len(), 0);
        assert_eq!(app.selected_line, 0);
        assert_eq!(app.selected_widget, None);
    }

    #[test]
    fn delete_middle_line_keeps_cursor_at_same_index() {
        let mut app = make_app();
        let json = r#"{"lines":[{"widgets":[]},{"widgets":[]},{"widgets":[]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        app.selected_line = 1;
        app.selected_widget = None;
        handle_key(&mut app, key(KeyCode::Char('d')));
        // After deleting index 1, cursor should stay at 1 (now pointing to old index 2)
        assert_eq!(app.editable.lines.len(), 2);
        assert_eq!(app.selected_line, 1);
    }

    #[test]
    fn down_walks_widgets_then_crosses_lines() {
        // ↑/↓ — flat-walk виджетов; ←/→ — переход между линиями.
        let mut app = make_app();
        let json = r#"{"lines":[{"widgets":[{"type":"model"},{"type":"version"}]},{"widgets":[{"type":"model"}]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        app.selected_line = 0;
        app.selected_widget = Some(0);
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!((app.selected_line, app.selected_widget), (0, Some(1)));
        handle_key(&mut app, key(KeyCode::Down));
        // Перешли на следующую линию, на её первый виджет.
        assert_eq!((app.selected_line, app.selected_widget), (1, Some(0)));
        handle_key(&mut app, key(KeyCode::Up));
        // Назад — последний виджет предыдущей линии.
        assert_eq!((app.selected_line, app.selected_widget), (0, Some(1)));
    }

    #[test]
    fn left_right_switches_lines_preserving_index() {
        let mut app = make_app();
        let json = r#"{"lines":[{"widgets":[{"type":"model"},{"type":"version"}]},{"widgets":[{"type":"model"}]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        app.selected_line = 0;
        app.selected_widget = Some(1);
        handle_key(&mut app, key(KeyCode::Right));
        // Линия 1 (1 виджет) — cursor зажат до 0.
        assert_eq!((app.selected_line, app.selected_widget), (1, Some(0)));
        handle_key(&mut app, key(KeyCode::Left));
        assert_eq!(app.selected_line, 0);
    }

    #[test]
    fn enter_in_lines_opens_palette_in_replace_mode() {
        let mut app = make_app();
        // make_app даёт line 0 с одним виджетом, selected_widget=Some(0)
        assert_eq!(app.selected_widget, Some(0));
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(app.palette_visible);
        assert_eq!(app.palette_mode, PaletteMode::Replace);
        assert_eq!(app.focus, Pane::Palette);
        assert!(matches!(app.editing_field, Some(EditField::PaletteFilter)));
    }

    #[test]
    fn a_in_lines_opens_palette_in_add_mode() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert!(app.palette_visible);
        assert_eq!(app.palette_mode, PaletteMode::Add);
        assert_eq!(app.focus, Pane::Palette);
        assert!(matches!(app.editing_field, Some(EditField::PaletteFilter)));
    }

    #[test]
    fn l_in_lines_adds_new_line() {
        let mut app = make_app();
        let before = app.editable.lines.len();
        handle_key(&mut app, key(KeyCode::Char('l')));
        assert_eq!(app.editable.lines.len(), before + 1);
        assert_eq!(app.selected_line, before);
    }

    #[test]
    fn palette_replace_mode_swaps_widget_kind() {
        use crate::types::config::WidgetConfig;
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Enter)); // open palette in replace mode
        // Двигаем фильтр на "version".
        for c in "version".chars() {
            handle_key(&mut app, key(KeyCode::Char(c)));
        }
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(!app.palette_visible);
        assert!(matches!(
            app.editable.lines[0].widgets[0].kind,
            WidgetConfig::Version
        ));
        assert_eq!(app.editable.lines[0].widgets.len(), 1);
    }

    #[test]
    fn palette_esc_closes_without_changes() {
        let mut app = make_app();
        let before_kind = format!("{:?}", app.editable.lines[0].widgets[0].kind);
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert!(app.palette_visible);
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(!app.palette_visible);
        assert_eq!(app.focus, Pane::Lines);
        assert_eq!(
            format!("{:?}", app.editable.lines[0].widgets[0].kind),
            before_kind
        );
    }

    #[test]
    fn enter_in_lines_does_nothing_without_widget() {
        let mut app = empty_app();
        app.editable.lines.push(Line::default()); // empty line, no widget
        app.selected_widget = None;
        app.focus = Pane::Lines;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.focus, Pane::Lines);
    }

    #[test]
    fn enter_on_fg_in_settings_opens_color_picker() {
        let mut app = make_app();
        app.focus = Pane::Settings;
        app.settings_field_cursor = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(matches!(
            app.editing_field,
            Some(EditField::ColorPicker {
                field: ColorField::Foreground
            })
        ));
    }

    #[test]
    fn color_picker_arrow_moves_cursor_and_enter_applies() {
        use crate::tui::widgets_ui::color_picker::NAMED_COLORS;
        let mut app = make_app();
        app.focus = Pane::Settings;
        app.settings_field_cursor = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(matches!(
            app.editing_field,
            Some(EditField::ColorPicker { .. })
        ));
        // Двигаемся вниз на "Red" (idx=2).
        handle_key(&mut app, key(KeyCode::Down));
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.color_fg_cursor, 2);
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(app.editing_field.is_none());
        let expected = NAMED_COLORS[2].1.to_string();
        assert_eq!(
            app.editable.lines[0].widgets[0].style.color,
            Some(expected)
        );
    }

    #[test]
    fn color_picker_custom_hex_transitions_to_hex_edit() {
        let mut app = make_app();
        app.focus = Pane::Settings;
        app.settings_field_cursor = 1;
        handle_key(&mut app, key(KeyCode::Enter));
        // Двигаем курсор на последнюю запись ("Custom hex…").
        app.color_bg_cursor = 17;
        handle_key(&mut app, key(KeyCode::Enter));
        assert!(matches!(
            app.editing_field,
            Some(EditField::ColorHex {
                field: ColorField::Background,
                ..
            })
        ));
    }

    #[test]
    fn enter_on_text_param_starts_text_edit() {
        let mut app = make_app();
        // Заменяем виджет на CustomText, где есть редактируемый text-param.
        let json = r#"{"lines":[{"widgets":[{"type":"custom-text","text":"hi"}]}]}"#;
        app.editable = serde_json::from_str(json).unwrap();
        app.focus = Pane::Settings;
        app.settings_field_cursor = 3; // first kind-specific field
        handle_key(&mut app, key(KeyCode::Enter));
        match &app.editing_field {
            Some(EditField::Text { field, buffer, .. }) => {
                assert_eq!(*field, SettingsField::CustomTextText);
                assert_eq!(buffer, "hi");
            }
            other => panic!("expected Text edit, got {other:?}"),
        }
    }

    #[test]
    fn settings_cursor_clamped_to_max_for_widget_kind() {
        let mut app = make_app();
        // make_app: один виджет Model — нет custom-параметров, max=2.
        app.focus = Pane::Settings;
        for _ in 0..10 {
            handle_key(&mut app, key(KeyCode::Down));
        }
        assert_eq!(app.settings_field_cursor, 2);
    }

    #[test]
    fn esc_in_settings_returns_focus_to_lines() {
        let mut app = make_app();
        app.focus = Pane::Settings;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.focus, Pane::Lines);
    }

    // --- Home screen tests (T3) ---

    #[test]
    fn home_down_arrow_advances_cursor() {
        let mut app = home_app();
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.home_cursor, 1);
    }

    #[test]
    fn home_up_arrow_at_zero_clamps() {
        let mut app = home_app();
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.home_cursor, 0);
    }

    #[test]
    fn home_down_arrow_clamps_at_three() {
        let mut app = home_app();
        for _ in 0..10 {
            handle_key(&mut app, key(KeyCode::Down));
        }
        assert_eq!(app.home_cursor, 3);
    }

    #[test]
    fn home_enter_on_edit_lines_switches_screen() {
        let mut app = home_app();
        app.home_cursor = 0;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::EditLines);
        assert_eq!(app.mode, Mode::Edit);
    }

    #[test]
    fn home_enter_on_choose_preset_switches_screen() {
        let mut app = home_app();
        app.home_cursor = 1;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::ChoosePreset);
    }

    #[test]
    fn home_enter_on_install_returns_run_install_effect() {
        let mut app = home_app();
        app.home_cursor = 2;
        let eff = handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(eff, ReducerEffect::RunInstall);
        assert_eq!(app.screen, Screen::Home);
    }

    #[test]
    fn home_enter_on_exit_quits_when_clean() {
        let mut app = home_app();
        app.home_cursor = 3;
        let eff = handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(eff, ReducerEffect::Quit);
    }

    #[test]
    fn home_q_when_dirty_opens_confirm_quit() {
        let mut app = home_app();
        app.editable.lines.push(Line::default());
        let eff = handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(eff, ReducerEffect::None);
        assert_eq!(app.mode, Mode::ConfirmQuit);
    }

    // --- Choose Preset tests (T5) ---

    #[test]
    fn home_enter_on_choose_preset_loads_presets() {
        let mut app = home_app();
        app.home_cursor = 1;
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::ChoosePreset);
        assert_eq!(app.presets.len(), 5);
    }

    #[test]
    fn choose_preset_down_advances_within_bounds() {
        let mut app = home_app();
        app.presets = crate::tui::presets::list_all();
        app.screen = Screen::ChoosePreset;
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.preset_cursor, 1);
    }

    #[test]
    fn choose_preset_down_clamps_at_last() {
        let mut app = home_app();
        app.presets = crate::tui::presets::list_all();
        app.screen = Screen::ChoosePreset;
        app.preset_cursor = app.presets.len() - 1;
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.preset_cursor, app.presets.len() - 1);
    }

    #[test]
    fn choose_preset_enter_applies_and_returns_home() {
        let mut app = home_app();
        app.presets = crate::tui::presets::list_all();
        app.screen = Screen::ChoosePreset;
        app.preset_cursor = 0; // "minimal"
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Home);
        assert!(!app.editable.lines.is_empty());
        assert!(app.status_message.is_some());
    }

    #[test]
    fn choose_preset_esc_returns_home_without_apply() {
        let mut app = home_app();
        app.presets = crate::tui::presets::list_all();
        app.screen = Screen::ChoosePreset;
        let lines_before = app.editable.lines.clone();
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);
        assert_eq!(app.editable.lines, lines_before);
    }

    // --- ConfirmReturnHome / Esc-routing tests (T6) ---

    #[test]
    fn edit_lines_esc_when_clean_returns_home() {
        let mut app = make_app();
        app.mode = Mode::Edit;
        app.focus = Pane::Lines;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);
    }

    #[test]
    fn edit_lines_esc_when_dirty_opens_confirm_return_home() {
        let mut app = make_app();
        app.mode = Mode::Edit;
        app.focus = Pane::Lines;
        app.editable.lines.push(Line::default());
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::ConfirmReturnHome);
        assert_eq!(app.screen, Screen::EditLines);
    }

    #[test]
    fn confirm_return_home_save_returns_save_effect() {
        let mut app = make_app();
        app.mode = Mode::ConfirmReturnHome;
        let eff = handle_key(&mut app, key(KeyCode::Char('s')));
        assert_eq!(eff, ReducerEffect::RequestSaveAndReturnHome);
    }

    #[test]
    fn confirm_return_home_discard_returns_discard_effect() {
        let mut app = make_app();
        app.mode = Mode::ConfirmReturnHome;
        let eff = handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(eff, ReducerEffect::RequestDiscardAndReturnHome);
    }

    #[test]
    fn confirm_return_home_cancel_stays_in_edit_lines() {
        let mut app = make_app();
        app.mode = Mode::ConfirmReturnHome;
        handle_key(&mut app, key(KeyCode::Char('c')));
        assert_eq!(app.mode, Mode::Edit);
        assert_eq!(app.screen, Screen::EditLines);
    }
}
