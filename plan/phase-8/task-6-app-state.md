# Task 6 — `tui::app` (App state, Mode, Pane, EditField, SettingsField)

**Цель:** Состояние TUI. `App` хранит `editable` и `initial: Settings` (для diff/discard), sample payload, RAII tempfile, focus/mode/cursors. Reducer (T7) мутирует `App` чисто, IO (T11/T12) читает `App` для side effects.

**Files:**
- Modify: `src/tui/app.rs` — наполнить (был skeleton после T1)

---

- [ ] **Step 1: Реализовать state types и `App::new`**

В `src/tui/app.rs`:

```rust
//! TUI App state — Phase 8 Task 6.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use tempfile::NamedTempFile;

use crate::types::config::Settings;
use crate::types::payload::StatusPayload;

/// Top-level UI mode. Edit — нормальное редактирование, panel focused.
/// Overlays open over Edit. ConfirmQuit — modal `[s/d/c]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Edit,
    ThemesOverlay,
    HelpOverlay,
    ConfirmQuit,
}

/// 4 видимые панели. `focus` field of App.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Lines,
    Palette,
    Settings,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Warn,
    Error,
}

/// Какое поле редактируется по типу. Используется в Settings panel и Palette filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Index в `params.args` (CustomCommand only).
    CustomCommandArgs(usize),
    ContextBarWidth,
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
pub enum ColorField {
    Foreground,
    Background,
}

/// Active edit-mode под курсором: Esc выходит → None. Печать символов в `App.editing_field`
/// модифицирует `buffer`/`cursor` соответствующего варианта.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditField {
    PaletteFilter,
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
    /// RAII держит NamedTempFile до Drop App'а. Не Pub — нужен только для жизни fixture-файла.
    _sample_transcript: Option<NamedTempFile>,

    pub mode: Mode,
    pub focus: Pane,

    pub selected_line: usize,
    /// None если линия пуста.
    pub selected_widget: Option<usize>,
    pub palette_filter: String,
    pub palette_cursor: usize,
    pub settings_field_cursor: usize,
    pub editing_field: Option<EditField>,
    pub theme_field_cursor: usize,
    pub status_message: Option<(String, MessageKind)>,
}

impl App {
    #[must_use]
    pub fn new(
        settings: Settings,
        sample_payload: StatusPayload,
        sample_transcript: Option<NamedTempFile>,
    ) -> Self {
        let initial = settings.clone();
        let selected_widget = settings.lines.first().map(|l| l.widgets.is_empty()).map_or(None, |empty| if empty { None } else { Some(0) });
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
            editing_field: None,
            theme_field_cursor: 0,
            status_message: None,
        }
    }

    /// Структурное сравнение через PartialEq (Phase 8 T1 derive chain).
    /// True если user сделал изменения относительно начального состояния.
    #[must_use]
    pub fn dirty(&self) -> bool {
        self.editable != self.initial
    }

    /// Reset editable к initial (Discard действие из ConfirmQuit modal).
    pub fn discard(&mut self) {
        self.editable = self.initial.clone();
    }

    /// После успешного save — initial = editable (теперь чистое состояние).
    pub fn mark_saved(&mut self) {
        self.initial = self.editable.clone();
    }

    /// Текущая выбранная линия (mut). None если `selected_line` за пределами.
    pub fn current_line_mut(&mut self) -> Option<&mut crate::types::config::Line> {
        self.editable.lines.get_mut(self.selected_line)
    }

    /// Текущий выбранный widget item (mut). None если линия пуста или selected_widget = None.
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
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test --features tui --locked tui::app
```

Expected: 7 PASS. Critical: `dirty_becomes_true_after_mutation` — упражняет PartialEq derive chain из T1.

- [ ] **Step 3: Verify --no-default-features build**

```bash
cargo build --locked --no-default-features
```

Expected: PASS.

- [ ] **Step 4: Lints**

```bash
cargo clippy --locked --features tui --all-targets -- -D warnings
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tui/app.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T6 app state — App, Mode, Pane, EditField, SettingsField

- Mode { Edit, ThemesOverlay, HelpOverlay, ConfirmQuit }
- Pane { Lines, Palette, Settings, Preview }
- EditField { PaletteFilter, Text, Number, ColorHex } — active text editing
- SettingsField — typed enum for per-widget + theme global field cursors
- App holds editable + initial Settings, sample_payload, RAII NamedTempFile
- App::dirty() = editable != initial via PartialEq derive chain (T1)
- 7 unit-tests covering dirty diff, discard, mark_saved, current_widget access

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
