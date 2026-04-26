# Decisions

> Append-only журнал решений по cchud. Каждая запись: дата, контекст, решение, последствия. Не переписывать историю — только добавлять новые записи поверх.

---

## 2026-04-26 — Окружение (Phase 0 / Task 1)

- rustc: 1.87.0 (Homebrew)
- cargo: 1.87.0 (Homebrew)
- node: v24.13.0 (nvm)
- hyperfine: 1.20.0 (установлен в ходе фазы 0, не было)
- gh: 2.88.1
- CPU: Apple M4 Pro / 24 GB RAM
- OS: macOS 15.6.1

**Последствия:** CI matrix фазы 1 берёт macOS-runner с этими версиями как baseline. hyperfine нужно добавить в dev-зависимости / CI setup step.

---

## 2026-04-26 — OQ-6: имя `cchud` (Phase 0 / Task 2)

- crates.io: **свободно** (HTTP 404)
- npm: **свободно** (HTTP 404)
- GitHub `IGoRFonin/cchud`: **свободно** (репо не существует)
- GitHub `IGoRFonin/homebrew-tap`: **свободно** (tap не создан — норм, создаётся в фазе 9)

**Решение:** использовать имя `cchud` без fallback.

**Последствия:** имя закреплено, PRD §0.2 закрыт. Репо создаётся в фазе 1 (`gh repo create IGoRFonin/cchud`).

---

## 2026-04-26 — OQ-1: формат settings.json (Phase 0 / Task 3 + Task 5)

**Контекст:** изучена структура `~/.claude/settings.json` (payload наблюдения) и исходный код ccstatusline `src/utils/config.ts`.

**Наблюдение (сюрприз):** ccstatusline хранит свой конфиг **не** в `~/.claude/settings.json`, а в отдельном файле:
```
~/.config/ccstatusline/settings.json
```
`~/.claude/settings.json` используется только для ключа `statusLine.command` (запуск бинаря), а собственные настройки (lines, flexMode, colorLevel, etc.) — отдельный файл.

**Структура `~/.claude/settings.json` (наш settings.json):**
```json
{
  "statusLine": {
    "type": "command",
    "command": "~/.claude/statusline.sh",
    "padding": 0
  }
}
```
Конфликт с другими ключами `~/.claude/settings.json`: **нет** — ccstatusline не трогает этот файл вовсе (кроме `statusLine.command`).

Версионирование конфига в ccstatusline: `CURRENT_VERSION = 3`, поле `version` в JSON, chain-миграции v1→v2→v3.

**Решение:** cchud хранит конфиг в **`~/.config/cchud/settings.json`** — аналогично upstream. Изоляция от других ключей `~/.claude/settings.json`, понятный путь миграции (ccstatusline → cchud via `cchud import`).

**Обоснование:** upstream уже решил эту задачу именно так. Своя секция в `~/.claude/settings.json` создаст coupling с форматом Claude Code, который может измениться. Отдельный файл = portable конфиг.

**Последствия:** Фаза 2 пишет `serde`-структуру под путь `~/.config/cchud/settings.json`. `cchud import` (фаза 3) читает `~/.config/ccstatusline/settings.json` и пишет в `~/.config/cchud/settings.json`.

---

## 2026-04-26 — Фактическая версия ccstatusline (Phase 0 / Task 3a)

- Запрошено в PRD: 2.2.8
- На npm: 2.2.8 (latest = 2.2.8 — совпадает)
- Установлено: 2.2.8
- Репозиторий: `sirmalloc/ccstatusline` (без тегов по версии; HEAD main = 2.2.8)

**Последствия:** `upstream_parity_target = 2.2.8` в PRD актуален, обновлять не нужно.

---

## 2026-04-26 — Baseline холодного старта ccstatusline (Phase 0 / Task 4)

**Контекст:** PRD §1 ожидал npx ~50–150 мс, global — «заметно быстрее, десятки мс».

**Фактические данные (Apple M4 Pro, node v24.13.0, ccstatusline 2.2.8):**

| Режим | Mean | Min | Max |
|---|---|---|---|
| `npx -y ccstatusline@latest` | **827.7 мс** | 792.5 мс | 933.0 мс |
| `ccstatusline` (global) | **246.7 мс** | 235.3 мс | 321.0 мс |

RAM (global): RSS пик 97.7 МБ, peak memory footprint 42.2 МБ.

**Анализ:** PRD §1 недооценил время примерно в 5–16× для npx и 5–50× для global. Вероятные причины: node.js cold-start (~150–200 мс на M4 Pro с nvm), загрузка JSONL-транскрипта при каждом вызове.

**Решение:** зафиксировать фактический baseline, обновить PRD §1 в фазе 1 как `pre-flight delta`. Цели cchud (< 5 мс, < 5 МБ) остаются — запас ×49 по времени и ×20 по RAM от текущего baseline, что реалистично для Rust.

**Последствия:** README-таблица в фазе 9 будет показывать реальный baseline (не PRD-ожидание). PRD §1 нужно пересмотреть в фазе 1.

---

## 2026-04-26 — Количество виджетов upstream (Phase 0 / Task 6)

- PRD §2 говорит «60+ виджетов»
- Фактически: 59 экспортируемых виджетов в `src/widgets/index.ts` + 1 встроенный `separator` = **60**
- PRD формулировка «60+» технически некорректна (ровно 60), но не критично

**Решение:** не обновлять PRD сейчас. Зафиксировать точное число 60 в `docs/widgets.md`. В фазе 1 упомянуть как pre-flight delta если потребуется.

**Последствия:** `docs/widgets.md` — canonical источник списка, 60 строк.
