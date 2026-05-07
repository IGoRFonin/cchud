# Task 3 — WidgetItem wrapper + WidgetStyleOverride + parse_color

**Цель:** Заменить `Line.widgets: Vec<WidgetConfig>` на `Vec<WidgetItem>` через flatten-wrapper, добавить per-widget overrides (`color`/`background_color`/`bold`). Реализовать `parse_color` для hex/ANSI-256. Механически мигрировать Phase 3/5/6 unit-тесты.

**Files:**
- Modify: `src/types/config.rs` — `Line.widgets: Vec<WidgetItem>`, новый `WidgetItem` + `WidgetStyleOverride`, миграция unit-тестов
- Modify: `src/widgets/mod.rs:114-120` — `build_widgets` принимает `&[WidgetItem]`, проброс `WidgetStyleOverride`
- Modify: `src/main.rs:69-78` — обновить вызов `build_widgets`
- Create: `src/render/color_parse.rs` — `parse_color(&str) -> Option<Color>`
- Modify: `src/render/mod.rs` — `pub mod color_parse;`

---

- [ ] **Step 1: Написать failing test для парсинга WidgetItem**

В `mod tests` `src/types/config.rs`:

```rust
#[test]
fn widget_item_parses_with_style_overrides() {
    let json = r#"{
        "lines": [{"widgets": [
            {"type": "model", "color": "#fafafa", "bold": true},
            {"type": "git-branch", "background_color": "#00ff00"}
        ]}]
    }"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    let w0 = &s.lines[0].widgets[0];
    assert!(matches!(w0.kind, WidgetConfig::Model { .. }));
    assert_eq!(w0.style.color.as_deref(), Some("#fafafa"));
    assert_eq!(w0.style.bold, Some(true));
    let w1 = &s.lines[0].widgets[1];
    assert_eq!(w1.style.background_color.as_deref(), Some("#00ff00"));
    assert!(w1.style.bold.is_none());
}

#[test]
fn widget_item_without_style_keeps_kind() {
    let json = r#"{"lines": [{"widgets": [{"type": "version"}]}]}"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    let w = &s.lines[0].widgets[0];
    assert!(matches!(w.kind, WidgetConfig::Version));
    assert!(w.style.color.is_none());
    assert!(w.style.bold.is_none());
}
```

- [ ] **Step 2: Run failing test**

Run: `cargo test --lib types::config::tests::widget_item_parses_with_style_overrides`
Expected: FAIL (нет поля `widgets[i].kind` / `widgets[i].style`).

- [ ] **Step 3: Заменить `Line` + ввести `WidgetItem` / `WidgetStyleOverride`**

