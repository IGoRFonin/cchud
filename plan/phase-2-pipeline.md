# Фаза 2 — Ядро pipeline

**Длительность:** 3–5 дней
**Входные условия:** Фазы 0, 1
**Релиз:** — (но первый бинарь который реально работает в Claude Code)

## Цель

Первый рабочий бинарь с одним виджетом (`Model`), который Claude Code реально использует. End-to-end pipeline: stdin → parse → render → stdout. Hyperfine показывает < 5 мс.

## Шаги

### 2.1. Типы (на базе upstream-map из Фазы 0)

`src/types/payload.rs`:
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub session_id: String,
    pub model: ModelInfo,
    pub workspace: Workspace,
    pub transcript_path: Option<String>,
    // ... остальное по фактическому формату из payload-семплов
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub display_name: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub current_dir: String,
    pub project_dir: Option<String>,
}
```

> Точная схема — после анализа payload-семплов из Фазы 0. Не угадывать.

`src/config/types.rs` — Settings, Line, WidgetConfig:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub lines: Vec<Line>,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Line {
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WidgetConfig {
    Model { #[serde(flatten)] params: ModelParams },
    // в фазе 3 добавятся остальные
}
```

### 2.2. Widget trait и registry

`src/widgets/mod.rs`:
```rust
pub trait Widget: Send + Sync {
    fn id(&self) -> &'static str;
    fn render(&self, ctx: &RenderContext) -> Option<String>;
}

pub struct RenderContext<'a> {
    pub payload: &'a StatusPayload,
    pub settings: &'a Settings,
    git: OnceCell<Option<GitInfo>>,
    transcript: OnceCell<Option<TranscriptCache>>,
}

impl<'a> RenderContext<'a> {
    pub fn new(payload: &'a StatusPayload, settings: &'a Settings) -> Self { /* ... */ }
    pub fn git(&self) -> Option<&GitInfo> { /* lazy init */ }
    pub fn transcript(&self) -> Option<&TranscriptCache> { /* lazy init */ }
}
```

`GitInfo` и `TranscriptCache` — заглушки в Фазе 2, наполняются в фазах 5/6.

### 2.3. Plain renderer

`src/render/mod.rs`:
```rust
pub trait Renderer {
    fn render(&self, segments: &[String]) -> String;
}

pub struct Plain { pub separator: String }

impl Renderer for Plain {
    fn render(&self, segments: &[String]) -> String {
        segments.iter().filter(|s| !s.is_empty())
            .cloned().collect::<Vec<_>>().join(&self.separator)
    }
}
```

### 2.4. Конфиг: load + migrate

`src/config/mod.rs`:
```rust
pub fn load() -> Settings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<RootSettings>(&s) {
            Ok(root) => migrations::migrate(root.cchud.unwrap_or_default()),
            Err(_) => Settings::default(),  // graceful fallback (REQ AC-007)
        },
        Err(_) => Settings::default(),
    }
}

fn settings_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".claude/settings.json")
}
```

Миграции:
```rust
pub fn migrate(mut s: Settings) -> Settings {
    while s.version < CURRENT_VERSION {
        s = match s.version {
            0 => migrate_v0_to_v1(s),
            1 => migrate_v1_to_v2(s),
            _ => break,
        };
    }
    s
}
```

### 2.5. CLI args (lexopt)

`src/main.rs`:
```rust
fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => { println!(env!("CARGO_PKG_VERSION")); ExitCode::SUCCESS }
        Some("install") => commands::install::run(),
        Some("configure") => commands::configure::run(),  // заглушка → фаза 8
        Some("import") => commands::import::run(),        // заглушка → фаза 9
        Some("doctor") => commands::doctor::run(),        // заглушка → фаза 9
        _ => render(),
    }
}

fn render() -> ExitCode {
    let payload: StatusPayload = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(p) => p,
        Err(_) => { eprintln!("cchud: invalid payload"); return ExitCode::SUCCESS }
    };
    let settings = config::load();
    let ctx = RenderContext::new(&payload, &settings);
    let widgets = build_widgets(&settings);
    let segments: Vec<String> = widgets.iter()
        .filter_map(|w| w.render(&ctx))
        .collect();
    let renderer = Plain { separator: " | ".into() };
    println!("{}", renderer.render(&segments));
    ExitCode::SUCCESS
}
```

