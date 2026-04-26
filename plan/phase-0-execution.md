# Фаза 0 — План выполнения

> **Спецификация:** [`phase-0-checks.md`](./phase-0-checks.md) (что и зачем)
> **Этот файл:** как именно мы это сделаем — пошагово, с командами и ожидаемыми результатами
>
> **Для агента-исполнителя:** REQUIRED SUB-SKILL — `superpowers:subagent-driven-development` (рекомендовано) или `superpowers:executing-plans`. Шаги идут в порядке зависимостей; чекбоксы (`- [ ]`) — для трекинга. Любой шаг с пометкой **USER REQUIRED** ставит выполнение на паузу до подтверждения пользователя.

**Цель:** валидировать допущения PRD и собрать baseline-данные **до написания кода**, чтобы фазы 1–9 опирались на реальные цифры и реальные payload'ы Claude Code.

**Архитектура:** 8 задач, выполняются преимущественно последовательно (Task 5–7 могут идти параллельно после Task 3). Кода не пишем — только shell-команды, конспекты и таблицы. `git init` отложен на фазу 1, поэтому коммитов внутри фазы 0 нет — артефакты копятся в `benches/` и `docs/`, фаза 1 их зафиксирует первым коммитом.

**Tech Stack:** `rustc ≥ 1.85`, `cargo`, `gh`, `hyperfine`, `npm`, `ccstatusline@2.2.8` (или текущий, если устарел), `/usr/bin/time -l`.

**Состояние на старте (проверено):**
- `benches/samples/` существует, **пуст**.
- `benches/baseline.md` — нет.
- `docs/upstream-map.md`, `docs/widgets.md`, `docs/DECISIONS.md` — нет.
- Не git-репозиторий.

**Не делаем в фазе 0:** `git init` (фаза 1), `cargo init` (фаза 1), правка PRD (только append в DECISIONS.md), любые коммиты.

---

## Task 1: Окружение (PRD §0.1)

**Цель:** убедиться, что все нужные тулы установлены и нужных версий. Любой gap фиксируем сразу — фазы 1–9 предполагают, что эти тулы есть.

**Файлы:** —

- [ ] **Шаг 1.1: rustc ≥ 1.85 (edition 2024)**

```bash
rustc --version
```

Ожидаемо: `rustc 1.85.0 (...)` или новее. Если меньше — `rustup update stable`, затем повторить.

- [ ] **Шаг 1.2: cargo доступен**

```bash
cargo --version
```

Ожидаемо: версия совпадает с rustc по minor.

- [ ] **Шаг 1.3: gh авторизован**

```bash
gh auth status
```

Ожидаемо: `Logged in to github.com account igorfonin (...)`. Если нет — пользователь делает `gh auth login` (interactive — `! gh auth login` в этой сессии).

- [ ] **Шаг 1.4: hyperfine установлен**

```bash
hyperfine --version || brew install hyperfine
```

Ожидаемо: версия. Установка через brew без подтверждений безопасна, но если brew спросит pass — это пользовательский шаг.

- [ ] **Шаг 1.5: подтвердить M-серию (для sonic-rs SIMD-пути)**

```bash
sysctl -n machdep.cpu.brand_string
```

Ожидаемо: `Apple M*`. Если x86_64 — записать в `docs/DECISIONS.md` (Task 7), что бенчи macOS будут на x86_64 пути sonic-rs.

- [ ] **Шаг 1.6: node для bootstrap-style npm wrapper (фаза 9)**

```bash
node --version
```

Ожидаемо: ≥ 18 (для современного npm). Если нет — отметить в DECISIONS.md, что node понадобится в фазе 9.

- [ ] **Шаг 1.7: зафиксировать версии**

В конце Task 7 (`docs/DECISIONS.md`) запишем фактические версии rustc/cargo/node/hyperfine/gh — это baseline для CI matrix в фазе 1.

---

## Task 2: Проверка занятости имён (PRD §0.2)

**Цель:** убедиться, что имя `cchud` свободно на всех трёх каналах дистрибуции (crates.io, npm, GitHub homebrew tap). Закрывает OQ-6 из PRD.

**Файлы:**
- Modify: `docs/DECISIONS.md` (создаётся в Task 7) — записать результат

- [ ] **Шаг 2.1: crates.io**

```bash
curl -sI https://crates.io/api/v1/crates/cchud | head -1
```

Интерпретация:
- `HTTP/2 404` → имя свободно ✓
- `HTTP/2 200` → занято; смотрим `curl -s https://crates.io/api/v1/crates/cchud | head -c 400`, чтобы понять, кто владелец и активен ли пакет.

