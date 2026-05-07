# Task 1 — Setup (deps, features, skeleton, PartialEq derive chain, DECISIONS)

**Цель:** Подготовить почву для Phase 8. Добавить `ratatui`/`crossterm` под feature `tui` (default), поднять `tempfile` в `[dependencies]` (optional), создать пустые skeleton-модули `src/tui/*` + `src/commands/{configure,import}.rs`, проставить `PartialEq, Eq` derive на всю Settings chain (для `App.dirty` diff), записать DECISIONS-запись D-2026-05-01.

**Files:**
- Modify: `Cargo.toml:14-58` — `[features]` блок, `ratatui`/`crossterm`/`tempfile` deps
- Modify: `src/lib.rs` — `#[cfg(feature = "tui")] pub mod tui;`
- Modify: `src/main.rs:9-17` — добавить `mod tui;` под cfg
- Modify: `src/commands/mod.rs` — `#[cfg(feature = "tui")] pub mod {configure, import};`
- Modify: `src/types/config.rs:10-279` — derive `PartialEq, Eq` chain
- Modify: `src/types/payload.rs` — derive `PartialEq` где нужно для App.sample_payload (только Clone уже есть)
- Modify: `src/render/mod.rs:50-59` — derive `Eq` на `Style` (он уже `PartialEq`)
- Create: `src/tui/mod.rs`
- Create: `src/tui/app.rs`
- Create: `src/tui/effects.rs`
- Create: `src/tui/reducer.rs`
- Create: `src/tui/event.rs`
- Create: `src/tui/ui.rs`
- Create: `src/tui/save.rs`
- Create: `src/tui/sample.rs`
- Create: `src/tui/style_map.rs`
- Create: `src/tui/widget_meta.rs`
- Create: `src/tui/panels/mod.rs`
- Create: `src/tui/panels/lines.rs`
- Create: `src/tui/panels/palette.rs`
- Create: `src/tui/panels/settings.rs`
- Create: `src/tui/panels/preview.rs`
- Create: `src/tui/overlays/mod.rs`
- Create: `src/tui/overlays/themes.rs`
- Create: `src/tui/overlays/help.rs`
- Create: `src/tui/overlays/modal.rs`
- Create: `src/tui/widgets_ui/mod.rs`
- Create: `src/tui/widgets_ui/input.rs`
- Create: `src/tui/widgets_ui/color_picker.rs`
- Create: `src/tui/widgets_ui/tri_bool.rs`
- Create: `src/tui/widgets_ui/number_input.rs`
- Create: `src/tui/widgets_ui/list_editor.rs`
- Create: `src/commands/configure.rs`
- Create: `src/commands/import.rs`
- Modify: `docs/DECISIONS.md` — добавить запись D-2026-05-01

---

- [ ] **Step 1: Cargo deps + features в Cargo.toml**

Заменить блок `[dependencies]` целиком (для `serde`/`serde_json`/`dirs` сохранить как есть, добавить ratatui/crossterm/tempfile в конец `[dependencies]`, добавить `[features]` секцию перед `[dependencies]`):

```toml
[features]
default = ["tui"]
tui = ["dep:ratatui", "dep:crossterm", "dep:tempfile"]

[dependencies]
# (keep existing entries unchanged)
# ...

# Phase 8 — TUI (optional; default = ["tui"]).
# Pin minor to avoid surprise breaks; ratatui 0.30.x line is stable.
ratatui   = { version = "=0.30.0", optional = true, default-features = false, features = ["crossterm"] }
crossterm = { version = "=0.29.0", optional = true }
# Phase 8 — TUI sample payload uses NamedTempFile (transcript fixture).
tempfile  = { version = "3", optional = true }
```

В `[dev-dependencies]` строку `tempfile = "3"` оставить как есть — feature unification сделает её доступной из тестов независимо от feature flag.

- [ ] **Step 2: Создать skeleton-файлы `src/tui/*`**

