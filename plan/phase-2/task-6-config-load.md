# Task 6 — Config loader + main wiring

**Files:**
- Create: `src/config/mod.rs`
- Modify: `src/main.rs` (использовать `config::load`, передать `&Settings` в `RenderContext`, использовать `default_line` для пустого/битого config)
- Modify: `src/widgets/mod.rs` (`RenderContext::new(payload, settings)`, `build_widgets(&Settings)`)
- Modify: `src/widgets/model.rs` (тестовый helper больше не нужен — обновить unit-тесты под новую signature `RenderContext::new`)

## Goal

Загрузить пользовательский конфиг из `~/.claude/settings.json`, обработать все error-пути в `Settings::default_line` (AC-007 distill: graceful, никаких panic). Прокинуть `&Settings` через `RenderContext`, сделать `build_widgets` config-driven (читает `settings.lines[0].widgets`). По-прежнему default-line с одним `Model` если конфига нет.

## Inputs

- Tasks 1–5 закрыты: walking skeleton работает, config types определены.
- `dirs = "6"` уже в `[dependencies]` (Phase 1).
- Walking skeleton использует hardcoded `[Box::new(Model)]` — Task 6 заменяет на config-driven.

---

- [ ] **Step 1: Создать `src/config/mod.rs`**

```rust
//! Config loader — reads `~/.claude/settings.json`, extracts the `cchud`
//! block, gracefully falls back to default-line on any error (AC-007 spirit).
//!
//! Never panics. Never propagates errors to caller — caller gets `Settings`.
//! Diagnostic warnings go to stderr with `cchud:` prefix.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use serde::Deserialize;

use crate::types::config::{Line, ModelParams, Settings, ThemeConfig, WidgetConfig};

#[derive(Deserialize)]
struct RootSettings {
    #[serde(default)]
    cchud: Option<Settings>,
}

#[must_use]
pub fn load() -> Settings {
    let path = settings_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return default_line();
    };
    match serde_json::from_str::<RootSettings>(&content) {
        Ok(root) => match root.cchud {
            Some(s) => s,
            None => default_line(),
        },
        Err(e) => {
            eprintln!("cchud: invalid cchud config block, using defaults: {e}");
            default_line()
        }
    }
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/settings.json")
}

#[must_use]
pub fn default_line() -> Settings {
    Settings {
        version: 1,
        lines: vec![Line {
            widgets: vec![WidgetConfig::Model {
                params: ModelParams::default(),
            }],
        }],
        theme: ThemeConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn default_line_has_one_model_widget() {
        let s = default_line();
        assert_eq!(s.lines.len(), 1);
        assert_eq!(s.lines[0].widgets.len(), 1);
        assert!(matches!(s.lines[0].widgets[0], WidgetConfig::Model { .. }));
    }

    #[test]
    fn default_line_version_is_one() {
        assert_eq!(default_line().version, 1);
    }

    #[test]
    fn root_settings_parses_with_cchud_block() {
        let json = r#"{"cchud":{"version":1,"lines":[{"widgets":[{"type":"Model"}]}]}}"#;
        let root: RootSettings = serde_json::from_str(json).unwrap();
        assert!(root.cchud.is_some());
        let cchud = root.cchud.unwrap();
        assert_eq!(cchud.lines.len(), 1);
    }

    #[test]
    fn root_settings_parses_without_cchud_block() {
        let json = r#"{"theme":"dark","mcpServers":{}}"#;
        let root: RootSettings = serde_json::from_str(json).unwrap();
        assert!(root.cchud.is_none());
    }
}
```

**Note про тесты:** мы покрыли `default_line` shape + `RootSettings` парсинг. Реальный `load()` зависит от FS — интеграционные тесты с tempdir+HOME-override написать сложно (они конфликтуют с install-тестами). Пропустим — `load()` тривиален и покрывается smoke-тестом в Step 5.

- [ ] **Step 2: Подключить `mod config;` в `src/main.rs`**

