# Task 10 — Release 0.4.0

**Files:**
- Modify: `Cargo.toml` (`version = "0.4.0"`)
- Modify: `Cargo.lock` (auto)
- Modify: `CHANGELOG.md` (новая секция `## 0.4.0 — Transcript widgets`)
- Modify: `README.md` (widget table 46 → 54/60, deps, quickstart с transcript-конфигом)
- Modify: `docs/widgets.md` (8 transcript-виджетов TODO → DONE; Skills counter 6 → 7)
- Modify: `plan/README.md` (Phase 6 → `[x]`, Phase 7 → `[~]`)
- Create: `plan/phase-6/manual-test-log.md` (battle-test протокол)
- Tag: `v0.4.0` (push после merge в main)

## Goal

Финализация Phase 6. Никаких code-изменений в `src/`.

## Inputs

- T9 закрыт: snapshot'ы committed без `.snap.new`, hyperfine gates met, `benches/phase-6.md` записан.
- `cargo test --locked` зелёный, CI matrix зелёный 2 раза подряд.
- Manual battle-test проведён ≥ 5 мин на этом репо с активной CC-сессией.

---

- [ ] **Step 1: Bump version в `Cargo.toml`**

Edit `Cargo.toml`:
- `old_string`: `version = "0.3.0"`
- `new_string`: `version = "0.4.0"`

```bash
cargo build --release --locked
./target/release/cchud --version
```

Expected: `cchud 0.4.0`.

- [ ] **Step 2: Manual battle-test**

Подготовка:

```bash
# 1. Установить Phase 6 cchud в путь, который CC увидит как statusline.
cargo install --path . --locked --force
which cchud   # должно показать ~/.cargo/bin/cchud

# 2. Создать testing config с активными transcript-виджетами.
mkdir -p ~/.config/cchud
cat > ~/.config/cchud/settings.json <<'EOF'
{
  "version": 1,
  "lines": [
    {
      "widgets": [
        {"type": "model"},
        {"type": "git-branch"},
        {"type": "block-timer"},
        {"type": "tokens-total"},
        {"type": "input-speed"},
        {"type": "thinking-effort"}
      ]
    }
  ]
}
EOF

# 3. Включить cchud как statusline в CC settings.
# (См. README.md "Configuration" секцию.)
```

Запустить CC, активно работать ≥ 5 минут (несколько user-messages, несколько assistant-responses, дать модели подумать с `thinking.effort != none`).

Проверки во время сессии:
- `BlockTimer` показывает уменьшающееся время (`⏰ 04:55:00 → 04:50:30 → ...`).
- `TokensTotal` растёт после каждого assistant-message.
- `InputSpeed` обновляется на новое значение после каждого ответа.
- `ThinkingEffort` показывает уровень из последнего assistant (если включён).
- Размер `~/.cache/cchud/transcript-*.bincode` ≤ 5% от transcript file size.
- `rm ~/.cache/cchud/transcript-*.bincode` → следующий рендер пересчитывает (никаких ошибок видимых пользователю).
- Truncate transcript / новая сессия → новый кэш создаётся без артефактов из старого.

Записать результат в `plan/phase-6/manual-test-log.md`:

Create `/Users/igor/mp/startup/cchud/plan/phase-6/manual-test-log.md`:

```markdown
# Phase 6 — Manual battle-test log

**Дата:** 2026-04-XX
**Тестировщик:** Igor Fonin
**Версия:** cchud 0.4.0 (commit `<sha>`)
**Окружение:** mac mini M2 / iTerm2 / Claude Code 1.X
**Конфиг:** `model | git-branch | block-timer | tokens-total | input-speed | thinking-effort`

## Сценарий

1. Запущена активная CC-сессия в репо `cchud` (этот же репозиторий).
2. ~7 пар user/assistant за 5 минут с миксом `thinking.effort` (`high`, `max`).
3. После 5 минут — `rm ~/.cache/cchud/transcript-*.bincode` и проверка следующего рендера.
4. После — новая сессия (`/clear`), проверка чистого кэша.

## Результаты

- [ ] `BlockTimer` корректно показывает `⏰ HH:MM:SS`, монотонно уменьшается.
- [ ] `TokensTotal` инкрементируется после assistant-ответа.
- [ ] `InputSpeed` обновляется (значение меняется между ответами).
- [ ] `ThinkingEffort` показывает `🧠 high` / `🧠 max` после соответствующих ответов.
- [ ] Cache file размер: <X> KB на <Y> KB transcript = <Z>% (target ≤ 5%).
- [ ] Cache wipe → следующий рендер ОК (~9 ms cold; не повисает).
- [ ] `/clear` → новый кэш создаётся; старые статы не leak'ают.

## Найденные баги

(если есть — список с шагами воспроизведения и приоритетом)

## Заключение

- [ ] PASS — все critical-проверки прошли.
- [ ] BLOCKED — баги, нельзя tag'ать.
```