- [ ] **Шаг 2.2: npm**

```bash
npm view cchud 2>&1 | head -20
```

Интерпретация:
- `npm error 404 ... is not in this registry` → свободно ✓
- Иначе вывод метаданных пакета → занято; зафиксировать имя владельца, last-publish.

- [ ] **Шаг 2.3: GitHub homebrew tap**

```bash
gh repo view igorfonin/homebrew-tap 2>&1 | head -5
```

Интерпретация:
- `GraphQL: Could not resolve to a Repository ...` → tap ещё не создан, имя репо доступно ✓ (это норм; tap создаём в фазе 9).
- Любой другой вывод — описать в DECISIONS.md.

- [ ] **Шаг 2.4: дополнительная проверка — GitHub-репо для самого `cchud`**

```bash
gh repo view igorfonin/cchud 2>&1 | head -5
```

Интерпретация: если 404 — нужно создать в фазе 1; если уже есть — проверить состояние.

- [ ] **Шаг 2.5: при необходимости — fallback**

Если хотя бы один канал занят, выбираем fallback из PRD: `cchud-cli`, `claude-hud`, `ccline`, `cchud-rs`. Перепроверяем три имени по тем же командам и фиксируем выбор в DECISIONS.md (Task 7) с обоснованием.

---

## Task 3: Сбор реальных payload'ов (PRD §0.3) — **USER REQUIRED**

**Цель:** получить ≥ 10 разных JSON-payload'ов от Claude Code, покрывающих основные режимы (git/no-git, dirty/clean, разные модели, длинный/свежий транскрипт). Это seeds для всех snapshot-тестов с фазы 2 и далее.

**Файлы:**
- Create: `/tmp/cchud-trace.sh` (временный)
- Modify: `~/.claude/settings.json` (временно, с бэкапом)
- Create: `~/.cache/cchud-dev/samples/payload-*.json`
- Create: `benches/samples/payload-*.json` (финальное место)

### 3a. Подготовка инструментария (агент)

- [ ] **Шаг 3a.1: установить ccstatusline**

```bash
npm i -g ccstatusline@2.2.8
ccstatusline --version 2>&1 | head -3
```

Если 2.2.8 не существует на npm (устарел/yanked), берём latest:

```bash
npm view ccstatusline version
npm i -g ccstatusline@latest
```

Зафиксировать **фактическую** версию в DECISIONS.md (Task 7) и при необходимости — обновить `upstream_parity_target` в PRD. Изменение PRD делать **отдельным изменением после фазы 0**, не сейчас.

- [ ] **Шаг 3a.2: создать каталог семплов**

```bash
mkdir -p ~/.cache/cchud-dev/samples
ls -la ~/.cache/cchud-dev/samples
```

Ожидаемо: каталог пустой.

- [ ] **Шаг 3a.3: создать trace-скрипт**

Записать в `/tmp/cchud-trace.sh`:

```bash
#!/bin/bash
TS=$(date +%s%N)
tee "$HOME/.cache/cchud-dev/samples/payload-$TS.json" | ccstatusline
```

И сделать исполняемым:

```bash
chmod +x /tmp/cchud-trace.sh
/tmp/cchud-trace.sh < /dev/null > /dev/null
ls ~/.cache/cchud-dev/samples/ | head -3
```

Ожидаемо: один пустой файл `payload-*.json` (smoke-test записи). Удалить его перед реальным сбором:

```bash
rm ~/.cache/cchud-dev/samples/payload-*.json
```

- [ ] **Шаг 3a.4: бэкап текущего settings.json**

```bash
cp ~/.claude/settings.json ~/.claude/settings.json.cchud-bak-$(date +%Y%m%d-%H%M%S)
ls ~/.claude/settings.json.cchud-bak-* | tail -1
```

Ожидаемо: путь к бэкапу. **Это критично** — без бэкапа Шаг 3b.4 нечего восстанавливать.

- [ ] **Шаг 3a.5: показать пользователю текущий statusLine-блок**

```bash
jq '.statusLine // "no statusLine block"' ~/.claude/settings.json
```

Запомнить (или скопировать пользователю в чат) — это то, что нужно будет вернуть в Шаге 3b.4.

- [ ] **Шаг 3a.6: подменить statusLine на trace-скрипт**

Обновить `~/.claude/settings.json` так, чтобы блок `statusLine` стал:

```json
{ "command": "/tmp/cchud-trace.sh", "padding": 0 }
```

