//! TUI App state.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use tempfile::NamedTempFile;

use crate::types::config::Settings;
use crate::types::payload::StatusPayload;

/// Top-level UI mode. Edit — нормальное редактирование, panel focused.
/// Overlays open over Edit. `ConfirmQuit` — modal `[s/d/c]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // wired in T6/T7
pub enum Mode {
    Edit,
    ThemesOverlay,
    HelpOverlay,
    ConfirmQuit,
    /// Esc из `EditLines` с dirty-state — ждём `s`/`d`/`c`.
    ConfirmReturnHome,
    /// `Ctrl+P` в `EditLines` — input для имени пресета.
    PresetNamePrompt,
    /// Home → Enter на "Install to Claude Code" — ждём `o`/`c`.
    ConfirmInstall,
}

/// Top-level screen — определяет, какой root-render использовать.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // wired in T3/T5
pub enum Screen {
    Home,
    EditLines,
    ChoosePreset,
}

/// 4 видимые панели. `focus` field of App.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Lines,
    Palette,
    Settings,
    Preview,
}

/// Контекст открытия палитры виджетов.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    /// Добавить новый виджет в текущую линию.
    Add,
    /// Заменить тип у `selected_widget`.
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MessageKind {
    Info,
    Warn,
    Error,
}

/// Какое поле редактируется по типу. Используется в Settings panel и Palette filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SettingsField {
    Color,
    BackgroundColor,
    Bold,
    CustomTextText,
    CustomSymbolSymbol,
    LinkUrl,
    LinkLabel,
    CustomCommandCommand,
    CustomCommandTimeoutMs,
    /// Index в `params.args` (`CustomCommand` only).
    CustomCommandArgs(usize),
    ContextBarWidth,
    CurrentWorkingDirSegments,
    CurrentWorkingDirAbbreviateHome,
    CurrentWorkingDirFishStyle,
    CurrentWorkingDirPrefix,
    /// Theme global (Themes overlay):
    ThemeGlobalBold,
    ThemeInheritSeparators,
    ThemeOverrideBg,
    ThemeOverrideFg,
    ThemeMinimalist,
    ThemeFlexMode,
    ThemeCompactThreshold,
    ThemeAutoAlign,
    ThemeContinueAcrossLines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ColorField {
    Foreground,
    Background,
}

/// Active edit-mode под курсором: Esc выходит → None. Печать символов в `App.editing_field`
/// модифицирует `buffer`/`cursor` соответствующего варианта.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EditField {
    PaletteFilter,
    /// Named-color picker submode: ↑/↓ navigate `color_*_cursor`, Enter applies
    /// hex (or transitions to `ColorHex` for "Custom hex…" entry), Esc cancels.
    ColorPicker {
        field: ColorField,
    },
    Text {
        field: SettingsField,
        buffer: String,
        cursor: usize,
    },
    Number {
        field: SettingsField,
        buffer: String, // только digits
    },
    ColorHex {
        field: ColorField,
        buffer: String,
        cursor: usize,
    },
}

pub struct App {
    pub editable: Settings,
    pub initial: Settings,
    pub sample_payload: StatusPayload,
    /// RAII держит `NamedTempFile` до Drop App'а. Не Pub — нужен только для жизни fixture-файла.
    _sample_transcript: Option<NamedTempFile>,

    pub mode: Mode,
    pub focus: Pane,

    pub selected_line: usize,
    /// None если линия пуста.
    pub selected_widget: Option<usize>,
    pub palette_filter: String,
    pub palette_cursor: usize,
    pub settings_field_cursor: usize,
    /// Navigation cursor inside the FG color picker (index into `NAMED_COLORS`).
    pub color_fg_cursor: usize,
    /// Navigation cursor inside the BG color picker (index into `NAMED_COLORS`).
    pub color_bg_cursor: usize,
    pub editing_field: Option<EditField>,
    pub theme_field_cursor: usize,
    pub status_message: Option<(String, MessageKind)>,
    /// Палитра скрыта по умолчанию; открывается из Lines (`a` add / Enter replace).
    pub palette_visible: bool,
    pub palette_mode: PaletteMode,

    /// Текущий top-level экран. Стартует на `Home`.
    #[allow(dead_code)] // wired in T3
    pub screen: Screen,
    /// Курсор в Home-меню (0..3).
    #[allow(dead_code)] // wired in T3
    pub home_cursor: usize,
    /// Курсор в Choose Preset списке (индекс в `presets`).
    #[allow(dead_code)] // wired in T5
    pub preset_cursor: usize,
    /// Загружается лениво при входе в Choose Preset.
    #[allow(dead_code)] // wired in T5
    pub presets: Vec<crate::tui::presets::Preset>,
    /// Буфер ввода в `Mode::PresetNamePrompt`.
    #[allow(dead_code)] // wired in T7
    pub preset_name_buffer: String,
    /// Курсор в `ConfirmInstall` modal: 0 = Overwrite, 1 = Cancel.
    pub confirm_install_cursor: usize,
}

