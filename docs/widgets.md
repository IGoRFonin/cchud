# Реестр виджетов cchud (паритет с ccstatusline 2.2.8)

> **Источник:** `src/widgets/index.ts` upstream snapshot (см. `upstream-map.md`)
> **Всего:** 60 (59 экспортируемых + встроенный `separator`)
> **Дата:** 2026-04-26

## Колонки

- **Name** — тип виджета (строка `type` в настройках)
- **Phase** — на какой фазе cchud добавляется
- **Source** — откуда данные: `payload` | `git` | `transcript` | `env` | `http` | `static`
- **Complexity** — `low` | `mid` | `high`
- **Status** — `TODO` | `WIP` | `DONE` (обновляется в фазах)

## Виджеты

| Name | Phase | Source | Complexity | Status | Note |
|---|---|---|---|---|---|
| `model` | 2 | payload | low | TODO | sentinel — первый виджет |
| `separator` | 2 | static | low | TODO | встроенный, не файл-виджет |
| `output-style` | 3 | payload | low | TODO | |
| `context-length` | 3 | payload | low | TODO | |
| `context-percentage` | 3 | payload | low | TODO | |
| `context-percentage-usable` | 3 | payload | mid | TODO | требует model context size table |
| `context-bar` | 3 | payload | low | TODO | ASCII bar из used_percentage |
| `session-clock` | 3 | payload | low | TODO | total_duration_ms |
| `session-cost` | 3 | payload | low | TODO | total_cost_usd |
| `session-name` | 3 | payload | low | TODO | из transcript_path basename |
| `version` | 3 | payload | low | TODO | CC version string |
| `vim-mode` | 3 | payload | low | TODO | context.data.vim.mode |
| `tokens-input` | 3 | payload | low | TODO | current_usage.input_tokens |
| `tokens-output` | 3 | payload | low | TODO | current_usage.output_tokens |
| `thinking-effort` | 3 | payload | mid | TODO | читает transcript файл |
| `worktree` | 3 | payload | low | TODO | data.worktree объект |
| `worktree-mode` | 3 | payload | low | TODO | data.worktree != null |
| `worktree-name` | 3 | payload | low | TODO | data.worktree.name |
| `worktree-branch` | 3 | payload | low | TODO | data.worktree.branch |
| `worktree-original-branch` | 3 | payload | low | TODO | data.worktree.original_branch |
| `claude-session-id` | 3 | payload | low | TODO | data.session_id |
| `terminal-width` | 3 | env | low | TODO | terminalWidth / COLUMNS env |
| `custom-text` | 3 | static | low | TODO | item.customText |
| `custom-symbol` | 3 | static | low | TODO | item.customSymbol |
| `custom-command` | 3 | static | mid | TODO | exec shell command |
| `link` | 3 | static | low | TODO | hyperlink escape seq |
| `git-branch` | 5 | git | mid | TODO | gix: HEAD branch name |
| `git-changes` | 5 | git | mid | TODO | gix: changed file count |
| `git-insertions` | 5 | git | mid | TODO | gix: diff insertions |
| `git-deletions` | 5 | git | mid | TODO | gix: diff deletions |
| `git-root-dir` | 5 | git | mid | TODO | gix: repo root path |
| `git-status` | 5 | git | mid | TODO | gix: status summary |
| `git-staged` | 5 | git | mid | TODO | gix: staged file count |
| `git-unstaged` | 5 | git | mid | TODO | gix: unstaged changes |
| `git-untracked` | 5 | git | mid | TODO | gix: untracked files |
| `git-ahead-behind` | 5 | git | mid | TODO | gix: commits ahead/behind upstream |
| `git-conflicts` | 5 | git | mid | TODO | gix: merge conflicts |
| `git-sha` | 5 | git | mid | TODO | gix: HEAD SHA |
| `git-origin-owner` | 5 | git | mid | TODO | git-remote.ts: parse origin URL |
| `git-origin-repo` | 5 | git | mid | TODO | git-remote.ts |
| `git-origin-owner-repo` | 5 | git | mid | TODO | git-remote.ts |
| `git-upstream-owner` | 5 | git | mid | TODO | git-remote.ts: upstream remote |
| `git-upstream-repo` | 5 | git | mid | TODO | git-remote.ts |
| `git-upstream-owner-repo` | 5 | git | mid | TODO | git-remote.ts |
| `git-is-fork` | 5 | git | mid | TODO | origin ≠ upstream owner |
| `git-pr` | 5 | http | high | TODO | fetchGitReviewData — GitHub API |
| `tokens-cached` | 6 | transcript | mid | TODO | tokenMetrics из JSONL |
| `tokens-total` | 6 | transcript | mid | TODO | tokenMetrics |
| `input-speed` | 6 | transcript | mid | TODO | speedMetrics.input |
| `output-speed` | 6 | transcript | mid | TODO | speedMetrics.output |
| `total-speed` | 6 | transcript | mid | TODO | speedMetrics.total |
| `block-timer` | 6 | transcript | mid | TODO | blockMetrics из jsonl-cache |
| `session-duration` | 6 | transcript | mid | TODO | sessionDuration string |
| `skills` | 6 | transcript | high | TODO | skillsMetrics из transcript |
| `claude-account-email` | 7 | env | mid | TODO | читает ~/.claude.json |
| `free-memory` | 7 | env | mid | TODO | os.freemem() + macOS sysctl |
| `session-usage` | 7 | http | high | TODO | usageData HTTP API |
| `weekly-usage` | 7 | http | high | TODO | usageData HTTP API |
| `block-reset-timer` | 7 | http | high | TODO | usageData + blockMetrics |
| `weekly-reset-timer` | 7 | http | high | TODO | usageData |

## Сводка по фазам

| Фаза | Виджеты | Кумулятивно |
|---|---|---|
| 2 | 2 (model, separator) | 2 |
| 3 | 24 (payload + env/static) | 26 |
| 4 | 0 (рендер-слой, не виджеты) | 26 |
| 5 | 20 (git + git-pr) | 46 |
| 6 | 8 (transcript) | 54 |
| 7 | 6 (env/http) | 60 |

## Замечания

- **`separator`** — специальный тип, обрабатывается в `ccstatusline.ts` напрямую, не через Widget interface. В cchud — аналогично, встроенная логика рендера.
- **`git-pr`** — единственный git-виджет с HTTP; помечен high, может быть отложен в фазу 7.
- **`thinking-effort`** — хотя источник payload, он дополнительно читает transcript файл (mid сложность).
- **`block-timer`** — использует `context.blockMetrics`, которые вычисляются из transcript через jsonl-cache (фаза 6).
- **Счёт upstream:** index.ts содержит 59 `export` строк → 59 классов виджетов. С встроенным `separator` = **60** типов. PRD §2 говорит «60+» — соответствует.
