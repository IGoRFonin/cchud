# Фаза 0 — Проверки и калибровка

**Длительность:** 1–2 часа
**Входные условия:** ничего
**Релиз:** —

## Цель

Валидировать допущения ресерча и собрать baseline-данные **до написания кода**. Любая цифра в PRD должна быть подтверждена на твоей машине.

## Шаги

### 0.1. Окружение

```bash
rustc --version              # ≥ 1.85 (edition 2024)
cargo --version
gh auth status               # для будущих API-вызовов
hyperfine --version || brew install hyperfine
sysctl -n machdep.cpu.brand_string  # подтвердить M-серию (для sonic-rs SIMD)
node --version               # для bootstrap-style npm wrapper
```

Если `rustc < 1.85` — `rustup update stable`.

### 0.2. Проверка занятости имён

```bash
# crates.io
curl -sI https://crates.io/api/v1/crates/cchud | head -1
# npm
npm view cchud 2>&1 | head -3
# Homebrew tap
gh repo view igorfonin/homebrew-tap 2>&1 | head -3
```

Если `cchud` где-то занят — записать в `docs/DECISIONS.md` решение и fallback. Кандидаты: `cchud-cli`, `claude-hud`, `ccline`, `cchud-rs`.

### 0.3. Сбор реальных payload'ов

ccstatusline получает JSON через stdin от Claude Code. Нужно ≥ 10 разных, чтобы покрыть варианты (с git, без git, разные модели, разные транскрипты).

```bash
npm i -g ccstatusline@2.2.8
mkdir -p ~/.cache/cchud-dev/samples

cat > /tmp/cchud-trace.sh <<'EOF'
#!/bin/bash
TS=$(date +%s%N)
tee ~/.cache/cchud-dev/samples/payload-$TS.json | ccstatusline
EOF
chmod +x /tmp/cchud-trace.sh
```

Подменить временно `~/.claude/settings.json`:
```json
{ "statusLine": { "command": "/tmp/cchud-trace.sh", "padding": 0 } }
```

Поработать в Claude Code 5–10 минут на разных проектах:
- Проект с git + чистым worktree
- Проект с git + dirty changes
- Проект без git
- Длинный транскрипт (>1 МБ)
- Свежий транскрипт (мало сообщений)
- С разными моделями (если есть доступ)

После — восстановить `~/.claude/settings.json`. Скопировать seeds в `benches/samples/`:
```bash
cp ~/.cache/cchud-dev/samples/*.json /Users/igor/mp/startup/cchud/benches/samples/
```

### 0.4. Baseline бенчи

```bash
SAMPLE=/Users/igor/mp/startup/cchud/benches/samples/$(ls /Users/igor/mp/startup/cchud/benches/samples | head -1)

hyperfine --warmup 10 --runs 200 --export-markdown /Users/igor/mp/startup/cchud/benches/baseline.md \
  "cat $SAMPLE | npx -y ccstatusline@latest" \
  "cat $SAMPLE | ccstatusline"
```

RAM-замер:
```bash
/usr/bin/time -l ccstatusline < $SAMPLE 2>&1 | grep "maximum resident"
```

Запиши обе цифры в `benches/baseline.md` — это floor для Rust-цели.

### 0.5. Карта upstream — что портировать

Прочитать и законспектировать в `docs/upstream-map.md`:

| Upstream-файл | Что портируется | Куда в cchud |
|---|---|---|
| `src/types/Settings.ts` | Схема конфига | `src/config/types.rs` (serde-derive) |
| `src/types/RenderContext.ts` | Контракт виджетов | `src/widgets/context.rs` |
| `src/types/StatusJSON.ts` | Схема payload | `src/types/payload.rs` |
| `src/utils/jsonl-cache.ts` | Алгоритм mtime+size инвалидации | `src/cache/jsonl.rs` |
| `src/utils/powerline.ts` | Powerline-рендер | `src/render/powerline.rs` |
| `src/utils/migrations.ts` | Миграции конфига | `src/config/migrations.rs` |
| `src/widgets/index.ts` | Реестр виджетов | `src/widgets/registry.rs` |
| `src/utils/git.ts`, `git-remote.ts` | Git-обёртки | `src/git/*.rs` (через gix) |

### 0.6. Список 60+ виджетов с приоритетом

Создать `docs/widgets.md` с таблицей:

| ccstatusline name | Phase | Source | Complexity | Status |
|---|---|---|---|---|
| Model | 2 | payload | low | TODO |
| ContextLength | 3 | payload | low | TODO |
| ... |

Где Source ∈ `{payload, git, transcript, env, http}` и Complexity ∈ `{low, mid, high}`.

### 0.7. Решения по нерешённым OQ из PRD

Записать в `docs/DECISIONS.md` решения по тем OQ, на которые есть ответ после фазы 0:
- OQ-1 (формат settings) — посмотреть payload'ы и settings.json от ccstatusline
- OQ-6 (имя cchud свободно) — результат 0.2

## Exit Criteria

- [ ] Все имена `cchud` свободны (или есть fallback в DECISIONS.md)
- [ ] ≥ 10 разных payload-семплов в `benches/samples/`
- [ ] `benches/baseline.md` с цифрами cold-start (npx, global) и RAM
- [ ] `docs/upstream-map.md` с конспектом типов и алгоритмов
- [ ] `docs/widgets.md` с приоритезированным списком всех 60+ виджетов
- [ ] `docs/DECISIONS.md` с ответами на OQ-1, OQ-6 (минимум)

## Связи

- **Фаза 1** использует payload-семплы для skeleton-теста
- **Фаза 2** использует upstream-map для типов и pipeline
- **Фаза 3** использует widgets.md для приоритезации MVP
- **Фаза 9** использует baseline.md для финальной таблицы в README

## Риски

- Если M-серии нет / только x86_64 Mac — sonic-rs всё равно работает, просто без ARM SIMD-пути.
- Если `ccstatusline@2.2.8` уже устарел к моменту начала — взять текущий, обновить версию в PRD.
- Если payload Claude Code сильно отличается от ожидаемого — это меняет схему `StatusJSON`. Не страшно, фаза 2 ещё не написана.