Все файлы — с module docstring и lint deny. Содержимое:

`src/tui/mod.rs`:
```rust
//! TUI configurator — Phase 8.
//!
//! Все модули под `#[cfg(feature = "tui")]`. Hot path рендера (`cchud` без аргументов)
//! не зависит от этого модуля. См. `docs/superpowers/specs/2026-05-01-phase-8-tui-design.md`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod app;
pub mod effects;
pub mod event;
pub mod overlays;
pub mod panels;
pub mod reducer;
pub mod sample;
pub mod save;
pub mod style_map;
pub mod ui;
pub mod widget_meta;
pub mod widgets_ui;

pub use event::run_event_loop;

/// Public entry — вызывается из `commands::configure::run`.
/// Возвращает `Ok(true)` если user сохранил, `Ok(false)` если discard или quit без save.
pub fn run_configure(
    settings: crate::types::config::Settings,
    sample: crate::types::payload::StatusPayload,
    transcript: Option<tempfile::NamedTempFile>,
) -> std::io::Result<bool> {
    let app = app::App::new(settings, sample, transcript);
    event::run_event_loop(app)
}
```

`src/tui/app.rs`:
```rust
//! TUI App state — Phase 8 Task 6.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/effects.rs`:
```rust
//! Reducer effects — Phase 8 Task 7.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/reducer.rs`:
```rust
//! Pure reducer — Phase 8 Task 7. handle_key(&mut App, KeyEvent) -> ReducerEffect.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/event.rs`:
```rust
//! TerminalGuard + event loop — Phase 8 Task 12.
#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::tui::app::App;

pub fn run_event_loop(_app: App) -> std::io::Result<bool> {
    // Заполняется в Task 12.
    Ok(false)
}
```

`src/tui/ui.rs`:
```rust
//! Top-level draw orchestration — Phase 8 Task 10.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/save.rs`:
```rust
//! Atomic save with backup — Phase 8 Task 11.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/sample.rs`:
```rust
//! Sample payload + transcript fixture — Phase 8 Task 4.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/style_map.rs`:
```rust
//! cchud Style/Color → ratatui Style/Color/Span — Phase 8 Task 3.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/widget_meta.rs`:
```rust
//! Static widget palette registry (60 entries) — Phase 8 Task 5.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/panels/mod.rs`:
```rust
//! 4 TUI panels — Phase 8 Task 9.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod lines;
pub mod palette;
pub mod preview;
pub mod settings;
```

`src/tui/panels/{lines,palette,settings,preview}.rs` — каждый:
```rust
//! Phase 8 Task 9.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/overlays/mod.rs`:
```rust
//! Modal overlays — Phase 8 Task 10.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod help;
pub mod modal;
pub mod themes;
```

`src/tui/overlays/{help,modal,themes}.rs` — каждый:
```rust
//! Phase 8 Task 10.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/tui/widgets_ui/mod.rs`:
```rust
//! Reusable embedded widgets — Phase 8 Task 8.
#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod color_picker;
pub mod input;
pub mod list_editor;
pub mod number_input;
pub mod tri_bool;
```

`src/tui/widgets_ui/{input,color_picker,tri_bool,number_input,list_editor}.rs` — каждый:
```rust
//! Phase 8 Task 8.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/commands/configure.rs`:
```rust
//! `cchud configure` — TUI entry-point. Phase 8 Task 13.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

`src/commands/import.rs`:
```rust
//! `cchud import` — best-effort migration from ccstatusline. Phase 8 Task 13.
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

- [ ] **Step 3: Подключить модули в parent mod.rs**

В `src/lib.rs` добавить в конец:

```rust
#[cfg(feature = "tui")]
pub mod tui;
```

В `src/main.rs` после `mod widgets;` (строка 16):

```rust
#[cfg(feature = "tui")]
mod tui;
```

В `src/commands/mod.rs` (текущее содержимое — `pub mod env_loader; pub mod install;`):