В `src/types/config.rs` заменить `pub struct Line { ... }` блок:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Line {
    #[serde(default)]
    pub widgets: Vec<WidgetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetItem {
    #[serde(flatten)]
    pub kind: WidgetConfig,
    #[serde(flatten, default)]
    pub style: WidgetStyleOverride,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WidgetStyleOverride {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub bold: Option<bool>,
}
```

- [ ] **Step 4: Механически обновить существующие unit-тесты**

В `src/types/config.rs` все `s.lines[X].widgets[Y]` → `s.lines[X].widgets[Y].kind` (тесты `parses_minimal_cchud_block`, `serde_roundtrip_preserves_shape`, `parses_phase3_widget_kinds`, `context_bar_default_width_is_ten`, `parses_phase5_*`, `parses_phase6_*`).

Для prebuilt `Vec<WidgetConfig>`-тестов (например `parses_phase5_head_widgets`):

```rust
#[test]
fn parses_phase5_head_widgets() {
    let json = r#"[
        { "type": "git-branch" },
        { "type": "git-sha" },
        { "type": "git-root-dir" }
    ]"#;
    let widgets: Vec<WidgetItem> = serde_json::from_str(json).unwrap();
    assert!(matches!(widgets[0].kind, WidgetConfig::GitBranch));
    assert!(matches!(widgets[1].kind, WidgetConfig::GitSha));
    assert!(matches!(widgets[2].kind, WidgetConfig::GitRootDir));
}
```

И аналогично для всех `let widgets: Vec<WidgetConfig> = ...` → `Vec<WidgetItem>` + `widgets[i]` → `widgets[i].kind`. Применить ко всем 8 тестам секции `parses_phase{3,5,6}_*`.

В тесте `serde_roundtrip_preserves_shape` обновить literal:

```rust
let original = Settings {
    version: 1,
    lines: vec![Line {
        widgets: vec![WidgetItem {
            kind: WidgetConfig::Model {
                params: ModelParams::default(),
            },
            style: WidgetStyleOverride::default(),
        }],
    }],
    theme: ThemeConfig::default(),
};
```

- [ ] **Step 5: Обновить `build_widgets` в `widgets/mod.rs`**

Заменить body `build_widgets`:

```rust
#[must_use]
pub fn build_widgets(settings: &Settings) -> Vec<(Box<dyn Widget>, WidgetStyleOverride)> {
    settings
        .lines
        .first()
        .map(|line| {
            line.widgets
                .iter()
                .map(|item| (build_one(&item.kind), item.style.clone()))
                .collect()
        })
        .unwrap_or_default()
}
```

И обновить use:

```rust
use crate::types::{
    config::{Settings, WidgetConfig, WidgetStyleOverride},
    payload::StatusPayload,
};
```

(NB: T12 расширит `build_widgets` до возврата `Vec<Vec<...>>` — пока first-line only.)

- [ ] **Step 6: Обновить `main.rs::render_pipeline`**

Заменить блок `let segments: Vec<Segment> = widgets...` на:

```rust
let widget_items = build_widgets(&settings);
let segments: Vec<Segment> = widget_items
    .iter()
    .filter_map(|(w, _override)| {
        w.render(&ctx).map(|text| Segment {
            text,
            style: w.default_style(),
            hyperlink: w.hyperlink(&ctx),
        })
    })
    .collect();
```

(Override-параметр игнорируется до T4. Поле `align_marker` появится в T11.)

- [ ] **Step 7: Создать `src/render/color_parse.rs`**

```rust
//! Phase 7: parse hex/ANSI-256 colors from per-widget overrides + theme globals.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::Color;

/// Парсит `#rrggbb` или ANSI-256 имя (`bright_red`, `cyan`, ...). None при невалидном вводе.
#[must_use]
pub fn parse_color(s: Option<&str>) -> Option<Color> {
    let s = s?.trim();
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    parse_ansi_named(s)
}

fn parse_hex(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn parse_ansi_named(name: &str) -> Option<Color> {
    let n = match name.to_ascii_lowercase().as_str() {
        "black" => 0,
        "red" => 1,
        "green" => 2,
        "yellow" => 3,
        "blue" => 4,
        "magenta" => 5,
        "cyan" => 6,
        "white" => 7,
        "bright_black" | "gray" | "grey" => 8,
        "bright_red" => 9,
        "bright_green" => 10,
        "bright_yellow" => 11,
        "bright_blue" => 12,
        "bright_magenta" => 13,
        "bright_cyan" => 14,
        "bright_white" => 15,
        _ => return None,
    };
    Some(Color::Ansi256(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses() {
        assert_eq!(parse_color(Some("#fafafa")), Some(Color::Rgb(0xfa, 0xfa, 0xfa)));
        assert_eq!(parse_color(Some("#000000")), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn ansi_named_parses() {
        assert_eq!(parse_color(Some("red")), Some(Color::Ansi256(1)));
        assert_eq!(parse_color(Some("bright_red")), Some(Color::Ansi256(9)));
        assert_eq!(parse_color(Some("CYAN")), Some(Color::Ansi256(6)));
    }

    #[test]
    fn invalid_returns_none() {
        assert!(parse_color(Some("monokai")).is_none());
        assert!(parse_color(Some("#zz0000")).is_none());
        assert!(parse_color(Some("#fff")).is_none()); // short hex not supported
        assert!(parse_color(None).is_none());
    }
}
```

В `src/render/mod.rs` после `pub mod color_sanitize;`:

```rust
pub mod color_parse;
```

- [ ] **Step 8: Проверить snapshot tests — Phase 4 byte-identical**

Run: `cargo test --test snapshots --locked`
Run: `cargo test --test snapshots_transcript --locked`
Run: `cargo test --test snapshots_git --locked`
Expected: PASS — все snapshot'ы byte-identical (default `WidgetStyleOverride` пуст, identity behavior).

- [ ] **Step 9: Run full test suite**

Run: `cargo test --locked`
Expected: PASS — все Phase 3/5/6 тесты + новые Phase 7 тесты.

- [ ] **Step 10: Commit**

```bash
git add src/types/config.rs src/widgets/mod.rs src/main.rs \
        src/render/color_parse.rs src/render/mod.rs
git commit -m "feat(phase-7): T3 — WidgetItem wrapper + WidgetStyleOverride + parse_color"
```
