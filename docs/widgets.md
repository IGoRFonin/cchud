# Реестр виджетов cchud (паритет с ccstatusline 2.2.8 + расширения)

> **Источник:** `src/widgets/index.ts` upstream snapshot (см. `upstream-map.md`)
> **Всего:** 61 (60 паритетных + `current-working-dir` из upstream 2.2.x)
> **Дата:** 2026-05-07
>
> **61 widgets supported**

## Колонки

- **Name** — тип виджета (строка `type` в настройках)
- **Phase** — на какой фазе cchud добавляется
- **Source** — откуда данные: `payload` | `git` | `transcript` | `env` | `http` | `static`
- **Complexity** — `low` | `mid` | `high`
- **Status** — `TODO` | `WIP` | `DONE` (обновляется в фазах)

## Виджеты

| Name | Phase | Source | Complexity | Status | Note |
|---|---|---|---|---|---|
| `model` | 2 | payload | low | DONE | sentinel — первый виджет |
| `separator` | 2 | static | low | TODO | встроенный, не файл-виджет |
| `output-style` | 3 | payload | low | DONE | |
| `context-length` | 3 | payload | low | DONE | |
| `context-percentage` | 3 | payload | low | DONE | |
| `context-percentage-usable` | 3 | payload | mid | DONE | требует model context size table |
| `context-bar` | 3 | payload | low | DONE | ASCII bar из used_percentage |
| `session-clock` | 3 | payload | low | DONE | total_duration_ms |
| `session-cost` | 3 | payload | low | DONE | total_cost_usd |
| `session-name` | 3 | payload | low | DONE | из transcript_path basename |
| `version` | 3 | payload | low | DONE | CC version string |
| `vim-mode` | 3 | payload | low | DONE | context.data.vim.mode |
| `tokens-input` | 3 | payload | low | DONE | current_usage.input_tokens |
| `tokens-output` | 3 | payload | low | DONE | current_usage.output_tokens |
| `thinking-effort` | 6 | transcript | mid | DONE | 🧠 {level} из последнего assistant |
| `cache-misses` | — | transcript | low | DONE | счётчик пауз > 300s, сбрасывается на /clear |
| `worktree` | 3 | payload | low | DONE | data.worktree объект |
| `worktree-mode` | 3 | payload | low | DONE | data.worktree != null |
| `worktree-name` | 3 | payload | low | DONE | data.worktree.name |
| `worktree-branch` | 3 | payload | low | DONE | data.worktree.branch |
| `worktree-original-branch` | 3 | payload | low | DONE | data.worktree.original_branch |
| `claude-session-id` | 3 | payload | low | DONE | data.session_id |
| `terminal-width` | 3 | env | low | DONE | terminalWidth / COLUMNS env |
| `custom-text` | 3 | static | low | DONE | item.customText |
| `custom-symbol` | 3 | static | low | DONE | item.customSymbol |
| `custom-command` | 3 | static | mid | DONE | exec shell command |
| `link` | 3 | static | low | DONE | hyperlink escape seq |
| `git-branch` | 5 | git | mid | DONE | gix: HEAD branch name |
| `git-changes` | 5 | git | mid | DONE | gix: changed file count |
| `git-insertions` | 5 | git | mid | DONE | gix: diff insertions |
| `git-deletions` | 5 | git | mid | DONE | gix: diff deletions |
| `git-root-dir` | 5 | git | mid | DONE | gix: repo root path |
| `git-status` | 5 | git | mid | DONE | gix: status summary |
| `git-staged` | 5 | git | mid | DONE | gix: staged file count |
| `git-unstaged` | 5 | git | mid | DONE | gix: unstaged changes |
| `git-untracked` | 5 | git | mid | DONE | gix: untracked files |
| `git-ahead-behind` | 5 | git | mid | DONE | gix: commits ahead/behind upstream |
| `git-conflicts` | 5 | git | mid | DONE | gix: merge conflicts |
| `git-sha` | 5 | git | mid | DONE | gix: HEAD SHA |
| `git-origin-owner` | 5 | git | mid | DONE | hand-parser: parse origin URL |
| `git-origin-repo` | 5 | git | mid | DONE | hand-parser |
| `git-origin-owner-repo` | 5 | git | mid | DONE | hand-parser |
| `git-upstream-owner` | 5 | git | mid | DONE | hand-parser: upstream remote |
| `git-upstream-repo` | 5 | git | mid | DONE | hand-parser |
| `git-upstream-owner-repo` | 5 | git | mid | DONE | hand-parser |
| `git-is-fork` | 5 | git | mid | DONE | origin ≠ upstream owner |
| `git-pr` | 5 | http | high | DONE | GitHub API, disk-cached, offline soft-fail |
| `tokens-cached` | 6 | transcript | mid | DONE | cT: <fmt> сумма cache_read + cache_creation |
| `tokens-total` | 6 | transcript | mid | DONE | totT: <fmt> сумма всех 4 групп |
| `input-speed` | 6 | transcript | mid | DONE | ↓N t/s от последнего assistant |
| `output-speed` | 6 | transcript | mid | DONE | ↑N t/s от последнего assistant |
| `total-speed` | 6 | transcript | mid | DONE | ⇅N t/s от последнего assistant |
| `block-timer` | 6 | transcript | mid | DONE | ⏰ HH:MM:SS time-to-end billing-блока |
| `session-duration` | 6 | transcript | mid | DONE | диапазон last_msg - first_msg |
| `skills` | 7 | transcript | high | DONE | skillsMetrics из transcript |
| `claude-account-email` | 7 | env | mid | DONE | читает ~/.claude.json |
| `free-memory` | 7 | env | mid | DONE | os.freemem() + macOS sysctl |
| `session-usage` | 7 | http | high | DONE | usageData HTTP API |
| `weekly-usage` | 7 | http | high | DONE | usageData HTTP API |
| `block-reset-timer` | 7 | http | high | DONE | usageData + blockMetrics |
| `weekly-reset-timer` | 7 | http | high | DONE | usageData |
| `current-working-dir` | post-1.0 | payload | low | DONE | payload.cwd, fish-style/segments/abbreviate-home/prefix |

## Сводка по фазам

| Фаза | Виджеты | Кумулятивно |
|---|---|---|
| 2 | 2 (model, separator) | 2 |
| 3 | 23 (payload + env/static) | 25 |
| 4 | 0 (рендер-слой, не виджеты) | 25 |
| 5 | 20 (git + git-pr) | 45 |
| 6 | 8 (transcript + thinking-effort; skills → 7) | 53 |
| 7 | 7 (env/http + skills) | 60 |
| post-1.0 | 1 (current-working-dir) | **61** |

## Замечания

- **`separator`** — специальный тип, обрабатывается в `ccstatusline.ts` напрямую, не через Widget interface. В cchud — аналогично, встроенная логика рендера.
- **`git-pr`** — единственный git-виджет с HTTP; помечен high, может быть отложен в фазу 7.
- **`thinking-effort`** — реализован в Phase 6 (читает JSONL transcript, не payload); перемещён в Phase 6 при фактической реализации.
- **`block-timer`** — использует `context.blockMetrics`, которые вычисляются из transcript через jsonl-cache (фаза 6).
- **Счёт upstream:** index.ts содержит 59 `export` строк → 59 классов виджетов. С встроенным `separator` = **60** типов. PRD §2 говорит «60+» — соответствует.
