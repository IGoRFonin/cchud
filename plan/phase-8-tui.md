# Фаза 8 — TUI-конфигуратор

**Длительность:** 1–2 недели
**Входные условия:** Фазы 0–7
**Релиз:** 0.9.0

## Цель

Подкоманда `cchud configure` — interactive TUI на ratatui+crossterm, зеркалит ccstatusline TUI: Lines panel, Widget palette, Settings panel, Live preview. Новый пользователь может собрать конфиг с нуля без правки JSON.

## Шаги

### 8.1. Cargo deps

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.30"
# опционально, оценить в первые 2 дня:
# ratatui-interact = "0.5"

[features]
default = ["tui"]
tui = ["dep:ratatui", "dep:crossterm"]
```

Feature-flag позволяет собрать минимальный бинарь без TUI для самых нагруженных юзеров.

### 8.2. Архитектура App

`src/tui/app.rs`:
```rust
pub struct App {
    pub settings: Settings,
    pub mode: Mode,
    pub focus: Pane,
    pub selected_line: usize,
    pub selected_widget: usize,
    pub palette_filter: String,
    pub preview_payload: StatusPayload,  // дефолтный sample
    pub dirty: bool,
}

pub enum Mode { Edit, AddWidget, EditWidget, Themes, Save, Quit }
pub enum Pane { Lines, Palette, Settings, Preview }
```

### 8.3. Главные экраны

#### 8.3.1. Lines panel (top-left)
Список линий статусбара (обычно 1-2). Можно add/delete. Каждая линия — список виджетов в порядке.

```
┌─ Lines ────────────────┐
│ ▶ Line 1 (8 widgets)   │
│   Line 2 (3 widgets)   │
│ + Add line             │
└────────────────────────┘
```

#### 8.3.2. Widget palette (top-right)
Список 60+ виджетов с фильтром (input). Enter — добавить в текущую линию.

```
┌─ Widgets ────[/git]────┐
│ ▶ GitBranch            │
│   GitStatus            │
│   GitChanges           │
│   GitPr                │
└────────────────────────┘
```

#### 8.3.3. Widget settings (bottom-left)
Параметры выбранного виджета (цвет, формат, custom-параметры).

```
┌─ Widget: GitBranch ─────┐
│ Color FG: [Cyan      ▾] │
│ Color BG: [None      ▾] │
│ Bold:     [✓]           │
│ Format:   [{branch}]    │
└─────────────────────────┘
```

#### 8.3.4. Live preview (bottom-right)
Рендерит текущий конфиг прямо на дефолтном payload-семпле.

```
┌─ Preview ───────────────┐
│ ▶ Sonnet 4.7  ⌘ 45%     │
│   git:main +2  $1.23    │
└─────────────────────────┘
```

### 8.4. Event loop

`src/tui/main.rs`:
```rust
pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    execute!(stdout(), EnterAlternateScreen)?;

    let mut app = App::new(load_settings()?, sample_payload());
    while app.mode != Mode::Quit {
        terminal.draw(|f| ui::draw(f, &app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key)?;
            }
        }
    }
    if app.dirty && app.mode == Mode::Save {
        save_settings(&app.settings)?;
    }
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
```

### 8.5. Keybindings

Стандартные:
- `Tab` / `Shift+Tab` — переключение панелей
- `↑↓←→` или `hjkl` — навигация в списке
- `Enter` — выбрать / добавить
- `Delete` / `d` — удалить
- `e` — edit settings виджета
- `t` — Themes
- `s` — Save
- `q` / `Ctrl+C` — quit (с подтверждением если dirty)
- `/` — фильтр в palette

### 8.6. Решение по фреймворку

**Старт с pure ratatui.** Если в первые 2 дня боль с focus management/click — пробуем `ratatui-interact`. Не тащить заранее.

```rust
// pure ratatui — сами трекаем focus
let block = Block::default().borders(Borders::ALL)
    .border_style(if app.focus == Pane::Lines { Style::default().fg(Color::Yellow) } else { Style::default() })
    .title("Lines");
```

### 8.7. Themes screen

Список встроенных тем (фаза 4). Превью.

### 8.8. Migration screen — `cchud import`

Не часть TUI per se, но reuse render-логики. Может быть отдельной командой:
```bash
cchud import --from ~/.claude/settings.json.bak
# или
cchud import   # автодетект ccstatusline-секции в settings.json
```

После импорта — сразу `cchud configure` для тонкой настройки.

### 8.9. Save с подтверждением

```rust
if app.dirty {
    show_modal("Save changes? [y/n/c]");
}
```

Backup старого settings.json в `~/.claude/settings.json.bak.<timestamp>`.

### 8.10. Тесты

TUI-тесты сложно snapshot'ить, но можно:
- Unit-тесты на reducer'ы (handle_key)
- Snapshot ratatui-buffer'а (`ratatui::backend::TestBackend`):
```rust
#[test]
fn render_initial_state() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new(default_settings(), sample_payload());
    terminal.draw(|f| ui::draw(f, &app)).unwrap();
    insta::assert_debug_snapshot!(terminal.backend().buffer());
}
```

### 8.11. Релиз 0.9.0

CHANGELOG: TUI configurator. README обновлён с GIF/asciinema TUI.

## Exit Criteria

- [ ] `cchud configure` запускается, открывает 4-панельный TUI
- [ ] Можно add/remove/reorder виджетов в линиях
- [ ] Можно менять параметры виджетов (цвет, формат)
- [ ] Live preview обновляется при каждом изменении
- [ ] Save → settings.json корректно записан, бэкап создан
- [ ] `cchud import` работает с реальным ccstatusline settings.json
- [ ] Все клавиши задокументированы в `?` help-overlay
- [ ] Feature `tui = false` — бинарь собирается на 30%+ меньше

## Связи

- **Фаза 9** добавляет TUI в multi-platform release (Windows тестируется отдельно — Crossterm там специфический)

## Риски

- **TUI на Windows ConPTY** — отдельные баги crossterm. Тестировать на Windows Terminal + cmd + powershell.
- **ratatui breaking changes между minor** — pin к 0.30.x, smoke-тест.
- **Слишком долго делать UX** — feature-freeze TUI на 2 недели жёсткий cap. Если упираемся — **выпустить 1.0 без TUI** (TUI в 1.1).
- **60+ виджетов в палитре** — нужен фильтр, скролл, иначе UX тяжёлый.
- **Live preview лагает** на больших транскриптах — debounce 100 мс.