Использовать `jq` для idempotent-правки (не перезаписывать остальные ключи):

```bash
TMP=$(mktemp)
jq '.statusLine = {command: "/tmp/cchud-trace.sh", padding: 0}' \
   ~/.claude/settings.json > "$TMP" && mv "$TMP" ~/.claude/settings.json
jq '.statusLine' ~/.claude/settings.json
```

Ожидаемо: на выводе видим новый блок.

### 3b. Сбор данных (USER REQUIRED) — **остановка агента**

- [ ] **Шаг 3b.1: пользователь работает в Claude Code 5–10 минут**

Сценарии (отметить по мере прохождения):

- [ ] Проект с git + чистый worktree
- [ ] Проект с git + dirty changes (создать grep-able файл, оставить незакоммиченным)
- [ ] Проект без git (например, `mkdir /tmp/no-git && cd /tmp/no-git && claude`)
- [ ] Длинный транскрипт (> 1 МБ) — открыть существующую долгую сессию
- [ ] Свежий транскрипт — `claude` в новом каталоге, 2–3 сообщения
- [ ] Если есть доступ к разным моделям — переключиться на каждую (`/model`)

Каждое окно Claude Code, открытое с подменённым settings.json, начинает писать payload'ы каждые ~300 мс. Достаточно держать окно открытым ~30 сек на каждый сценарий.

- [ ] **Шаг 3b.2: проверить, что payload'ы накопились**

```bash
ls ~/.cache/cchud-dev/samples/ | wc -l
ls -lhS ~/.cache/cchud-dev/samples/ | head -5
```

Ожидаемо: ≥ 30–60 файлов (трейсер не дедуплицирует — файлов будет много, выберем 10+ разных в шаге 3b.5).

### 3c. Восстановление + куратирование (агент)

- [ ] **Шаг 3b.3: проверить структуру одного payload'а**

```bash
ls ~/.cache/cchud-dev/samples/ | head -1 | xargs -I{} jq . ~/.cache/cchud-dev/samples/{} | head -40
```

Ожидаемо: видим JSON со ссылкой на transcript-файл, текущую модель, cwd, etc. Если payload не валидный JSON — диагностировать (возможно, ccstatusline что-то добавил в stdout).

- [ ] **Шаг 3b.4: восстановить settings.json**

```bash
LATEST_BAK=$(ls -t ~/.claude/settings.json.cchud-bak-* | head -1)
cp "$LATEST_BAK" ~/.claude/settings.json
jq '.statusLine' ~/.claude/settings.json
```

Ожидаемо: на выводе — оригинальный блок (тот, что зафиксировали в шаге 3a.5). **Не удалять `.cchud-bak-*` — пусть полежит до конца фазы 0** на случай, если что-то пойдёт не так.

- [ ] **Шаг 3b.5: куратировать ≥ 10 разных семплов в `benches/samples/`**

Стратегия отбора: разные `cwd`, разные размеры transcript-файла, разные модели. Грубо — по одному из каждого сценария 3b.1 + дополнительные.

```bash
cd ~/.cache/cchud-dev/samples
# отсортировать по размеру (proxy на «разные транскрипты»):
ls -S | head -20
```

Вручную (или скриптом) выбрать ≥ 10 уникальных и скопировать с осмысленными именами:

```bash
DEST=/Users/igor/mp/startup/cchud/benches/samples
mkdir -p "$DEST"
# пример именования: payload-<scenario>-<short-hash>.json
# scenario ∈ {git-clean, git-dirty, no-git, transcript-large, transcript-small, model-opus, model-sonnet, ...}
```

После — проверить:

```bash
ls -la /Users/igor/mp/startup/cchud/benches/samples/ | wc -l
# ожидаемо: ≥ 10 файлов + строка total
```

- [ ] **Шаг 3b.6: убедиться, что в семплах нет секретов**

```bash
grep -r -l -E '(sk-ant-|ghp_|github_pat_|AKIA[0-9A-Z]{16})' /Users/igor/mp/startup/cchud/benches/samples/ || echo "no secrets found"
```

Ожидаемо: `no secrets found`. Если что-то нашлось — пересмотреть payload и либо удалить файл, либо зачистить секрет (но обычно payload Claude Code не содержит API-ключей; если содержит — это сюрприз, документируем в DECISIONS.md).

- [ ] **Шаг 3b.7: cleanup временных артефактов**

```bash
rm /tmp/cchud-trace.sh
# ~/.cache/cchud-dev/samples/ оставить как локальный кэш (можем вернуться)
# ~/.claude/settings.json.cchud-bak-* оставить до конца фазы 0
```

