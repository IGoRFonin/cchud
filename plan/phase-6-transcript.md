# Фаза 6 — Транскрипт и метрики

**Длительность:** 4–6 дней
**Входные условия:** Фазы 0–3 (фаза 4/5 параллельны)
**Релиз:** 0.4.0

## Цель

Полная агрегация transcript.jsonl: cost, tokens, скорости, блок-таймеры, weekly. JSONL-кэш на диске с инвалидацией. Все виджеты, читающие транскрипт.

## Виджеты на транскрипте

| Виджет | Что считает |
|---|---|
| `BlockTimer` | время до конца текущего 5-часового блока |
| `BlockResetTimer` | то же, формат `HH:MM` |
| `WeeklyResetTimer` | до конца недельного окна |
| `WeeklyUsage` | $ за неделю |
| `SessionClock` | длительность текущей сессии |
| `SessionUsage` | $ за сессию (готов частично) |
| `SessionCost` | $ total (готов в Фазе 3) |
| `SessionName` | имя сессии |
| `TokensOutput` | output tokens сессии |
| `TokensTotal` | total tokens сессии |
| `InputSpeed` | input tokens/сек последнего сообщения |
| `OutputSpeed` | output tokens/сек последнего сообщения |
| `TotalSpeed` | total tokens/сек |
| `ContextPercentageUsable` | % с учётом ratio |
| `ContextBar` | визуальный бар |
| `ThinkingEffort` | extracted из payload или transcript |
| `ClaudeAccountEmail` | extracted |
| `ClaudeSessionId` | extracted |
| `SessionUsage` | API usage |

## Шаги

### 6.1. Расширение TranscriptStats

`src/cache/jsonl_types.rs`:
```rust
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TranscriptStats {
    // session-wide
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub session_started_at: Option<u64>,  // unix timestamp
    pub messages: u32,

    pub cost_total: f64,
    pub tokens_in_total: u64,
    pub tokens_out_total: u64,

    pub last_message: Option<MessageStats>,

    // billing blocks (5-hour windows)
    pub blocks: Vec<BillingBlock>,

    // weekly window
    pub weekly_cost: f64,
    pub weekly_tokens: u64,
}

pub struct MessageStats {
    pub started_at: u64,
    pub completed_at: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
}

pub struct BillingBlock {
    pub started_at: u64,
    pub ends_at: u64,    // started_at + 5h
    pub cost: f64,
    pub tokens: u64,
}
```

### 6.2. JSONL-парсер с агрегацией

Порт `utils/jsonl-*.ts` (6 файлов):

```rust
pub fn parse_transcript(path: &Path) -> Option<TranscriptStats> {
    let file = File::open(path).ok()?;
    let reader = BufReader::with_capacity(64 * 1024, file);
    let mut stats = TranscriptStats::default();
    let mut current_block: Option<BillingBlock> = None;

    for line in reader.lines().flatten() {
        let entry: TranscriptEntry = match sonic_rs::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,  // skip broken lines (tail может быть partial)
        };

        match entry.kind {
            EntryKind::User => stats.messages += 1,
            EntryKind::Assistant => {
                stats.messages += 1;
                if let Some(usage) = &entry.usage {
                    stats.tokens_in_total += usage.input_tokens;
                    stats.tokens_out_total += usage.output_tokens;
                    stats.cost_total += compute_cost(usage, &entry.model);
                    update_block(&mut current_block, &mut stats.blocks, &entry);
                    stats.last_message = Some(extract_message_stats(&entry));
                }
            }
            // ...
        }
    }

    if let Some(b) = current_block { stats.blocks.push(b); }
    stats.weekly_cost = stats.blocks.iter()
        .filter(|b| within_last_week(b.started_at))
        .map(|b| b.cost).sum();
    Some(stats)
}
```

> Цены `compute_cost(usage, model)` — порт таблицы из upstream `model-context.ts` или захардкод.

### 6.3. Инвалидация кэша

Расширение алгоритма из Фазы 3:
- Сравнивать **mtime + size**: если size файла > size при создании кэша, делать **incremental parse** (читать только хвост от last_offset).
- Если mtime изменился, но size меньше (truncate) — пересобрать с нуля.

