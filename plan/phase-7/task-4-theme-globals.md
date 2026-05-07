# Task 4 — ThemeConfig +9, FlexMode, AlignRight, apply_widget_style, RenderState

**Цель:** Расширить `ThemeConfig` 9 глобальными настройками, ввести `FlexMode` enum, добавить `WidgetConfig::AlignRight` sentinel и Phase 7 widget-варианты в enum, реализовать функцию `apply_widget_style` (override stack) и `RenderState` (theme/separator cursor).

**Files:**
- Modify: `src/types/config.rs` — `ThemeConfig` +9 полей, `FlexMode` enum, `WidgetConfig::AlignRight` + 7 Phase 7 widget вариантов
- Modify: `src/render/mod.rs` — `apply_widget_style` функция, `RenderState` struct
- Modify: `src/widgets/mod.rs::build_one` — temp stub для Phase 7 enum-вариантов (T8/T9/T10/T11 заменят)
- Modify: `src/widgets/trivial.rs` — добавить `Stub` placeholder

---

- [ ] **Step 1: Написать failing test для FlexMode default + theme globals**

В `src/types/config.rs::tests`:

```rust
#[test]
fn flex_mode_default_is_full() {
    let s = Settings::default();
    assert_eq!(s.theme.flex_mode, FlexMode::Full);
    assert!(!s.theme.global_bold);
    assert_eq!(s.theme.compact_threshold, 60);
    assert!(!s.theme.auto_align);
    assert!(!s.theme.continue_theme_across_lines);
}

#[test]
fn theme_globals_parse_from_json() {
    let json = r#"{
        "theme": {
            "global_bold": true,
            "inherit_separator_colors": true,
            "override_background_color": "#aabbcc",
            "minimalist_mode": true,
            "flex_mode": "full-minus-40",
            "compact_threshold": 80,
            "auto_align": true,
            "continue_theme_across_lines": true
        }
    }"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    assert!(s.theme.global_bold);
    assert!(s.theme.inherit_separator_colors);
    assert_eq!(s.theme.override_background_color.as_deref(), Some("#aabbcc"));
    assert!(s.theme.minimalist_mode);
    assert_eq!(s.theme.flex_mode, FlexMode::FullMinus40);
    assert_eq!(s.theme.compact_threshold, 80);
    assert!(s.theme.auto_align);
    assert!(s.theme.continue_theme_across_lines);
}

#[test]
fn align_right_widget_parses() {
    let json = r#"{"lines": [{"widgets": [{"type": "align-right"}]}]}"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    assert!(matches!(s.lines[0].widgets[0].kind, WidgetConfig::AlignRight));
}

#[test]
fn phase7_widget_variants_parse() {
    let json = r#"[
        {"type": "session-usage"}, {"type": "weekly-usage"},
        {"type": "block-reset-timer"}, {"type": "weekly-reset-timer"},
        {"type": "claude-account-email"}, {"type": "free-memory"},
        {"type": "skills"}
    ]"#;
    let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0].kind, WidgetConfig::SessionUsage));
    assert!(matches!(widgets[6].kind, WidgetConfig::Skills));
}
```

- [ ] **Step 2: Run failing tests**

Run: `cargo test --lib types::config::tests::flex_mode_default_is_full types::config::tests::theme_globals_parse_from_json types::config::tests::align_right_widget_parses types::config::tests::phase7_widget_variants_parse`
Expected: FAIL.

- [ ] **Step 3: Расширить `ThemeConfig` 9 полями + `FlexMode`**

В `src/types/config.rs` заменить определение `ThemeConfig`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default)]
    pub kind: ThemeKind,
    #[serde(default)]
    pub theme_name: Option<String>,
    #[serde(default)]
    pub custom: Option<crate::render::themes::PowerlineTheme>,
    #[serde(default)]
    pub separators: Vec<String>,
    #[serde(default)]
    pub start_caps: Vec<String>,
    #[serde(default)]
    pub end_caps: Vec<String>,
    #[serde(default)]
    pub color_level: Option<crate::render::ColorLevel>,

    // Phase 7 — global theme settings (7.0b):
    #[serde(default)]
    pub global_bold: bool,
    #[serde(default)]
    pub inherit_separator_colors: bool,
    #[serde(default)]
    pub override_background_color: Option<String>,
    #[serde(default)]
    pub override_foreground_color: Option<String>,
    #[serde(default)]
    pub minimalist_mode: bool,
    #[serde(default)]
    pub flex_mode: FlexMode,
    #[serde(default = "default_compact_threshold")]
    pub compact_threshold: u32,
    #[serde(default)]
    pub auto_align: bool,
    #[serde(default)]
    pub continue_theme_across_lines: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FlexMode {
    #[default]
    Full,
    FullMinus20,
    FullMinus40,
    Disabled,
}

const fn default_compact_threshold() -> u32 { 60 }
```

- [ ] **Step 4: Добавить `WidgetConfig::AlignRight` + 7 Phase 7 widget вариантов**

В `src/types/config.rs` в `pub enum WidgetConfig` после Phase 6 строк добавить:

```rust
    // Phase 7 — usage cluster (payload.rate_limits):
    SessionUsage,
    WeeklyUsage,
    BlockResetTimer,
    WeeklyResetTimer,

    // Phase 7 — env cluster:
    ClaudeAccountEmail,
    FreeMemory,

    // Phase 7 — transcript meta:
    Skills,

    // Phase 7 — sentinel for auto_align (не считается в "60 widgets"):
    AlignRight,