---

## Task 4: Baseline-бенчи (PRD §0.4)

**Цель:** зафиксировать cold-start CPU (ccstatusline через `npx -y` и через global install) и пиковую RAM. Это **floor** для Rust-цели (PRD §6 Performance) и контент для финальной таблицы README в фазе 9.

**Файлы:**
- Create: `benches/baseline.md`

**Зависит от:** Task 3 (нужен хотя бы один валидный payload).

- [ ] **Шаг 4.1: выбрать представительный sample**

```bash
SAMPLE=$(ls /Users/igor/mp/startup/cchud/benches/samples/*.json | head -1)
echo "$SAMPLE"
wc -c "$SAMPLE"
```

Ожидаемо: путь и размер. Если sample совсем мелкий (< 200 байт) — выбрать средний по размеру:

```bash
# берём sample медианного размера: сортировка по размеру + выбор середины
FILES=( $(ls -S /Users/igor/mp/startup/cchud/benches/samples/*.json) )
SAMPLE=${FILES[${#FILES[@]}/2]}
echo "$SAMPLE"
wc -c "$SAMPLE"
# либо вручную выбрать «обычный» сценарий — git-clean medium transcript
```

Документ требует **одного** sample для baseline; полный матричный бенч идёт в фазе 9.

- [ ] **Шаг 4.2: hyperfine — npx-режим vs global-режим**

```bash
hyperfine --warmup 10 --runs 200 \
  --export-markdown /Users/igor/mp/startup/cchud/benches/baseline.md \
  "cat $SAMPLE | npx -y ccstatusline@latest" \
  "cat $SAMPLE | ccstatusline"
```

Ожидаемо: `baseline.md` с двумя строками таблицы (mean, stddev, min, max, relative). По PRD §1 npx-режим должен быть в районе 50–150 мс; global — заметно быстрее, но всё ещё миллисекунды-десятки миллисекунд. Если цифры **сильно** отличаются от ожидаемого (например, npx < 10 мс или global > 300 мс) — пересмотреть PRD §1.

- [ ] **Шаг 4.3: пиковая RAM (RSS)**

```bash
/usr/bin/time -l ccstatusline < $SAMPLE 2>&1 | grep -E "maximum resident|peak memory"
```

Ожидаемо: на macOS — строка `... maximum resident set size` в байтах. На M-серии для node-приложения обычно 50–100 МБ. Цифру записать вручную.

- [ ] **Шаг 4.4: дополнить baseline.md контекстом**

Открыть `benches/baseline.md` и **дописать** после автогенерированной таблицы:

```markdown
## Контекст замера

- **Машина:** <вывод `sysctl -n machdep.cpu.brand_string`> / <RAM, GB>
- **OS:** <`sw_vers -productVersion`>
- **node:** <`node --version`>
- **ccstatusline:** <`ccstatusline --version` или `npm view ccstatusline version`>
- **hyperfine:** <`hyperfine --version`>
- **Sample:** `<basename SAMPLE>`, <wc -c байт>, сценарий: <git-clean/dirty/...>
- **Дата:** <`date -u +%Y-%m-%dT%H:%M:%SZ`>

## RAM (пик RSS, ccstatusline global)

- maximum resident set size: <число> байт ≈ <число> МБ

## Цели cchud (из PRD §2)

- cold-start p95 (M-серия): **< 5 мс**  ← запас от текущего baseline: ×<кратность>
- RSS пик: **< 5 МБ**  ← запас от текущего baseline: ×<кратность>
```

- [ ] **Шаг 4.5: верификация**

```bash
cat /Users/igor/mp/startup/cchud/benches/baseline.md
```

Ожидаемо: видим таблицу + контекст + цели. Файл готов как floor для бенч-сравнений в CI фазы 1.

---

## Task 5: Карта upstream — что портировать (PRD §0.5)

**Цель:** в `docs/upstream-map.md` зафиксировать конкретные файлы/типы/алгоритмы upstream-проекта ccstatusline, которые нужно портировать в фазах 2–8. Без этой карты фаза 2 (`StatusJSON`, `Settings`, `RenderContext`) пишется вслепую.

**Файлы:**
- Create: `/tmp/cchud-upstream/` (рабочий каталог; удаляется в конце)
- Create: `docs/upstream-map.md`

**Зависит от:** Task 2 (нужно знать, какой репозиторий клонируем — обычно тот, что указан в `npm view ccstatusline repository`).

