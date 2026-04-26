# Фаза 7 — Остальные виджеты (паритет 60+)

**Длительность:** 2–3 дня
**Входные условия:** Фазы 0–6
**Релиз:** **0.5.0** — паритет 60+ виджетов с ccstatusline 2.2.8

## Цель

Закрыть последние ~15 виджетов, которые не попали в фазы 3/5/6. Это разнородная группа: системные, env, custom commands, decorative.

## Виджеты

| Виджет | Источник | Сложность | Notes |
|---|---|---|---|
| `FreeMemory` | sysinfo | low | crate `sysinfo` |
| `Version` | env | low | `env!("CARGO_PKG_VERSION")` |
| `OutputStyle` | payload | low | extract из payload |
| `VimMode` | payload | low | extract из payload |
| `Skills` | payload/config | mid | список skills из payload |
| `Link` | static | low | OSC 8 hyperlink wrapper |
| `CustomCommand` | spawn | mid | timeout 200 мс |
| `ContextBar` | payload | mid | визуальный progress bar |

Возможные дополнительные (если не покрыты фазой 3/6):
- `ClaudeAccountEmail`, `ClaudeSessionId`, `ThinkingEffort` — из payload, если ещё не в фазе 6

## Шаги

### 7.1. System / env виджеты

`Cargo.toml`:
```toml
sysinfo = { version = "0.32", default-features = false, features = ["system"] }
```

`src/widgets/free_memory.rs`:
```rust
impl Widget for FreeMemory {
    fn render(&self, _ctx: &RenderContext) -> Option<String> {
        let mut sys = System::new();
        sys.refresh_memory();
        let free_mb = sys.available_memory() / 1024 / 1024;
        Some(format!("💾 {} MB", free_mb))
    }
}
```

> `sysinfo::refresh_memory()` — самая дешёвая операция, не делает full refresh всех CPU/processes.

`Version` тривиально: `env!("CARGO_PKG_VERSION")`.

### 7.2. CustomCommand с timeout

`Cargo.toml`:
```toml
process_control = "5"
```

`src/widgets/custom_command.rs`:
```rust
pub struct CustomCommand {
    pub command: String,
    pub timeout_ms: u64,
}

impl Widget for CustomCommand {
    fn render(&self, _ctx: &RenderContext) -> Option<String> {
        use process_control::{ChildExt, Control};
        let mut child = Command::new("sh")
            .args(["-c", &self.command])
            .stdout(Stdio::piped())
            .spawn().ok()?;
        let output = child
            .controlled_with_output()
            .time_limit(Duration::from_millis(self.timeout_ms))
            .terminate_for_timeout()
            .wait().ok()??;
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(stdout.trim().to_string())
    }
}
```

Дефолтный timeout: 200 мс. Конфигурируется в settings.

### 7.3. Skills виджет

Skills передаются в payload Claude Code:
```rust
impl Widget for Skills {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let skills = ctx.payload.skills.as_ref()?;
        if skills.is_empty() { return None; }
        Some(format!("🎯 {} skills", skills.len()))
    }
}
```

Опциональные параметры конфига: показывать только активные skills, ограничение по количеству.

### 7.4. Link виджет

```rust
pub struct Link {
    pub text: String,
    pub url: String,
}

impl Widget for Link {
    fn render(&self, _ctx: &RenderContext) -> Option<String> {
        Some(crate::render::hyperlink::link(&self.text, &self.url, supports_hyperlinks()))
    }
}
```

### 7.5. ContextBar — визуальный

```rust
impl Widget for ContextBar {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let used = ctx.payload.context_used?;
        let total = ctx.payload.context_total?;
        let ratio = used as f64 / total as f64;
        let width = self.width.unwrap_or(10);
        let filled = (ratio * width as f64) as usize;
        Some(format!("[{}{}]", "█".repeat(filled), " ".repeat(width - filled)))
    }
}
```

### 7.6. VimMode и OutputStyle

Простые getter'ы из payload — порт upstream `widgets/VimMode.ts`, `OutputStyle.ts`.

### 7.7. Полная регистрация

`WidgetConfig` enum достигает ~60 вариантов. Это **много, но управляемо** — серде-derive справляется.

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WidgetConfig {
    Model(ModelParams),
    ContextLength(NoParams),
    ContextPercentage(NoParams),
    // ... все 60+
}
```

> Альтернатива: data-driven подход с реестром `HashMap<&str, fn(WidgetConfig) -> Box<dyn Widget>>` — проще для динамической загрузки в TUI, но для 1.0 enum достаточно.

### 7.8. Документация виджетов

Сгенерировать `docs/widgets.md` через скрипт, который пробегает по реестру и печатает таблицу:

| Name | Description | Source | Config params |
|---|---|---|---|
| Model | Display model name | payload | format |
| ... |

### 7.9. Полный snapshot-набор

Конфиг `tests/configs/full.json` использует **все 60+ виджетов** в одной строке. Snapshot-тест на нём.

```rust
#[test]
fn full_widget_set_renders() {
    let config = include_str!("configs/full.json");
    let payload = include_str!("../benches/samples/payload-rich.json");
    let output = run_with(config, payload);
    insta::assert_snapshot!(output);
}
```

### 7.10. Релиз 0.5.0 — заявка на паритет

CHANGELOG: "Feature parity with ccstatusline 2.2.8 (60+ widgets)". README обновлён с полной таблицей виджетов.

## Exit Criteria

- [ ] Все ~60 виджетов реализованы и зарегистрированы
- [ ] `docs/widgets.md` автоматически сгенерирован, актуален
- [ ] Snapshot-тест с полным набором виджетов проходит
- [ ] `CustomCommand` с timeout не блокирует cchud при зависшем shell-скрипте
- [ ] `cchud import` (заглушка из Фазы 2 → реальная реализация) понимает все типы виджетов из ccstatusline-конфига
- [ ] Hyperfine: < 5 мс p95 на типичном конфиге (8–15 виджетов), < 12 мс на full-set (60+)
- [ ] Релиз **0.5.0** на GitHub Releases (single-platform пока, multi-platform в Фазе 9)

## Связи

- **Фаза 8** TUI-конфигуратор использует полный реестр виджетов как палитру
- **Фаза 9** релиз 1.0.0 (после TUI и дистрибуции)

## Риски

- **`sysinfo` slow init** на Linux — `new_all()` делает много syscall'ов. Использовать `new()` + `refresh_memory()` точечно.
- **CustomCommand с tail -f** или другим long-running — `process_control` корректно убивает по timeout. Тестировать.
- **Полный набор 60+ виджетов в одной строке** — нереалистичный кейс, но snapshot-тест ловит регрессии.
- **Skills-payload format меняется** — defensive parsing, `Option<Vec<String>>`.
- **Backwards compat ccstatusline import** — фикстуры из реальных конфигов upstream-юзеров.