Найти `mod render;` строку и добавить выше:
```rust
mod config;
mod render;
mod types;
mod widgets;
```

- [ ] **Step 3: Обновить `src/widgets/mod.rs` — пробросить `&Settings`**

Заменить определение `RenderContext` и `build_widgets`:

```rust
//! Widget trait, render context, and registry/factory for widgets.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod model;

use crate::types::{
    config::{Settings, WidgetConfig},
    payload::StatusPayload,
};

pub trait Widget: Send + Sync {
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String>;
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    pub settings: &'a Settings,
    // Phase 5/6 will add: git, transcript (OnceCell)
}

impl<'a> RenderContext<'a> {
    #[must_use]
    pub const fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self {
        Self { payload, settings }
    }
}

#[must_use]
pub fn build_widgets(settings: &Settings) -> Vec<Box<dyn Widget>> {
    settings
        .lines
        .first()
        .map(|line| line.widgets.iter().map(build_one).collect())
        .unwrap_or_default()
}

fn build_one(cfg: &WidgetConfig) -> Box<dyn Widget> {
    match cfg {
        WidgetConfig::Model { .. } => Box::new(model::Model),
    }
}
```

- [ ] **Step 4: Обновить unit-тесты в `src/widgets/model.rs`**

Найти test helper `fn payload_with_display_name(...)` — он работает с `RenderContext::new(&p)`, теперь нужен `&Settings`. Заменить тесты на:

```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_display_name(name: &str) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: name.into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
        }
    }

    #[test]
    fn renders_display_name() {
        let p = payload_with_display_name("Sonnet 4.6");
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(Model.render(&ctx), Some("Sonnet 4.6".into()));
    }

    #[test]
    fn returns_none_for_empty_name() {
        let p = payload_with_display_name("");
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(Model.render(&ctx), None);
    }
}
```

- [ ] **Step 5: Обновить `src/main.rs::render_pipeline` под `config::load`**

```rust
fn render_pipeline() -> ExitCode {
    let payload: StatusPayload = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: invalid payload: {e}");
            return ExitCode::SUCCESS; // AC-007
        }
    };
    let settings = config::load();
    let ctx = RenderContext::new(&payload, &settings);
    let widgets = build_widgets(&settings);
    let segments: Vec<String> = widgets.iter().filter_map(|w| w.render(&ctx)).collect();
    let renderer = Plain { separator: " | ".into() };
    println!("{}", renderer.render(&segments));
    ExitCode::SUCCESS
}
```

- [ ] **Step 6: Smoke test — config-driven**

```bash
cargo build --release --locked

# Кейс 1: нет конфиг-файла → default_line с Model
HOME=/tmp/cchud-test-empty mkdir -p /tmp/cchud-test-empty/.claude
HOME=/tmp/cchud-test-empty cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud
# Expected: "Sonnet 4.6"

# Кейс 2: settings.json без cchud-блока → default_line
HOME=/tmp/cchud-test-other mkdir -p /tmp/cchud-test-other/.claude
echo '{"theme":"dark"}' > /tmp/cchud-test-other/.claude/settings.json
HOME=/tmp/cchud-test-other cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud
# Expected: "Sonnet 4.6"

# Кейс 3: битый JSON в settings.json → default_line + warning в stderr
HOME=/tmp/cchud-test-broken mkdir -p /tmp/cchud-test-broken/.claude
echo '{"cchud": {{not valid' > /tmp/cchud-test-broken/.claude/settings.json
HOME=/tmp/cchud-test-broken cat benches/samples/payload-cchud-sonnet-xlarge.json | ./target/release/cchud 2>&1
# Expected: "Sonnet 4.6" в stdout + "cchud: invalid cchud config block..." в stderr

# Cleanup
rm -rf /tmp/cchud-test-empty /tmp/cchud-test-other /tmp/cchud-test-broken
```

Все три кейса должны давать одинаковый stdout `Sonnet 4.6`. Кейс 3 дополнительно даёт warning на stderr.

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Тесты:
- 4 новых в `config::tests::*`
- 2 обновлённых в `widgets::model::tests::*`
- остальные без изменений