- [ ] **Шаг 5.1: найти upstream-репозиторий**

```bash
npm view ccstatusline repository.url
npm view ccstatusline homepage
```

Ожидаемо: URL вида `git+https://github.com/<owner>/ccstatusline.git`. Зафиксировать `<owner>/<repo>`.

- [ ] **Шаг 5.2: клонировать на фиксированный тег**

```bash
mkdir -p /tmp/cchud-upstream
cd /tmp/cchud-upstream
TAG=v$(npm view ccstatusline version)   # либо v2.2.8 если зафиксировали
# Используем git clone напрямую — у gh repo clone передача флагов в git ненадёжна.
# Подставь <owner> из шага 5.1.
git clone --depth 1 --branch "$TAG" https://github.com/<owner>/ccstatusline.git 2>&1 | tail -5
ls ccstatusline/src/ 2>&1 | head -20
```

Если тег `v$TAG` не существует (некоторые upstream используют просто `2.2.8` без `v`), повторить без префикса:

```bash
git clone --depth 1 --branch "${TAG#v}" https://github.com/<owner>/ccstatusline.git
```

Ожидаемо: видим структуру `src/` с `types/`, `utils/`, `widgets/`, etc. Если структура **сильно** отличается от ожидаемой PRD'шной — это уже сигнал, обновим upstream-map под реальность.

- [ ] **Шаг 5.3: проверить, что все ожидаемые файлы существуют**

```bash
cd /tmp/cchud-upstream/ccstatusline
for f in \
  src/types/Settings.ts \
  src/types/RenderContext.ts \
  src/types/StatusJSON.ts \
  src/utils/jsonl-cache.ts \
  src/utils/powerline.ts \
  src/utils/migrations.ts \
  src/widgets/index.ts \
  src/utils/git.ts \
  src/utils/git-remote.ts; do
  [ -f "$f" ] && echo "OK  $f" || echo "MISS $f"
done
```

Ожидаемо: все `OK`. Если что-то `MISS` — найти аналог:

```bash
# пример поиска переименованного файла
find . -type f -name "*.ts" | xargs grep -l "interface Settings" | head -5
```

Зафиксировать фактические пути.

- [ ] **Шаг 5.4: создать `docs/upstream-map.md`**

Шаблон (заполняется фактическими данными из шага 5.3):

```markdown
# Upstream map — ccstatusline → cchud

> **Версия upstream:** <TAG>
> **Репо:** <owner>/ccstatusline
> **Дата снимка:** <дата>
> **Локальный клон:** /tmp/cchud-upstream/ccstatusline (временный, удалить после фазы 0)

## Карта портирования

| Upstream-файл | LOC | Что портируется | Куда в cchud | Фаза |
|---|---|---|---|---|
| `src/types/Settings.ts` | <N> | Схема конфига | `src/config/types.rs` (serde-derive) | 2 |
| `src/types/RenderContext.ts` | <N> | Контракт виджетов | `src/widgets/context.rs` | 2 |
| `src/types/StatusJSON.ts` | <N> | Схема payload | `src/types/payload.rs` | 2 |
| `src/utils/jsonl-cache.ts` | <N> | mtime+size инвалидация | `src/cache/jsonl.rs` | 6 |
| `src/utils/powerline.ts` | <N> | Powerline-рендер | `src/render/powerline.rs` | 4 |
| `src/utils/migrations.ts` | <N> | Миграции конфига | `src/config/migrations.rs` | 2 |
| `src/widgets/index.ts` | <N> | Реестр виджетов | `src/widgets/registry.rs` | 2 |
| `src/utils/git.ts` | <N> | Git-обёртки | `src/git/info.rs` (gix) | 5 |
| `src/utils/git-remote.ts` | <N> | Git remote | `src/git/remote.rs` (gix) | 5 |

LOC берём через `wc -l <file>` для оценки усилий.

## Конспект ключевых типов

### `StatusJSON` (`src/types/StatusJSON.ts`)
<вставить структуру: какие поля payload приходят от Claude Code; обязательные vs опциональные>

### `Settings` (`src/types/Settings.ts`)
<вставить корневую схему конфига: lines, widgets, theme, customCommands, ...>

### `RenderContext` (`src/types/RenderContext.ts`)
<вставить контракт, который видит виджет: payload, settings, lazy git, lazy transcript, ...>

## Алгоритмы

### JSONL-кэш (`src/utils/jsonl-cache.ts`)
<коротко: ключ кэша (path + mtime + size), что хранится, как инвалидируется>

### Powerline (`src/utils/powerline.ts`)
<glyphs U+E0Bx, цветовые переходы, fallback-режим>

### Миграции (`src/utils/migrations.ts`)
<цепочка `migrate_vN_to_vN+1`, как определяется текущая версия>

## Реестр виджетов (`src/widgets/index.ts`)

Список имён → используется как input для Task 6 (`docs/widgets.md`).
```

