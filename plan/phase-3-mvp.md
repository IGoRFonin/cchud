# Фаза 3 — MVP виджеты

**Длительность:** 1 неделя
**Входные условия:** Фазы 0–2
**Релиз:** **0.1.0-alpha**

## Цель

Топ-10 виджетов, покрывающих ~80% реальных конфигов. Бинарь годен для базового использования. Подключаем `gix`, `sonic-rs`, `bincode` (через минимальные интерфейсы). Стартовый JSONL-кэш.

## Топ-10 виджетов

Приоритет по частоте упоминания в README/issues ccstatusline и реальных конфигах:

| # | Виджет | Источник | Сложность | Зависимости |
|---|---|---|---|---|
| 1 | `Model` | payload | low | — (готов в Фазе 2) |
| 2 | `ContextLength` | payload | low | — |
| 3 | `ContextPercentage` | payload + utils | mid | таблица модель→max-tokens |
| 4 | `CurrentWorkingDir` | payload/env | low | — |
| 5 | `GitBranch` | gix | mid | gix |
| 6 | `GitStatus` | gix status | mid | gix |
| 7 | `SessionCost` | transcript jsonl | high | sonic-rs, JSONL-кэш |
| 8 | `CustomText` | settings | low | — |
| 9 | `CustomSymbol` | settings | low | — |
| 10 | `TerminalWidth` | terminal_size | low | — |

## Шаги

### 3.1. Cargo deps (расширение)

```toml
sonic-rs = "0.3"
gix = { version = "0.81", default-features = false, features = ["max-performance-safe", "revision", "zlib-rs", "status", "sha1"] }
bincode = "2"
once_cell = "1"
```

### 3.2. Простые виджеты (1–4, 8, 9, 10)

`src/widgets/context_length.rs`, `context_percentage.rs`, `cwd.rs`, `custom_text.rs`, `custom_symbol.rs`, `terminal_width.rs`.

`ContextPercentage` нужна таблица макс-токенов на модель — порт из upstream `utils/context-window.ts`. Хранить как `static MODEL_LIMITS: phf::Map<&str, u32>` (через `phf` крейт, опционально) или `match` по prefix (`claude-3-5-sonnet*` → 200000, и т.д.).

`TerminalWidth` через `terminal_size::terminal_size()`.

### 3.3. Git infrastructure

`src/git/mod.rs`:
```rust
pub struct GitInfo {
    repo: gix::Repository,
    pub head_branch: Option<String>,
    pub head_sha: Option<String>,
    pub status: GitStatus,  // counts: staged, unstaged, untracked, conflicts
    pub remotes: HashMap<String, RemoteInfo>,
}

impl GitInfo {
    pub fn open(dir: &Path) -> Option<Self> {
        let repo = gix::discover(dir).ok()?;
        let head_branch = repo.head_name().ok().flatten().map(...);
        // Остальные поля — lazy внутри по запросу, но в фазе 3 минимум
        Some(Self { repo, head_branch, ... })
    }
}
```

`RenderContext::git()` инициализирует один раз, переиспользуется всеми git-виджетами.

### 3.4. Виджеты `GitBranch` и `GitStatus`

`src/widgets/git_branch.rs`:
```rust
impl Widget for GitBranch {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        ctx.git()?.head_branch.clone()
    }
}
```

`src/widgets/git_status.rs` — компактная строка `+1 ~2 -3 ?4` (insertions/staged/deletions/untracked).

### 3.5. JSONL-кэш (минимальный, для SessionCost)

`src/cache/jsonl.rs`:
```rust
#[derive(Serialize, Deserialize, Default)]
pub struct TranscriptStats {
    pub cost_total: f64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub messages: u32,
}

pub fn load_or_build(transcript_path: &Path) -> Option<TranscriptStats> {
    let cache_path = cache_path_for(transcript_path);
    if let Some(cached) = try_load_cache(&cache_path, transcript_path) {
        return Some(cached);
    }
    let stats = parse_transcript(transcript_path)?;
    write_cache(&cache_path, &stats);
    Some(stats)
}

fn try_load_cache(cache: &Path, src: &Path) -> Option<TranscriptStats> {
    let cache_meta = std::fs::metadata(cache).ok()?;
    let src_meta = std::fs::metadata(src).ok()?;
    if cache_meta.modified().ok()? >= src_meta.modified().ok()? {
        let bytes = std::fs::read(cache).ok()?;
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).ok().map(|(v, _)| v)
    } else {
        None
    }
}

fn parse_transcript(path: &Path) -> Option<TranscriptStats> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut stats = TranscriptStats::default();
    for line in reader.lines().flatten() {
        // sonic_rs::from_str для скорости
        if let Ok(entry) = sonic_rs::from_str::<TranscriptEntry>(&line) {
            stats.cost_total += entry.cost.unwrap_or(0.0);
            // ...
        }
    }
    Some(stats)
}
```

