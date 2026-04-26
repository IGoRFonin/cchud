---
title: "ccstatusline: анализ ресурсов, безопасности и альтернатив"
date: 2026-04-25
author: research compilation
repo: https://github.com/sirmalloc/ccstatusline
version_analyzed: 2.2.8
---

# ccstatusline — research compilation

## TL;DR для аналитика

- **ccstatusline** — самая популярная statusline-утилита для Claude Code CLI (8.3k⭐, npm-пакет, написан на TypeScript/React+Ink, бандл 3 МБ).
- **Сама утилита нормальная**, но **дефолтная конфигурация из README (`npx -y ccstatusline@latest`) даёт два больших минуса:**
  1. **Ресурсы:** 50–150 мс CPU и 50–100 МБ RAM на каждый рендер, который происходит **до 3 раз в секунду** (Claude Code дёргает statusline раз в 300 мс).
  2. **Безопасность:** `@latest` подтягивает любую новую версию мгновенно → постоянная supply-chain поверхность.
- **Решение для обеих проблем:** глобальная установка с пином версии (`npm i -g ccstatusline@2.2.8`) и правка `settings.json` на прямой вызов `ccstatusline`. Экономит ~50–100 мс/рендер и закрывает supply-chain.
- **Радикальные альтернативы (по росту производительности):**
  - bash+jq (`claude-lens`, `mini-ccstatus`): ~10–15 мс/рендер, ~2–5 МБ RAM
  - Гипотетический Rust-порт: ~1–5 мс/рендер, ~1–3 МБ RAM
  - Node остаётся самым медленным из-за V8 cold start, который нельзя «прогреть» — процесс живёт <150 мс и умирает.

---

## 1. Факты по проекту

| Параметр | Значение |
|---|---|
| Repo | sirmalloc/ccstatusline |
| Stars | 8 296 |
| Forks | 363 |
| License | MIT |
| Author | Matthew Breedlove (@sirmalloc) |
| Текущая версия | 2.2.8 |
| Размер npm пакета (unpacked) | ~3 МБ (4 файла, single bundled JS) |
| Stack | TypeScript, React, Ink (TUI), zod, https-proxy-agent |
| Build target | Node 14+, single bundled file (через bun build) |
| Виджетов | 30+ (git, usage API, block timer, vim mode, thinking effort, cost, и т.д.) |

### Конфигурация по умолчанию (README)
```json
{
  "statusLine": {
    "type": "command",
    "command": "npx -y ccstatusline@latest",
    "padding": 0
  }
}
```

---

## 2. Проблема №1 — ресурсы

### Контекст
Claude Code вызывает команду statusline **до 3 раз в секунду** (раз в 300 мс) во время активной сессии — на каждое обновление токенов, контекста, состояния.

### Что делает `npx -y ccstatusline@latest` на каждый вызов

| Шаг | Время |
|---|---|
| npx: проверка кэша `~/.npm/_npx/` | ~5–10 мс |
| npx: HTTP-запрос на registry.npmjs.org для проверки `@latest` | сетевая задержка, кэшируется ETag |
| `fork() + exec(node)` | ~5–10 мс |
| V8 cold start (инициализация рантайма) | ~30–80 мс |
| Загрузка bundle 3 МБ, parse JS | ~10–20 мс |
| Чтение settings.json, transcript (с кэшем по mtime после v2.0.28), git-команды | ~5–20 мс |
| Рендер в stdout, exit | ~5–10 мс |
| **Итого** | **~50–150 мс CPU, 50–100 МБ RAM пик** |

### Известные баги по ресурсам (issues)