LOC снять автоматически:

```bash
cd /tmp/cchud-upstream/ccstatusline
wc -l src/types/Settings.ts src/types/RenderContext.ts src/types/StatusJSON.ts \
      src/utils/jsonl-cache.ts src/utils/powerline.ts src/utils/migrations.ts \
      src/widgets/index.ts src/utils/git.ts src/utils/git-remote.ts
```

Конспекты типов — буквально вставить интерфейсы из `.ts`-файлов (read + skim, не дословно). Цель — чтобы фаза 2 могла начать писать `serde`-структуры, не возвращаясь в upstream.

- [ ] **Шаг 5.5: верификация**

```bash
wc -l /Users/igor/mp/startup/cchud/docs/upstream-map.md
grep -c "^|" /Users/igor/mp/startup/cchud/docs/upstream-map.md   # строк в таблицах
```

Ожидаемо: файл не пустой, в таблице портирования ≥ 9 строк (по числу upstream-файлов).

---

## Task 6: Список 60+ виджетов с приоритетом (PRD §0.6)

**Цель:** `docs/widgets.md` — таблица всех виджетов ccstatusline с указанием фазы, источника данных и сложности. Это input для фаз 3 (MVP), 5 (git), 6 (transcript), 7 (остальные).

**Файлы:**
- Create: `docs/widgets.md`

**Зависит от:** Task 5 (нужен клон upstream для извлечения списка).

- [ ] **Шаг 6.1: извлечь канонический список виджетов**

```bash
cd /tmp/cchud-upstream/ccstatusline
ls src/widgets/ | grep -v index.ts
# или, если виджеты не файлами, а export'ами:
grep -E "^export (const|class|function)" src/widgets/index.ts | head -80
```

Ожидаемо: список из 60+ имён. Зафиксировать **точный** список (вставить в DECISIONS.md, если число ≠ 60+ — обновим PRD §2).

- [ ] **Шаг 6.2: классифицировать каждый виджет**

Для каждого имени определить:
- **Source** ∈ `{payload, git, transcript, env, http, static}` — откуда берёт данные
- **Complexity** ∈ `{low, mid, high}` — low: read-from-payload, mid: gix или env, high: HTTP/transcript-cache/Powerline-spec
- **Phase** ∈ `{2, 3, 4, 5, 6, 7}` — на какой фазе появляется (используем PRD-распределение и здравый смысл)

Источник классификации: имя виджета + быстрый peek в `src/widgets/<name>.ts` (15–30 сек на каждый).

- [ ] **Шаг 6.3: создать `docs/widgets.md`**

Шаблон:

```markdown
# Реестр виджетов cchud (паритет с ccstatusline <TAG>)

> **Источник:** `src/widgets/index.ts` upstream snapshot (см. `upstream-map.md`)
> **Всего:** <N>
> **Дата:** <дата>

## Колонки

- **Name** — имя в ccstatusline (точное)
- **Phase** — на какой фазе cchud добавляется
- **Source** — откуда данные: `payload` | `git` | `transcript` | `env` | `http` | `static`
- **Complexity** — `low` | `mid` | `high`
- **Status** — `TODO` | `WIP` | `DONE` (обновляется в фазах)

## Виджеты

| Name | Phase | Source | Complexity | Status | Note |
|---|---|---|---|---|---|
| Model | 2 | payload | low | TODO | sentinel — первый виджет |
| ContextLength | 3 | payload | low | TODO | |
| ... | | | | | |

## Сводка по фазам

| Фаза | Кол-во виджетов | Кумулятивно |
|---|---|---|
| 2 | 1 | 1 |
| 3 | 9 (MVP топ-10) | 10 |
| 4 | 0 (рендер, не виджеты) | 10 |
| 5 | <N> (git) | ... |
| 6 | <N> (transcript) | ... |
| 7 | <N> (остальные) | <всего> |
```

Заполнить таблицу из шага 6.2.

- [ ] **Шаг 6.4: верификация**

```bash
grep -c "^| [A-Z]" /Users/igor/mp/startup/cchud/docs/widgets.md
```