> В Фазе 3 — минимальные поля (`cost_total`, `tokens_*`). Полная агрегация (blocks, weekly, speed) — в Фазе 6.

### 3.6. Виджет `SessionCost`

```rust
impl Widget for SessionCost {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let stats = ctx.transcript()?;
        Some(format!("${:.2}", stats.cost_total))
    }
}
```

### 3.7. Полная регистрация виджетов

`src/widgets/registry.rs`:
```rust
pub fn build_widgets(settings: &Settings) -> Vec<Box<dyn Widget>> {
    settings.lines.iter().flat_map(|line| {
        line.widgets.iter().map(|cfg| match cfg {
            WidgetConfig::Model { .. } => Box::new(Model) as Box<dyn Widget>,
            WidgetConfig::ContextLength { .. } => Box::new(ContextLength),
            WidgetConfig::ContextPercentage { .. } => Box::new(ContextPercentage),
            WidgetConfig::CurrentWorkingDir { .. } => Box::new(Cwd),
            WidgetConfig::GitBranch { .. } => Box::new(GitBranch),
            WidgetConfig::GitStatus { .. } => Box::new(GitStatus),
            WidgetConfig::SessionCost { .. } => Box::new(SessionCost),
            WidgetConfig::CustomText { value } => Box::new(CustomText(value.clone())),
            WidgetConfig::CustomSymbol { value } => Box::new(CustomSymbol(value.clone())),
            WidgetConfig::TerminalWidth { .. } => Box::new(TerminalWidth),
        })
    }).collect()
}
```

### 3.8. Тесты

- Unit-тесты на каждый виджет: `tests/widgets/<name>.rs`.
- Snapshot-тесты с конфигом, использующим все 10 виджетов.
- Performance-тест: `bench` с конфигом из 10 виджетов → < 5 мс p95.

### 3.9. README обновление

Таблица "supported widgets" с галочками. Базовый install:
```bash
git clone https://github.com/igorfonin/cchud
cd cchud
cargo build --release
./target/release/cchud install
```

### 3.10. Релиз 0.1.0-alpha

```bash
git tag v0.1.0-alpha
git push --tags
```

CI собирает GH Release с одним бинарём (хост-платформа). Полный multi-platform release — в Фазе 9.

## Exit Criteria

- [ ] 10 виджетов работают, покрыты unit-тестами
- [ ] Конфиг с 10 виджетами рендерится за < 5 мс p95
- [ ] JSONL-кэш работает: первый запуск медленный, второй (с кэшем) быстрый
- [ ] Snapshot-тесты на ≥ 5 разных конфигах (минимум, средний, с git, без git, с транскриптом)
- [ ] Релиз 0.1.0-alpha опубликован на GitHub
- [ ] README с install-инструкцией и живой строкой статуса

## Связи

- **Фаза 4** добавляет Powerline-рендерер для тех же 10 виджетов
- **Фаза 5** расширяет `GitInfo` — добавляются ahead/behind, conflicts, remotes, worktrees
- **Фаза 6** расширяет `TranscriptStats` — blocks, weekly, speed, tokens-per-message
- **Фаза 7** добавляет оставшиеся ~50 виджетов

## Риски

- **`gix` discovery медленный** на больших монорепо — измерить, при необходимости делать в фоне (но для статической строки — не оправдано). Кэшировать root в env-переменной.
- **`sonic-rs` падает на не-UTF-8** в транскрипте — обернуть в graceful fallback на `serde_json`.
- **Bincode формат меняется между версиями** — добавить magic-байты + version в начало кэша; при mismatch — пересобрать.
- **JSONL-кэш стал stale** в момент чтения — игнорить, в худшем случае одна устаревшая строка статуса; в следующий рендер пересоберётся.