| # | Статус | Суть |
|---|---|---|
| [#137](https://github.com/sirmalloc/ccstatusline/issues/137) | ✅ closed 2026-02-21 | Парсил весь transcript.jsonl на каждом рендере → 60–80% CPU. Профилирование (`sample` macOS) показало `v8::internal::Builtin_JsonParse` доминирующим. Фиксано хэш-кэшем block timer и fs.stat-инвалидацией в v2.0.28. |
| [#22](https://github.com/sirmalloc/ccstatusline/issues/22) | ✅ closed | CPU runaway. |
| [#73](https://github.com/sirmalloc/ccstatusline/issues/73) | ✅ closed | Claude Code freeze. |
| [#103](https://github.com/sirmalloc/ccstatusline/issues/103) | ⚠️ **open** (с 2025-10-23) | Бенчмарк показал, что `npx/bunx` сами по себе — основной источник оверхеда vs прямой вызов бинаря. Альтернативы: `claude-lens` (~10 мс / 2 МБ RSS), `mini-ccstatus`. |

### Влияние на параллельные сессии

При 4 одновременных Claude Code сессиях (типичный split-panel workflow):

| Подход | CPU фон | RAM фон |
|---|---|---|
| ccstatusline через npx | до **120%** одного ядра, пики ~400 МБ | ощутимо |
| ccstatusline global | ~30–60% одного ядра, ~200 МБ | заметно |
| bash+jq | ~15% | <20 МБ |
| Гипотетический Rust | ~2–4% | <10 МБ |

На MacBook Pro (M-серия) разница между Node и bash/Rust **слышна по вентилятору** при долгих сессиях; на батарее заметна по power budget.

---

## 3. Проблема №2 — supply-chain безопасность

### Угроза
[Issue #298 (open)](https://github.com/sirmalloc/ccstatusline/issues/298): команда `npx -y ccstatusline@latest` каждые 300 мс эффективно проверяет наличие новой версии. Любая компрометация:
- npm-аккаунта автора (`sirmalloc`)
- любой transitive dependency
- malicious update

→ **в течение 300 мс попадает в исполнение** в пользовательской сессии с доступом к:
- `~/.ssh/`
- `~/.aws/credentials`
- `~/.claude.json` (содержит Anthropic API-ключ)
- любым переменным окружения текущего пользователя

### Поверхность атаки
- 8 296 звёзд → высокоценный таргет для supply-chain атак
- `@latest` обходит любые механизмы pin/lock
- `npm audit` не помогает: он смотрит на published vulnerabilities, а свежая компрометация ещё не попадёт в базу
- Автор — solo maintainer, нет 2-person review при релизах

---

## 4. Решения

### Решение А — глобальная установка с пином (рекомендуемое)

```bash
npm i -g ccstatusline@2.2.8
```

`~/.claude/settings.json`:
```json
{
  "statusLine": {
    "type": "command",
    "command": "ccstatusline",
    "padding": 0
  }
}
```

**Закрывает:**
- ✅ npx-резолв и HTTP-запросы (~50–150 мс/рендер экономии)
- ✅ supply-chain: версия зафиксирована до явного `npm update`
- ❌ Не убирает V8 cold start (~30–80 мс остаётся)

**Maintenance:** раз в 4–8 недель проверять `gh issue list -R sirmalloc/ccstatusline --state open` и changelog, обновлять вручную пином.

### Решение Б — фиксированная версия через npx

```json
{ "command": "npx -y ccstatusline@2.2.8" }
```

Версия запинена → npx после первого запуска кэширует в `~/.npm/_npx/<hash>/`, повторных HTTP-запросов нет. Spawn node-процесса остаётся.

### Решение В — самостоятельная сборка из git

```bash
git clone https://github.com/sirmalloc/ccstatusline ~/tools/ccstatusline
cd ~/tools/ccstatusline
git checkout v2.2.8
bun install && bun run build
```

```json
{ "command": "/Users/<user>/tools/ccstatusline/dist/ccstatusline.js" }
```

Параноидальный режим: можно ревьюить diff перед каждой пересборкой.

### Решение Г — отказаться от Node-варианта вовсе

Альтернативы на bash:
- [claude-lens](https://github.com/Astro-Han/claude-lens) — ~10 мс / 2 МБ RSS, нет TUI и тем
- mini-ccstatus — минималистичный

---

## 5. Сравнение технологий — детальный бенчмарк

### Старт + работа на каждый рендер

| Подход | Старт рантайма | Полезная работа | Итого CPU/рендер | RAM пик |
|---|---|---|---|---|
| Node (ccstatusline через npx) | ~80–150 мс (npx + V8) | ~10–30 мс | **100–150 мс** | 50–100 МБ |
| Node (ccstatusline global) | ~30–80 мс (V8 cold) | ~10–30 мс | **50–100 мс** | 50–100 МБ |
| Bash + jq (4 утилиты в pipeline) | ~2–5 мс | ~5–10 мс fork'ов | **10–15 мс** | 2–5 МБ |
| Bash builtins-only (regex) | ~2–5 мс | ~1–3 мс | **3–5 мс** | <2 МБ |
| Rust (статически слинкован) | ~0.5–2 мс | ~1–3 мс | **1–5 мс** | 1–3 МБ |

### Почему такая иерархия

| Фактор | Node | Bash+jq | Rust |
|---|---|---|---|
| Cold start рантайма | V8 init heavy (~30–80 мс) | bash ~2–5 мс | mmap+jump ~0.5 мс |
| JIT-оптимизации | V8 хорош в JIT, но процесс умирает раньше прогрева — оптимизации не успевают применяться | нет JIT, не нужен | ahead-of-time compile |
| Парсинг JSON | `JSON.parse` в холодном V8 | `jq` отдельным процессом, parse быстрый | `serde_json`/`simd-json` — функция в том же процессе |
| Pipeline | один процесс, всё внутри | **N fork+exec** (по одному на утилиту) | один процесс, всё внутри |
| Память | V8 heap минимум 30–50 МБ | bash ~1 МБ + jq ~3 МБ | только то, что аллоцировано |

### Ключевой инсайт по Node
**V8 нельзя «прогреть» в коротко-живущем процессе.** Все оптимизации V8 (TurboFan, inline caches, hidden classes) рассчитаны на долгоживущие процессы. Когда процесс живёт 50–150 мс и умирает — это **худший сценарий для Node**. Поэтому даже хорошо написанный TypeScript-код для statusline принципиально не может конкурировать с bash или Rust.

### Ключевой инсайт по bash
Главный источник оверхеда в bash — **fork+exec на каждую внешнюю утилиту** (jq, git, awk, cut). Типичный statusline-скрипт делает 3–5 таких вызовов = 5–15 мс только на спавн. Скрипты на чистых bash builtins (regex через `[[ $x =~ ... ]]`) приближаются к Rust по скорости, но JSON через regex — хрупко и больно.

### Аналогии в индустрии
- [`starship`](https://starship.rs/) — кросс-шелл prompt на Rust, ~5–20 мс/рендер
- `oh-my-posh` — Go-аналог, ~10–30 мс/рендер
- Оба доказывают: компилируемые языки в нише statusline на порядок быстрее Node.

---

## 6. Гипотеза: ccstatusline на Rust

Если бы кто-то портировал ccstatusline на Rust по образцу `starship`:

| Метрика | Текущий Node | Гипотетический Rust |
|---|---|---|
| CPU/рендер | 50–150 мс | **1–5 мс** (30–100×) |
| RAM на процесс | 50–100 МБ | **1–3 МБ** (~30×) |
| 4 параллельных сессии | до 120% CPU + 400 МБ | **2–4% CPU + 10 МБ** |
| Размер бинаря | 3 МБ JS + 80 МБ node | **2–5 МБ статический бинарь** |

### Что было бы непросто
- TUI-конфигуратор (сейчас на React+Ink) → переписывать на `ratatui`/`crossterm`. Реалистично — `gitui`, `bottom`, `helix` доказывают.
- Powerline Unicode-рендер — `unicode-width`, `unicode-segmentation` есть из коробки.
- 30+ виджетов — портируются один-в-один, git вызывается через `std::process::Command`.
- Кросс-платформенность Windows — Rust здесь даже лучше Node.

**Объём:** ~10–15k строк TS → примерно столько же Rust. Реалистично для одного maintainer'а за 1–3 месяца. Open-source ниша свободна.

---

## 7. Рекомендации по сегментам пользователей

### Casual user (1 сессия, любит TUI и темы)
→ `npm i -g ccstatusline@<pinned>`. Потеря 30% одного ядра не критична на M-чипе, удобство TUI оправдано.

### Power user (split-panel, несколько сессий, длинные рабочие дни)
→ `claude-lens` или `mini-ccstatus` на bash. Экономия CPU/батареи существенна на длинной дистанции, фичи минимальны но обычно достаточны.

### Параноик по безопасности
→ Самостоятельная сборка из git с фиксацией коммита, либо bash-скрипт (~50 строк, читаешь сам = доверие 100%).

### Будущее
→ Rust-порт ccstatusline закрыл бы все три ниши одновременно: фичи Node-версии, скорость Rust, отсутствие supply-chain через npm `@latest`.

---

## 8. Источники и evidence

- README ccstatusline: https://github.com/sirmalloc/ccstatusline (v2.2.8 changelog)
- npm package metadata: `npm view ccstatusline` — unpacked 2 975 569 байт, 4 файла
- Issue #103 (open) — бенчмарк npx vs direct call
- Issue #137 (closed) — профилирование `sample` macOS, V8 JsonParse доминирует
- Issue #298 (open) — supply-chain угроза `@latest`
- Claude Code documentation: statusline command вызывается раз в 300 мс
- Аналоги: starship.rs, oh-my-posh, claude-lens (Astro-Han), mini-ccstatus

---

## 9. Открытые вопросы для дальнейшего ресерча

1. **Реальные замеры на типичной нагрузке.** Все цифры выше — оценки на базе issues и общих знаний о V8/bash/Rust. Стоит сделать собственный бенчмарк: hyperfine на 1000 итераций для каждого варианта.
2. **Влияние конкретных виджетов.** Git PR, Usage API (HTTP к Anthropic), Block Timer — какой вклад каждого в общее время рендера?
3. **Behavior при offline.** Как ведут себя `@latest`/usage widgets без интернета? Не блокируется ли Claude Code?
4. **Альтернативы на Go.** `oh-my-posh`-подобный подход на Go — компромисс между Rust-скоростью и Node-эргономикой.
5. **Persistent statusline daemon.** Ссылка из issue #103 на anthropics/claude-code#10162 — есть feature request на persistent process. Если реализуют, Node-вариант перестанет страдать от cold start.