```rust
#[cfg(feature = "tui")]
pub mod configure;
#[cfg(feature = "tui")]
pub mod import;
```

- [ ] **Step 4: PartialEq, Eq derive chain в `src/types/config.rs`**

Цель: `App.dirty()` использует `editable != initial` через структурное сравнение. Все типы Settings-chain должны быть `PartialEq + Eq`. Цвета (`Color`, `Style`) уже `PartialEq + Eq`.

В `src/types/config.rs` добавить `PartialEq, Eq` к derive макросам существующих структур:

- `Settings` (строка 10): `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `Line` (строка 20): `#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]`
- `WidgetItem` (строка 26): `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `WidgetStyleOverride` (строка 34): `#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]`
- `WidgetConfig` (строка 44): `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `ModelParams` (строка 152): `#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]`
- `CustomTextParams`: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `CustomSymbolParams`: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `LinkParams`: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `CustomCommandParams`: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
- `ContextBarParams`: `#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]`
- `ThemeConfig` (строка 195): `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]` — НЕТ `Default` derive (он `impl` ниже).
- `FlexMode`, `ThemeKind` уже `PartialEq, Eq` — без изменений.

В `src/render/themes.rs` (`PowerlineTheme`) — добавить `PartialEq, Eq` к существующему derive (`PowerlineTheme` = `Vec<Color>` + char поля, всё `Eq`).

В `src/render/mod.rs` строки 50–59: `Style` уже `PartialEq, Eq`. `Color` (строка 11) уже `PartialEq, Eq`. Без изменений.

Проверь: `cargo check` зелёный. Если линтеp ругается на `Eq` для float — таких полей в `Settings` нет (`compact_threshold: u32`).

- [ ] **Step 5: Verify PartialEq compiles (smoke)**

Run:
```bash
cargo check --no-default-features
cargo check
```
Expected: оба зелёные. Compile-time ошибок derive нет.

Add temporary smoke-test для derive-chain (затем удалить — это just verify, не keep):

В конец `src/types/config.rs` добавить временный test:

```rust
#[cfg(test)]
#[test]
fn settings_partial_eq_works_for_dirty_diff() {
    let a = Settings::default();
    let b = Settings::default();
    assert_eq!(a, b);
    let mut c = Settings::default();
    c.lines.push(Line::default());
    assert_ne!(a, c);
}
```

Run: `cargo test types::config::tests::settings_partial_eq_works_for_dirty_diff`
Expected: PASS.

- [ ] **Step 6: Запись DECISIONS**

В `docs/DECISIONS.md` добавить запись:

````markdown
## D-2026-05-01 — Phase 8 ключевые решения

**Контекст:** Реализация Phase 8 (релиз 0.9.0): `cchud configure` (interactive TUI) + `cchud import` (CLI миграция с ccstatusline).

**Решения:**