```

- [ ] **Step 5: Run config tests**

Run: `cargo test --lib types::config`
Expected: PASS — все Phase 7 tests + Phase 3/5/6 без регрессии.

- [ ] **Step 6: Добавить `RenderState` + `apply_widget_style` в `render/mod.rs`**

В `src/render/mod.rs` после `pub mod themes;`, перед `pub enum Renderer`:

```rust
#[derive(Debug, Default, Clone)]
pub struct RenderState {
    pub global_theme_index: usize,
    pub global_separator_index: usize,
}

impl RenderState {
    pub fn reset(&mut self) {
        self.global_theme_index = 0;
        self.global_separator_index = 0;
    }
}

/// Композирует финальный `Style` для одного виджета.
/// Порядок применения: widget default → theme.widget_styles[id] → per-widget override → theme globals.
#[must_use]
pub fn apply_widget_style(
    widget_default: Style,
    theme_widget_style: Option<Style>,
    per_widget_override: &crate::types::config::WidgetStyleOverride,
    theme_globals: &crate::types::config::ThemeConfig,
) -> Style {
    let mut style = theme_widget_style.unwrap_or(widget_default);

    // 7.0a — per-widget override:
    if let Some(c) = color_parse::parse_color(per_widget_override.color.as_deref()) {
        style.fg = Some(c);
    }
    if let Some(c) = color_parse::parse_color(per_widget_override.background_color.as_deref()) {
        style.bg = Some(c);
    }
    if let Some(b) = per_widget_override.bold {
        style.bold = b;
    }

    // 7.0b — theme globals (force-override per upstream semantics):
    if theme_globals.global_bold {
        style.bold = true;
    }
    if let Some(c) = color_parse::parse_color(theme_globals.override_foreground_color.as_deref()) {
        style.fg = Some(c);
    }
    if let Some(c) = color_parse::parse_color(theme_globals.override_background_color.as_deref()) {
        style.bg = Some(c);
    }

    style
}
```

- [ ] **Step 7: Тест для apply_widget_style**

В `src/render/mod.rs`:

```rust
#[cfg(test)]
mod apply_style_tests {
    use super::*;
    use crate::types::config::{ThemeConfig, WidgetStyleOverride};

    fn theme() -> ThemeConfig { ThemeConfig::default() }

    #[test]
    fn returns_widget_default_when_no_overrides() {
        let dflt = Style::none().bold();
        let theme = theme();
        let ovr = WidgetStyleOverride::default();
        let s = apply_widget_style(dflt, None, &ovr, &theme);
        assert!(s.bold);
        assert!(s.fg.is_none());
    }

    #[test]
    fn theme_widget_style_overrides_default() {
        let dflt = Style::none().bold();
        let theme_style = Style::none().fg(Color::Rgb(255, 0, 0));
        let theme = theme();
        let ovr = WidgetStyleOverride::default();
        let s = apply_widget_style(dflt, Some(theme_style), &ovr, &theme);
        assert_eq!(s.fg, Some(Color::Rgb(255, 0, 0)));
        assert!(!s.bold, "theme style replaced widget default entirely");
    }

    #[test]
    fn per_widget_override_changes_color_and_bold() {
        let mut ovr = WidgetStyleOverride::default();
        ovr.color = Some("#fafafa".into());
        ovr.bold = Some(true);
        let s = apply_widget_style(Style::none(), None, &ovr, &theme());
        assert_eq!(s.fg, Some(Color::Rgb(0xfa, 0xfa, 0xfa)));
        assert!(s.bold);
    }

    #[test]
    fn global_bold_force_enables_after_override() {
        let mut theme = theme();
        theme.global_bold = true;
        let mut ovr = WidgetStyleOverride::default();
        ovr.bold = Some(false); // user disabled bold per-widget…
        let s = apply_widget_style(Style::none(), None, &ovr, &theme);
        assert!(s.bold, "global_bold force-enables after per-widget override");
    }

    #[test]
    fn override_foreground_color_wins_over_per_widget() {
        let mut theme = theme();
        theme.override_foreground_color = Some("#aabbcc".into());
        let mut ovr = WidgetStyleOverride::default();
        ovr.color = Some("#000000".into());
        let s = apply_widget_style(Style::none(), None, &ovr, &theme);
        assert_eq!(s.fg, Some(Color::Rgb(0xaa, 0xbb, 0xcc)));
    }
}
```

- [ ] **Step 8: Run apply_widget_style tests**

Run: `cargo test --lib render::apply_style_tests`
Expected: PASS — 5 тестов.

- [ ] **Step 9: Stub для новых WidgetConfig вариантов в build_one**

В `src/widgets/trivial.rs` добавить:

```rust
/// Phase 7 setup stub. T8/T9/T10/T11 заменят на реальные виджеты.
#[allow(dead_code)]
pub struct Stub;

impl super::Widget for Stub {
    fn id(&self) -> &'static str { "stub" }
    fn render(&self, _: &super::RenderContext<'_>) -> Option<String> { None }
}
```

В `src/widgets/mod.rs::build_one` добавить временный блок (T8/T9/T10/T11 заменят):

```rust
        // Phase 7 — temp stubs (T8/T9/T10 заменят):
        WidgetConfig::SessionUsage
        | WidgetConfig::WeeklyUsage
        | WidgetConfig::BlockResetTimer
        | WidgetConfig::WeeklyResetTimer
        | WidgetConfig::ClaudeAccountEmail
        | WidgetConfig::FreeMemory
        | WidgetConfig::Skills => Box::new(trivial::Stub),

        // Phase 7 — auto_align sentinel (T11 даст специальную обработку):
        WidgetConfig::AlignRight => Box::new(trivial::Stub),
```

- [ ] **Step 10: Run full test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 11: Commit**

```bash
git add src/types/config.rs src/render/mod.rs src/widgets/mod.rs src/widgets/trivial.rs
git commit -m "feat(phase-7): T4 — ThemeConfig +9 fields, FlexMode, AlignRight, apply_widget_style, RenderState"
```