`file_path`: `/Users/igor/mp/startup/cchud/plan/phase-6/manual-test-log.md`.

После прохождения теста — заполнить чекбоксы и `<X>/<Y>/<Z>`.

- [ ] **Step 3: Обновить `CHANGELOG.md`**

Read текущий `CHANGELOG.md`. Найти заголовок `# Changelog`. Добавить новую секцию **выше** записи `0.3.0`:

```markdown
## 0.4.0 — 2026-04-XX

### Added (Phase 6 — Transcript widgets + JSONL cache)

8 transcript-виджетов поверх нового JSONL-кэша с incremental tail-merge:

**Tokens (5):**
- `tokens-cached` — `cT: <fmt>` сумма `cache_read + cache_creation`
- `tokens-total` — `totT: <fmt>` сумма всех 4 групп
- `input-speed` / `output-speed` / `total-speed` — `↓N t/s` / `↑N t/s` / `⇅N t/s` от последнего assistant-сообщения

**Timing (2):**
- `block-timer` — `⏰ HH:MM:SS` time-to-end текущего 5h billing-блока
- `session-duration` — диапазон `last_msg - first_msg` в `HH:MM:SS` / `MM:SS`

**Meta (1):**
- `thinking-effort` — `🧠 {level}` уровень thinking из последнего assistant

### Performance

- Cold parse 50 МБ JSONL < 10 ms (sonic-rs ≈ 3× быстрее serde_json)
- Warm cache hit < 2 ms; warm + 1 МБ append < 3 ms
- Lazy `RenderContext::transcript()` через `OnceCell` — нулевая стоимость для строк без transcript-виджетов
- Один parse на 8 виджетов через shared `TranscriptStats`
- Phase 5 (20 git-виджетов) baseline без регрессии > 10%

### Internals

- `src/cache/`: новый изолированный модуль (jsonl_types, parser, store)
- `cache::store::load_or_build_incremental` — единственный entry point. Hit-path при `format_version match + last_parsed_offset ≤ src.size + mtime_ns ≤ src.mtime_ns`; cold path при любом mismatch (corrupt / truncate / format-bump → silent reset)
- `cache_path_for`: SipHash24 от канонического пути → `~/.cache/cchud/transcript-<hex16>.bincode`
- `format_version: u32 = 1` в `CacheMeta`; mismatch не крэшит
- `RenderContext` +`transcript: OnceCell<Option<TranscriptStats>>` +`now_ms: u64`
- `util/format_tokens.rs` — k/M formatter без trailing zero; `util/now.rs` — `unix_now_ms()`

### Dependencies

- `sonic-rs = "0.5"` (runtime — JSONL парсинг)
- `siphasher = "1"` (runtime — cache-key hash)
- `time = "0.3"` (runtime — ISO-8601 → ms; default features off)
- `filetime = "0.2"` (dev-dep — explicit mtime в append-merge тесте)

### Decisions

См. `docs/DECISIONS.md` D-2026-04-28 — sonic-rs vs serde_json (perf rationale, Windows fallback контракт).

### Scope notes

`Skills` (6 → 7), HTTP-кластер (`SessionUsage`, `WeeklyUsage`, `BlockResetTimer`, `WeeklyResetTimer`, `ClaudeAccountEmail`) и `FreeMemory` остаются в Phase 7. Total widget coverage: 54/60.
```

`file_path`: `/Users/igor/mp/startup/cchud/CHANGELOG.md`.

- [ ] **Step 4: Обновить `README.md`**

Read `README.md`. Найти секцию widget-таблицы (точно так же, как Phase 5 T9 обновлял 26→46/60).

- Заменить статус 8 transcript-виджетов с `❌` / `TODO` на `✅` / `DONE`.
- Cumulative count: 54/60 (46 после Phase 5 + 8 новых).

Если есть quickstart — добавить пример с transcript:

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model"},
      {"type": "git-branch"},
      {"type": "tokens-total"},
      {"type": "block-timer"},
      {"type": "thinking-effort"}
    ]
  }]
}
```

`file_path`: `/Users/igor/mp/startup/cchud/README.md`.

- [ ] **Step 5: Обновить `docs/widgets.md`**

Read `docs/widgets.md`. Найти 8 строк transcript-виджетов; заменить статус TODO → DONE. Skills counter (если присутствует) — `6 → 7`.

`file_path`: `/Users/igor/mp/startup/cchud/docs/widgets.md`.

- [ ] **Step 6: Обновить `plan/README.md`**

Edit `plan/README.md`:
- Phase 6 строка: `[~]` → `[x]`
- Phase 7 строка: `[ ]` → `[~]`

`file_path`: `/Users/igor/mp/startup/cchud/plan/README.md`.

- [ ] **Step 7: Standard gate перед tag**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
cargo insta pending-snapshots   # должно быть пусто
```