- [ ] **Step 8: Verification — task-specific gate**

```bash
test -f src/config/mod.rs && echo "config/mod.rs ok"
grep -q 'pub fn load' src/config/mod.rs && echo "load fn ok"
grep -q 'pub fn default_line' src/config/mod.rs && echo "default_line fn ok"
grep -q 'mod config;' src/main.rs && echo "main wired ok"
grep -q 'pub fn build_widgets(settings: &Settings)' src/widgets/mod.rs && echo "build_widgets signature ok"
grep -q 'config::load' src/main.rs && echo "main uses config::load ok"
cargo test --locked config:: 2>&1 | grep -c "test result: ok"
```

Expected: все строки `ok` + последняя `1`.

- [ ] **Step 9: Commit**

```bash
git add src/config/mod.rs src/main.rs src/widgets/mod.rs src/widgets/model.rs
git commit -m "feat(phase-2): config loader + RenderContext settings wiring

config::load reads ~/.claude/settings.json, returns Settings.
All error paths fall through to default_line() (AC-007 spirit):
- missing file
- invalid root JSON
- valid root but no cchud block
- valid root, cchud block but invalid → warning on stderr

build_widgets now reads settings.lines[0].widgets — config-driven
with hardcoded default-line as fallback. RenderContext gains
&Settings; model widget tests updated for new constructor signature.

4 new unit tests in config::tests covering default_line and
RootSettings parsing.

Task 6/10 of Phase 2.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/config/mod.rs` создан, экспортирует `pub fn load`, `pub fn default_line`
- [ ] Все error-ветки в `load()` ведут к `default_line()` без panic
- [ ] Битый `cchud` блок печатает warning `cchud: invalid cchud config block, using defaults: ...` на stderr
- [ ] `RenderContext` теперь имеет поле `settings: &'a Settings`
- [ ] `build_widgets(&Settings)` читает `settings.lines[0].widgets`
- [ ] `WidgetConfig::Model` мапится в `Box::new(Model)` через `build_one`
- [ ] `src/widgets/model.rs` unit-тесты обновлены под новый `RenderContext::new(payload, settings)`
- [ ] `src/main.rs` использует `config::load()` и `build_widgets(&settings)`
- [ ] Smoke test (Step 6) проходит все 3 кейса
- [ ] 4 новых config-теста + 2 обновлённых model-теста зелёные
- [ ] Один коммит `feat(phase-2): config loader + RenderContext settings wiring`

## Files touched

- `src/config/mod.rs` (created)
- `src/main.rs` (modified — `mod config`, `config::load`, обновлённый pipeline)
- `src/widgets/mod.rs` (modified — `RenderContext::new(p, s)`, `build_widgets(&Settings)`, `build_one`)
- `src/widgets/model.rs` (modified — обновлённые unit-тесты)

## Risks & rollback

- **Smoke test cleanup**: если skript упал между mkdir и cleanup, остаются temp директории в `/tmp`. Скрипт явно делает `rm -rf` в конце; при ручном прерывании выполни вручную.
- **`HOME` override на macOS не подменяет `dirs::home_dir()` в редких случаях**: на macOS `dirs::home_dir()` использует `HOME` env (как и на Linux). Если падает — это инфраструктурная проблема, не кода.
- **Clippy `module_name_repetitions` для `RootSettings`**: добавить `#[allow(clippy::module_name_repetitions)]` локально на структуру.
- **Test `widgets::model::tests::renders_display_name` не находит `crate::config::default_line`**: убедись что `mod config` подключён в `main.rs` ВЫШЕ `mod widgets`. Порядок в Rust 2024 edition не критичен для resolve, но логически опрятнее.
- **Rollback**: `git revert HEAD` снимает все изменения; смотри что `widgets/mod.rs` и `widgets/model.rs` вернутся к Task 4 версии (RenderContext без settings).