impl App {
    #[must_use]
    pub fn new(
        settings: Settings,
        sample_payload: StatusPayload,
        sample_transcript: Option<NamedTempFile>,
    ) -> Self {
        let initial = settings.clone();
        let selected_widget = settings
            .lines
            .first()
            .and_then(|l| if l.widgets.is_empty() { None } else { Some(0) });
        Self {
            editable: settings,
            initial,
            sample_payload,
            _sample_transcript: sample_transcript,
            mode: Mode::Edit,
            focus: Pane::Lines,
            selected_line: 0,
            selected_widget,
            palette_filter: String::new(),
            palette_cursor: 0,
            settings_field_cursor: 0,
            color_fg_cursor: 0,
            color_bg_cursor: 0,
            editing_field: None,
            theme_field_cursor: 0,
            status_message: None,
            palette_visible: false,
            palette_mode: PaletteMode::Add,
            screen: Screen::Home,
            home_cursor: 0,
            preset_cursor: 0,
            presets: Vec::new(),
            preset_name_buffer: String::new(),
            confirm_install_cursor: 0,
        }
    }

    /// Структурное сравнение через `PartialEq` derive chain.
    /// True если user сделал изменения относительно начального состояния.
    #[must_use]
    pub fn dirty(&self) -> bool {
        self.editable != self.initial
    }

    /// Reset editable к initial (Discard действие из `ConfirmQuit` modal).
    pub fn discard(&mut self) {
        self.editable = self.initial.clone();
    }

    /// После успешного save — initial = editable (теперь чистое состояние).
    pub fn mark_saved(&mut self) {
        self.initial = self.editable.clone();
    }

    /// Текущая выбранная линия (mut). None если `selected_line` за пределами.
    #[allow(dead_code)]
    pub fn current_line_mut(&mut self) -> Option<&mut crate::types::config::Line> {
        self.editable.lines.get_mut(self.selected_line)
    }

    /// Текущий выбранный widget item (mut). None если линия пуста или `selected_widget` = None.
    pub fn current_widget_mut(&mut self) -> Option<&mut crate::types::config::WidgetItem> {
        let idx = self.selected_widget?;
        self.editable
            .lines
            .get_mut(self.selected_line)?
            .widgets
            .get_mut(idx)
    }

    /// Текущий выбранный widget item (immut).
    #[must_use]
    pub fn current_widget(&self) -> Option<&crate::types::config::WidgetItem> {
        let idx = self.selected_widget?;
        self.editable
            .lines
            .get(self.selected_line)?
            .widgets
            .get(idx)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::tui::sample;

    fn empty_app() -> App {
        let (p, f) = sample::payload();
        App::new(Settings::default(), p, f)
    }

    #[test]
    fn new_sets_focus_to_lines_and_mode_to_edit() {
        let app = empty_app();
        assert_eq!(app.focus, Pane::Lines);
        assert_eq!(app.mode, Mode::Edit);
    }

    #[test]
    fn dirty_starts_false() {
        let app = empty_app();
        assert!(!app.dirty());
    }

    #[test]
    fn dirty_becomes_true_after_mutation() {
        use crate::types::config::Line;
        let mut app = empty_app();
        app.editable.lines.push(Line::default());
        assert!(app.dirty());
    }

    #[test]
    fn discard_resets_editable_to_initial() {
        use crate::types::config::Line;
        let mut app = empty_app();
        app.editable.lines.push(Line::default());
        assert!(app.dirty());
        app.discard();
        assert!(!app.dirty());
    }

    #[test]
    fn mark_saved_clears_dirty_flag() {
        use crate::types::config::Line;
        let mut app = empty_app();
        app.editable.lines.push(Line::default());
        assert!(app.dirty());
        app.mark_saved();
        assert!(!app.dirty());
    }

    #[test]
    fn selected_widget_is_none_for_empty_default_line() {
        let app = empty_app();
        // Settings::default() has no lines → selected_widget = None.
        assert_eq!(app.selected_widget, None);
    }

    #[test]
    fn current_widget_handles_empty_state() {
        let app = empty_app();
        assert!(app.current_widget().is_none());
    }

    #[test]
    fn new_sets_screen_to_home() {
        let app = empty_app();
        assert_eq!(app.screen, Screen::Home);
    }

    #[test]
    fn new_initializes_home_cursor_at_zero() {
        let app = empty_app();
        assert_eq!(app.home_cursor, 0);
    }

    #[test]
    fn new_initializes_preset_state_empty() {
        let app = empty_app();
        assert!(app.presets.is_empty());
        assert_eq!(app.preset_cursor, 0);
        assert!(app.preset_name_buffer.is_empty());
    }
}