### 2.6. Виджет `Model`

`src/widgets/model.rs`:
```rust
pub struct Model;

impl Widget for Model {
    fn id(&self) -> &'static str { "Model" }
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        Some(ctx.payload.model.display_name.clone())
    }
}
```

### 2.7. `cchud install` команда

Прописывает в `~/.claude/settings.json` запуск самого себя:
```rust
pub fn run() -> ExitCode {
    let path = current_exe_path();
    let mut settings = read_claude_settings();  // serde_json::Value, чтобы не сломать чужие ключи
    settings["statusLine"] = json!({
        "type": "command",
        "command": path.to_string_lossy(),
        "padding": 0,
    });
    write_claude_settings(&settings);
    println!("cchud installed: {}", path.display());
    ExitCode::SUCCESS
}
```

### 2.8. Snapshot-тесты с реальными payload'ами

`tests/snapshots.rs`:
```rust
#[test]
fn render_model_for_samples() {
    insta::glob!("../benches/samples", "payload-*.json", |path| {
        let payload = std::fs::read_to_string(path).unwrap();
        let output = Command::cargo_bin("cchud").unwrap()
            .write_stdin(payload).output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        insta::assert_snapshot!(stdout);
    });
}
```

`cargo insta review` для подтверждения первичных снапшотов.

### 2.9. Бенч-сравнение

```bash
hyperfine --warmup 20 --runs 200 \
  "cat benches/samples/payload-001.json | ./target/release/cchud" \
  "cat benches/samples/payload-001.json | ccstatusline"
```

**Ожидание:** Rust-версия в 10–30× быстрее (1–5 мс vs 50–150 мс).

### 2.10. Реальный тест в Claude Code

```bash
./target/release/cchud install
```

Запустить Claude Code, увидеть `Model` в строке. Поработать 5 минут. Убедиться, что нет лагов, нет крашей.

После теста — вернуть ccstatusline (или оставить cchud, как удобно):
```bash
# вручную поправить ~/.claude/settings.json statusLine.command
```

## Exit Criteria

- [ ] Виджет `Model` рендерится в реальном Claude Code
- [ ] `hyperfine` показывает p95 < 5 мс на M-серии
- [ ] `cargo test` проходит, snapshot'ы зафиксированы
- [ ] `cchud --version` работает
- [ ] `cchud install` корректно правит `~/.claude/settings.json` (не теряет другие ключи)
- [ ] Поломанный JSON в payload → cchud не падает, печатает дефолт (AC-007)
- [ ] Первый push с тэгом `phase-2-pipeline` (но не релиз)

## Связи

- **Фаза 3** добавляет 9 виджетов в этот же pipeline
- **Фаза 4** меняет `Plain` на `Powerline`
- **Фаза 6** наполняет `transcript()` lazy-initializer
- **Фаза 8** наполняет `cchud configure` команду
- **Фаза 9** наполняет `cchud import` и `cchud doctor`

## Риски

- **Точная схема payload может отличаться** от ожидаемой. Решение: `#[serde(default)]` + `Option<T>` на всё, что не критично; виджет читает только нужные поля.
- **Lazy git/transcript через `OnceCell`** должен быть `Send + Sync` если в будущем добавим параллелизм. Для одного потока — `Cell<Option<...>>` достаточно.
- **`cchud install` может затереть чужой `statusLine`** — обязательно сначала читать как `serde_json::Value`, менять только нужное поле, писать обратно с `to_string_pretty`.
- **Производительность хуже ожидаемой** на macOS из-за codesign — `xattr -d com.apple.quarantine` после первого запуска.