1. **Pure ratatui + crossterm.** Без `ratatui-interact`, без `tui-input`, без `ansi-to-tui`. Собственный `widgets_ui::input.rs` (~50 LOC) и `style_map::to_span` после refactor `compose_line` устраняют необходимость в этих хелперах.
2. **Feature flag `default = ["tui"]`.** `cargo install cchud --no-default-features` собирает минимальный бинарь (~8 MB). `commands::{configure, import}` + весь `tui/` под `#[cfg(feature = "tui")]`. Без feature `cchud configure`/`cchud import` печатают error + exit 2.
3. **`ratatui = "=0.30.0"`, `crossterm = "=0.29.0"`.** Pin patch — minor breaking changes защищены. (ratatui 0.30 тянет crossterm 0.29 как транзитивную dep; явный pin = 0.29 совместим.)
4. **`compose_line` refactor.** Pure-функция в `Renderer` возвращает `Vec<StyledSegment>` (после flex/separator/auto-align/apply_widget_style). `Renderer::render_line = compose_line + emit_ansi`. TUI live preview = `compose_line + style_map::to_span`. Single source of truth — drift между TUI preview и реальным рендером невозможен.
5. **`PartialEq` на Settings chain.** Derived на `Settings`/`Line`/`WidgetItem`/`WidgetConfig`/`WidgetStyleOverride`/`ThemeConfig`/`ModelParams`/`CustomTextParams`/`CustomSymbolParams`/`LinkParams`/`CustomCommandParams`/`ContextBarParams`/`PowerlineTheme`. `App.dirty()` = `editable != initial`. Никаких runtime-изменений. `StatusPayload` — намеренно исключён: `App.dirty()` сравнивает только `Settings`, не sample; `StatusPayload` never flows through user edits (YAGNI). Технического блокера нет — `Option<f64>` и `serde_json::Value` поддерживают `PartialEq`.
6. **Save target — `~/.config/cchud/settings.json`.** Тот же путь, что `config::load()`. Backup `<path>.bak.<unix-ts-ms>`. Atomic write `<path>.tmp` → `rename`.
7. **`cchud import` best-effort + warn.** Unknown widget type → пропуск + stderr warn. Если ноль валидных widgets — exit 1. `~/.claude/settings.json` → автодетект секции `ccstatusline`; `--from <path>` — explicit.
8. **Reducer pure (no IO).** `handle_key(&mut App, KeyEvent) -> ReducerEffect`. Save/Quit/Discard поднимаются как `ReducerEffect`, обрабатываются в event loop. ≥15 unit-тестов без `TestBackend`.
9. **`tempfile::NamedTempFile` для sample transcript.** Owned `App` через `Option<NamedTempFile>` (RAII) — fixture-файл живёт ровно столько, сколько TUI.
10. **Sample payload — захардкоженный inline.** `tui::sample::payload() -> (StatusPayload, NamedTempFile)`. Никаких runtime-аллокаций.

**Связанные документы:** `docs/superpowers/specs/2026-05-01-phase-8-tui-design.md`, `plan/phase-8-tui.md` (outline).
````

- [ ] **Step 7: Build oба варианта**

```bash
cargo build --release --locked
cargo build --release --locked --no-default-features
```

Expected: оба PASS. Skeleton-модули компилируются (только docstring + lint).

```bash
ls -la target/release/cchud
du -h target/release/cchud
```

Default build < 9.5 MB (приблизительно +1 MB на ratatui/crossterm). No-default-features build < 8.5 MB.

- [ ] **Step 8: Run existing tests (regression smoke)**

```bash
cargo test --locked
```

Expected: PASS — никаких регрессий. Existing tests не зависят от TUI.

```bash
cargo test --locked --no-default-features
```

Expected: PASS. Без TUI feature тесты `src/tui/*` исключаются.

- [ ] **Step 9: Удалить temporary smoke-test**

Удалить `settings_partial_eq_works_for_dirty_diff` из `src/types/config.rs`. PartialEq будет упражняться реальным `App.dirty()` в Task 6/7.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock \
        src/lib.rs src/main.rs src/commands/mod.rs \
        src/types/config.rs src/render/themes.rs \
        src/tui/ src/commands/configure.rs src/commands/import.rs \
        docs/DECISIONS.md
git commit -m "$(cat <<'EOF'
chore(phase-8): T1 setup — ratatui/crossterm deps under feature, skeleton tui/, PartialEq chain

- Cargo: default = ["tui"], ratatui/crossterm/tempfile pinned to =0.30/=0.30/3 (optional)
- Skeleton: src/tui/{app,effects,reducer,event,ui,save,sample,style_map,widget_meta}
- Skeleton: src/tui/{panels,overlays,widgets_ui}/* + commands/{configure,import}
- PartialEq, Eq derived on Settings/Line/WidgetItem/WidgetConfig/ThemeConfig/...
- DECISIONS D-2026-05-01: 10 architecture decisions for Phase 8

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