Ожидаемо: ≥ 60 (по числу виджетов upstream). Если меньше 60 — пересмотреть, не пропустили ли часть `index.ts`.

---

## Task 7: Решения по нерешённым OQ (PRD §0.7)

**Цель:** `docs/DECISIONS.md` — append-only журнал решений по open questions PRD. Минимум: OQ-1 (формат settings) и OQ-6 (имя cchud свободно).

**Файлы:**
- Create: `docs/DECISIONS.md`

**Зависит от:** Task 1 (версии тулов), Task 2 (имена), Task 3 (payload'ы → ясность по формату settings).

- [ ] **Шаг 7.1: создать DECISIONS.md с шапкой**

```markdown
# Decisions

> Append-only журнал решений по cchud. Каждая запись: дата, контекст, решение, последствия. Не переписывать историю — только добавлять новые записи поверх.

---

## 2026-04-26 — Окружение (Phase 0 / Task 1)

- rustc: <версия>
- cargo: <версия>
- node: <версия>
- hyperfine: <версия>
- gh: <версия>
- CPU: <вывод sysctl>

**Последствия:** CI matrix фазы 1 берёт macOS-runner с этими версиями как baseline.

---

## 2026-04-26 — OQ-6: имя `cchud` (Phase 0 / Task 2)

- crates.io: <свободно/занято + детали>
- npm: <свободно/занято + детали>
- GitHub `igorfonin/cchud`: <свободно/занято>
- GitHub `igorfonin/homebrew-tap`: <свободно/занято>

**Решение:** <использовать `cchud` | переключиться на `<fallback>`>.
**Последствия:** <если fallback — что нужно поправить в PRD/README/install.sh>.

---

## 2026-04-26 — OQ-1: формат settings.json (Phase 0 / Task 3)

**Контекст:** проанализировано <N> payload'ов и текущая структура `~/.claude/settings.json` ccstatusline-юзера.

**Наблюдения:**
- ccstatusline хранит конфиг в <месте/секции>: <короткий пример>
- Конфликт с другими ключами settings.json: <есть/нет>
- Версионирование конфига в ccstatusline: <как сделано>

**Решение:** <один из>:
1. **Общая секция `statusLine.config` с дискриминатором `kind: "cchud"`** — бесшовный switch для пользователя.
2. **Своя секция `cchud`** — изоляция, но требует `cchud import` для миграции.

**Обоснование:** <1–2 предложения>.
**Последствия:** Фаза 2 пишет `serde`-структуру под выбранный путь; `cchud import` (фаза 3) маппит ccstatusline → cchud.

---

## 2026-04-26 — Фактическая версия ccstatusline (Phase 0 / Task 3)

- Запрошено в PRD: 2.2.8
- Установлено: <фактически>
- **Последствия:** <обновить `upstream_parity_target` в PRD после фазы 0 | оставить как есть>

---
```

(Заполнить значения по результатам Task 1–3.)

- [ ] **Шаг 7.2: учесть «опциональные» OQ, на которые есть ответ**

Если в ходе Task 3 стало ясно что-то про OQ-2 (npm-конфликт), OQ-4 (`gh auth token`), OQ-5 (`tui` feature-flag) — добавить отдельные блоки. Если не стало — **не выдумывать**, оставить на фазы, где они закрываются.

- [ ] **Шаг 7.3: верификация**

```bash
grep -c "^## 2026-" /Users/igor/mp/startup/cchud/docs/DECISIONS.md
```

Ожидаемо: ≥ 4 (окружение + OQ-6 + OQ-1 + версия ccstatusline). Если меньше — какие-то блоки забыты.

---

## Task 8: Проверка Exit Criteria + обновление статуса

**Цель:** убедиться, что все 6 Exit Criteria из `phase-0-checks.md` выполнены, и отметить фазу 0 как сделанную в `plan/README.md`.

**Файлы:**
- Modify: `plan/README.md` — отметить чекбокс
- Cleanup: `/tmp/cchud-upstream/`, `~/.claude/settings.json.cchud-bak-*`

- [ ] **Шаг 8.1: чек-лист Exit Criteria**

Один за другим (каждый — `echo` + проверка):

```bash
# 1. Имена cchud свободны (или fallback в DECISIONS.md)
grep -A2 "OQ-6" /Users/igor/mp/startup/cchud/docs/DECISIONS.md | head -10

# 2. ≥ 10 разных payload-семплов в benches/samples/
ls /Users/igor/mp/startup/cchud/benches/samples/ | wc -l

# 3. benches/baseline.md с цифрами
test -s /Users/igor/mp/startup/cchud/benches/baseline.md && echo "OK baseline.md exists & non-empty"

# 4. docs/upstream-map.md с конспектом
test -s /Users/igor/mp/startup/cchud/docs/upstream-map.md && echo "OK upstream-map.md exists & non-empty"

# 5. docs/widgets.md с приоритезированным списком 60+
grep -c "^| [A-Z]" /Users/igor/mp/startup/cchud/docs/widgets.md

# 6. docs/DECISIONS.md с OQ-1 и OQ-6
grep -E "OQ-1|OQ-6" /Users/igor/mp/startup/cchud/docs/DECISIONS.md
```

Ожидаемо:
1. блок про OQ-6 присутствует
2. число ≥ 10
3. `OK baseline.md exists & non-empty`
4. `OK upstream-map.md exists & non-empty`
5. число ≥ 60
6. обе строки видны

Любой `FAIL` — вернуться в соответствующую Task и закрыть.

- [ ] **Шаг 8.2: обновить `plan/README.md`**

В строке `- [ ] Фаза 0 — Проверки` поменять на `- [x] Фаза 0 — Проверки`.

- [ ] **Шаг 8.3: cleanup**

```bash
rm -rf /tmp/cchud-upstream
ls ~/.claude/settings.json.cchud-bak-* 2>&1 | head -3
# если бэкап(ы) больше не нужны — удалить, иначе оставить с переименованием в архив:
# mkdir -p ~/.cache/cchud-dev/backups && mv ~/.claude/settings.json.cchud-bak-* ~/.cache/cchud-dev/backups/
```

Сэмплы в `~/.cache/cchud-dev/samples/` оставляем как локальный кэш — пригодятся, если в фазе 2 захочется добавить ещё payload-сценариев.

- [ ] **Шаг 8.4: финальный отчёт пользователю**

В конце сессии выписать:
- Фактические числа cold-start (npx + global), RAM, размер sample
- Решение по имени cchud (✓ свободно / fallback `<name>`)
- Решение по формату settings.json (общая секция / своя секция)
- Фактическое число виджетов upstream (если ≠ 60 — пометить «обновить PRD §2 после фазы 0»)
- Любые сюрпризы (payload содержит неожиданные поля, ccstatusline сильно изменился, etc.)

Это вход в фазу 1.

---

## Зависимости между задачами

```
Task 1 (env) ──┐
Task 2 (names) ┴──► Task 7 (DECISIONS.md)
                          ▲
Task 3 (payloads) ────────┤
       │
       ├──► Task 4 (baseline)
       │
       └──► Task 5 (upstream-map) ──► Task 6 (widgets)
                                          │
                                          ▼
                                   Task 7 (DECISIONS.md)
                                          │
                                          ▼
                                   Task 8 (exit criteria)
```

Параллелизация: Task 4, Task 5 могут идти одновременно после Task 3. Task 6 зависит от Task 5.

## Что не делаем в этой фазе

- `git init`, `cargo init`, `Cargo.toml` — фаза 1.
- Любой Rust-код — фаза 1+.
- Правка `docs/prd-cchud.md` — даже если выясним, что виджетов 58 или ccstatusline = 2.3.0. Просто фиксируем расхождение в DECISIONS.md и поднимаем в фазе 1 как `pre-flight delta`.
- TUI-конфигуратор, Powerline-рендер, любая дистрибуция — поздние фазы.

## Риски / что может пойти не так

| Риск | Вероятность | Митигация |
|---|---|---|
| `ccstatusline@2.2.8` устарел/yanked | средняя | Шаг 3a.1 fallback на `@latest`, фиксируем фактическую версию |
| Имя `cchud` занято | низкая | Шаг 2.5 fallback'и; PRD уже перечисляет `cchud-cli`/`claude-hud`/`ccline` |
| Пользователь забыл сделать бэкап settings.json | низкая | Шаг 3a.4 — `cp` обязательный, без него Шаг 3a.6 не делать |
| Payload Claude Code сильно отличается от ожиданий | средняя | Не фейлим фазу — фиксируем фактическую структуру в `upstream-map.md` и DECISIONS.md, фаза 2 подстроится |
| Upstream-репо переименован/перенесён | низкая | Шаг 5.1 берёт URL из `npm view`, актуальный |
| Семплы содержат секреты | очень низкая | Шаг 3b.6 — grep на типовые паттерны |
| Trace-скрипт ломает Claude Code сессию | низкая | `tee` пропускает stdin как есть; ccstatusline в pipeline сохраняет UX |