Expected: все exit 0; pending пусто.

- [ ] **Step 8: Commit release**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md docs/widgets.md plan/README.md plan/phase-6/manual-test-log.md
git commit -m "release: 0.4.0 — transcript widgets + JSONL cache

Phase 6 ship — 8 transcript widgets (5 tokens, 2 timing, 1 meta) atop
a new incremental JSONL cache with bincode persistence.

- Cold parse 50MB <10ms (sonic-rs); warm <2ms; warm+1MB append <3ms.
- Lazy transcript() via OnceCell — non-transcript widgets pay zero IO.
- format_version-bumped bincode cache; corrupt/truncate/mismatch → silent reset.
- 8 widgets cumulative 54/60 vs upstream ccstatusline.

Manual battle-test 5 min on this repo: PASS (see plan/phase-6/manual-test-log.md).
Hyperfine: see benches/phase-6.md.

Closes Phase 6."
```

- [ ] **Step 9: Создать tag**

```bash
git tag -a v0.4.0 -m "0.4.0 — Transcript widgets (Phase 6)"
git push origin main
git push origin v0.4.0
```

> **Confirm with user before pushing**: push к main и tag — visible to others. Если репозиторий публичный, тег запускает `cargo publish` через GitHub Actions если настроено. Уточнить у user перед `git push`.

- [ ] **Step 10: Verification после tag**

```bash
git tag --list 'v0.4.0'
./target/release/cchud --version
cargo install --path . --locked --force
cchud --version
```

Expected:
```
v0.4.0
cchud 0.4.0
cchud 0.4.0
```

- [ ] **Step 11: Закрытие плана**

Verification что Phase 6 закрыта:

```bash
grep -E '^\| 6 \|.*\[x\]' plan/README.md   # Phase 6 done
grep -E '^\| 7 \|.*\[~\]' plan/README.md   # Phase 7 in progress
grep -c "0.4.0" CHANGELOG.md
ls plan/phase-6/manual-test-log.md
```

Expected: все checks ОК.

- [ ] **Step 12: Уведомить о завершении**

Финальная команда — короткое сообщение в канал команды (если есть) или в `git commit log` на следующий PR:

> Phase 6 (`v0.4.0`) shipped — 8 transcript widgets, JSONL incremental cache, sonic-rs hot path. Coverage 54/60 vs upstream. Phase 7 unblocked.

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
git tag --list 'v0.4.0'
./target/release/cchud --version
```

## Definition of Done

- [ ] `Cargo.toml` `version = "0.4.0"`; `cchud --version` показывает `0.4.0`
- [ ] `CHANGELOG.md` содержит `## 0.4.0` секцию с 8 виджетами + perf + deps + decisions
- [ ] `README.md` widget table обновлён 46 → 54/60; quickstart с transcript-примером
- [ ] `docs/widgets.md`: 8 transcript-виджетов TODO → DONE; Skills 6 → 7
- [ ] `plan/README.md`: Phase 6 → `[x]`, Phase 7 → `[~]`
- [ ] `plan/phase-6/manual-test-log.md` создан и заполнен (PASS)
- [ ] `git tag v0.4.0` создан и запушен (после user confirmation)
- [ ] CI matrix (macos / ubuntu / windows) зелёный 2 раза подряд
- [ ] Hyperfine targets met (см. `benches/phase-6.md`)
- [ ] Один commit `release: 0.4.0 — ...`

## Files touched

- `Cargo.toml` (modified — version)
- `Cargo.lock` (auto)
- `CHANGELOG.md` (modified — `## 0.4.0` секция)
- `README.md` (modified — widget table + quickstart)
- `docs/widgets.md` (modified — 8 transcript строк → DONE)
- `plan/README.md` (modified — Phase 6/7 status)
- `plan/phase-6/manual-test-log.md` (created)

## Risks & rollback

- **Push к main без code review**: если repo требует PR — открыть PR с `release: 0.4.0` коммитом, дождаться CI зелёного, merge. Tag — после merge.
- **`cargo install --path .` не подтянет registry deps**: если `cargo publish` ещё не сделан, `cargo install` берёт из локального path. ОК для self-test.
- **CI fails в release-mode**: чаще всего связано с warning'ами от новых deps на одной из платформ. Митигация: повторить `cargo clippy --locked` локально перед push; если windows-specific — добавить `cfg(windows)` ветку.
- **Manual battle-test обнаруживает баг**: блокирует tag. Откат — продолжить работу в Phase 6, T10 не закрывать. Создать `plan/phase-6/task-10-followup-<bug>.md` если баг крупный.
- **Rollback**: `git tag -d v0.4.0 && git push origin :refs/tags/v0.4.0`. CHANGELOG / README / version — `git revert <release-commit>`. Phase 7 разблокирующий маркер не пострадает (план остаётся, можно ре-релизить).