```rust
pub struct CacheMeta {
    pub source_size: u64,
    pub source_mtime_ns: u128,
    pub last_parsed_offset: u64,
    pub format_version: u32,
}

pub struct CacheFile {
    pub meta: CacheMeta,
    pub stats: TranscriptStats,
}

pub fn load_or_build_incremental(path: &Path) -> Option<TranscriptStats> {
    let cache_path = cache_path_for(path);
    let src_meta = fs::metadata(path).ok()?;
    let src_size = src_meta.len();
    let src_mtime = src_meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();

    if let Some(cached) = read_cache(&cache_path) {
        if cached.meta.format_version == FORMAT_VERSION
            && cached.meta.source_size <= src_size
            && cached.meta.source_mtime_ns <= src_mtime
        {
            // incremental: parse from last_parsed_offset
            let new_stats = merge_stats(cached.stats.clone(),
                                        parse_from_offset(path, cached.meta.last_parsed_offset)?);
            write_cache(&cache_path, &CacheFile {
                meta: CacheMeta {
                    source_size: src_size,
                    source_mtime_ns: src_mtime,
                    last_parsed_offset: src_size,
                    format_version: FORMAT_VERSION,
                },
                stats: new_stats.clone(),
            });
            return Some(new_stats);
        }
    }
    // полный rebuild
    let stats = parse_transcript(path)?;
    write_cache(&cache_path, &CacheFile { meta: ..., stats: stats.clone() });
    Some(stats)
}
```

### 6.4. Виджеты

Каждый — тонкий getter:

```rust
impl Widget for BlockTimer {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let stats = ctx.transcript()?;
        let active = stats.blocks.last()?;
        let now = unix_now();
        let remaining = active.ends_at.saturating_sub(now);
        Some(format!("⏰ {}", format_duration(remaining)))
    }
}

impl Widget for InputSpeed {
    fn render(&self, ctx: &RenderContext) -> Option<String> {
        let stats = ctx.transcript()?;
        let last = stats.last_message.as_ref()?;
        let dur_secs = (last.completed_at - last.started_at).max(1);
        let speed = last.tokens_in as f64 / dur_secs as f64;
        Some(format!("↓{:.0} t/s", speed))
    }
}
```

### 6.5. Anthropic Usage API виджет (опционально)

`SessionUsage` в ccstatusline дёргает Anthropic API:
```rust
fn fetch_usage() -> Option<UsageInfo> {
    let key = read_anthropic_key()?;
    let resp = ureq::get("https://api.anthropic.com/v1/usage")
        .set("X-API-Key", &key)
        .timeout(Duration::from_millis(200))
        .call().ok()?;
    resp.into_json().ok()
}
```

С кэшем 5 минут TTL.

### 6.6. Performance test

```bash
hyperfine --warmup 20 --runs 200 \
  "cat benches/samples/payload-with-transcript.json | ./target/release/cchud"
```

Цель:
- Холодный кэш на 50 МБ транскрипта: < 30 мс
- Горячий кэш: < 5 мс

### 6.7. Релиз 0.4.0

## Exit Criteria

- [ ] Все 19 транскрипт-виджетов работают
- [ ] Incremental кэш: добавление 1 МБ к транскрипту → парсинг < 2 мс
- [ ] Cache-versioning: смена format_version → пересобрать без крэша
- [ ] Battle-test: реальная сессия Claude Code с большим транскриптом → виджеты обновляются корректно
- [ ] Поломанные строки JSONL (partial-write) не крэшат
- [ ] Hyperfine: < 5 мс p95 при горячем кэше
- [ ] Релиз 0.4.0

## Связи

- **Фаза 7** добавляет оставшиеся неformaт-виджеты (FreeMemory, VimMode, Skills и т.д.)
- **Фаза 8** TUI-конфигуратор показывает live preview виджетов на транскрипте
- **Фаза 9** README с примерами usage-конфигов

## Риски

- **Цены моделей меняются** — таблица в коде должна быть updateable; рассмотреть JSON-конфиг `~/.config/cchud/model-pricing.json` с дефолтом из embedded.
- **Большой транскрипт + первый раз** — холодный кэш > 30 мс. Документировать; в перспективе — фоновый prewarm-демон (но не для 1.0).
- **Concurrent-write на jsonl** во время парсинга — последняя строка может быть partial; пропускать broken lines.
- **Часовые пояса в block-timer** — все таймстампы в UTC, конверсия только при отображении.
- **Bincode-формат меняется** — `format_version` бамп + автоматический rebuild при mismatch.
