# Upstream map — ccstatusline → cchud

> **Версия upstream:** 2.2.8 (latest, теги не используются — клонирован HEAD main)
> **Репо:** sirmalloc/ccstatusline
> **Дата снимка:** 2026-04-26
> **Локальный клон:** /tmp/cchud-upstream/ccstatusline (временный, удалить после фазы 0)

## Карта портирования

| Upstream-файл | LOC | Что портируется | Куда в cchud | Фаза |
|---|---|---|---|---|
| `src/types/Settings.ts` | 73 | Схема конфига (v1/v3, поля flexMode/lines/powerline/...) | `src/config/types.rs` (serde-derive) | 2 |
| `src/types/RenderContext.ts` | 44 | Контракт виджетов (data, tokenMetrics, gitData, ...) | `src/widgets/context.rs` | 2 |
| `src/types/StatusJSON.ts` | 77 | Схема payload от Claude Code (zod looseObject) | `src/types/payload.rs` | 2 |
| `src/utils/jsonl-cache.ts` | 133 | block-cache: configDir hash → `~/.cache/ccstatusline/block-cache-<hash>.json` | `src/cache/block.rs` | 6 |
| `src/utils/powerline.ts` | 336 | Powerline font detection + рендер (U+E0B0..E0B3, glyphs, caps) | `src/render/powerline.rs` | 4 |
| `src/utils/migrations.ts` | 220 | Миграции конфига v1→v2→v3 (chain функций, поле `version`) | `src/config/migrations.rs` | 2 |
| `src/widgets/index.ts` | 59 | Реестр 59 виджетов + встроенный separator | `src/widgets/registry.rs` | 2 |
| `src/utils/git.ts` | 177 | Git-обёртки (branch, status, staged, unstaged, untracked, SHA) | `src/git/info.rs` (gix) | 5 |
| `src/utils/git-remote.ts` | 190 | Git remote: owner/repo/fork-detection (origin + upstream) | `src/git/remote.rs` (gix) | 5 |

LOC снят через `wc -l`.

## Конспект ключевых типов

### `StatusJSON` (`src/types/StatusJSON.ts`)

Zod `looseObject` (unknown поля не фейлят парсинг):

```typescript
{
  hook_event_name?: string
  session_id?: string
  transcript_path?: string
  cwd?: string
  model?: string | { id?: string; display_name?: string }
  workspace?: { current_dir?: string; project_dir?: string }
  version?: string                  // CC version e.g. "2.1.119"
  output_style?: { name?: string }
  cost?: {
    total_cost_usd?: number
    total_duration_ms?: number
    total_api_duration_ms?: number
    total_lines_added?: number
    total_lines_removed?: number
  }
  context_window?: {
    context_window_size?: number | null
    total_input_tokens?: number | null
    total_output_tokens?: number | null
    current_usage?: number | {
      input_tokens?: number
      output_tokens?: number
      cache_creation_input_tokens?: number
      cache_read_input_tokens?: number
    } | null
    used_percentage?: number | null
    remaining_percentage?: number | null
  } | null
  vim?: { mode?: string } | null
  worktree?: {
    name?: string; path?: string; branch?: string
    original_cwd?: string; original_branch?: string
  } | null
  rate_limits?: {
    five_hour?: { used_percentage?: number | null; resets_at?: number | null }
    seven_day?: { used_percentage?: number | null; resets_at?: number | null }
  } | null
}
```

**Особенность:** `CoercedNumberSchema` — числа могут прийти строками (предобработка `preprocess`).
Все поля опциональны. Сериализация в Rust: `#[serde(default)]` + custom deserializer для coerced numbers.

### `Settings` (`src/types/Settings.ts`)

Версионированная конфигурация (`CURRENT_VERSION = 3`). Хранится в `~/.claude/settings.json`
под секцией `statusLine.config` (OQ-1 — решается в Task 7).

```typescript
{
  version: number                 // default: 3
  lines: WidgetItem[][]           // 1–3 строки; default: [7 виджетов], [], []
  flexMode: 'full' | 'full-minus-40' | ...   // default: 'full-minus-40'
  compactThreshold: number        // 1–99, default: 60
  colorLevel: number              // default: 2 (256-color)
  defaultSeparator?: string
  defaultPadding?: string
  inheritSeparatorColors: boolean // default: false
  overrideBackgroundColor?: string
  overrideForegroundColor?: string
  globalBold: boolean             // default: false
  minimalistMode: boolean         // default: false
  powerline: PowerlineConfig      // see below
  updatemessage?: { message?: string | null; remaining?: number | null }
}

// PowerlineConfig defaults:
{
  enabled: false
  separators: ['']
  separatorInvertBackground: [false]
  startCaps: []
  endCaps: []
  theme: undefined
  autoAlign: false
  continueThemeAcrossLines: false
}
```

**Версии:**
- v1: без поля `version`, поля flat в `statusLine.config`
- v2: добавлено `version`
- v3: current (+ powerline, minimalistMode)

Миграция: chain функций `migrate_v1_to_v2` → `migrate_v2_to_v3`. Версия определяется по наличию поля `version`.

### `RenderContext` (`src/types/RenderContext.ts`)

```typescript
{
  data?: StatusJSON                   // payload от Claude Code
  tokenMetrics?: TokenMetrics | null  // производные от transcript
  speedMetrics?: SpeedMetrics | null
  windowedSpeedMetrics?: Record<string, SpeedMetrics> | null
  usageData?: RenderUsageData | null  // HTTP: API usage (rate limits)
  sessionDuration?: string | null
  blockMetrics?: BlockMetrics | null  // из jsonl-cache
  skillsMetrics?: SkillsMetrics | null
  terminalWidth?: number | null
  isPreview?: boolean
  minimalist?: boolean
  lineIndex?: number                  // для theme cycling
  globalSeparatorIndex?: number
  gitData?: { changedFiles?: number; insertions?: number; deletions?: number }
  globalPowerlineThemeIndex?: number
}
```

## Алгоритмы

### Block-cache (`src/utils/jsonl-cache.ts`)

- **Ключ кэша:** SHA-256 от `path.resolve(configDir)` → первые 16 hex-символов
- **Файл:** `~/.cache/ccstatusline/block-cache-<hash>.json`
- **Содержимое:** `{ startTime: ISO-string, configDir: string }`
- **Инвалидация:** по времени — если `now > startTime + sessionDurationHours*3600s` → пересчёт через `getBlockMetrics()`
- **Запись:** best-effort (silently fail)

> **Замечание:** `jsonl-cache.ts` — это block-cache (для billing blocks), а не JSONL transcript cache. Название вводит в заблуждение.

### Powerline (`src/utils/powerline.ts`)

- Glyphs: U+E0B0 ``, U+E0B1 ``, U+E0B2 ``, U+E0B3 ``
- Font detection: heuristic по наличию файлов шрифтов в стандартных путях (macOS/Linux/Windows)
- Debug: `DEBUG_FONT_INSTALL=1` → притворяется что шрифты не установлены
- Caps: `startCaps`, `endCaps` из `PowerlineConfig`

### Миграции (`src/utils/migrations.ts`)

- Определение версии: `data.version` (отсутствует → v1, иначе число)
- Chain: `migrate_v1_to_v2 → migrate_v2_to_v3`
- Тип функции: `(data: Record<string, unknown>) => Record<string, unknown>`

## Реестр виджетов (`src/widgets/index.ts`)

59 экспортированных виджетов + встроенный `separator` = **60 типов виджетов**.
Полный список — в `docs/widgets.md` (Task 6).

### Структура `WidgetItem` (общая для всех виджетов)

```typescript
{
  id: string
  type: string           // имя виджета
  color?: string
  backgroundColor?: string
  bold?: boolean
  character?: string
  rawValue?: boolean
  customText?: string
  customSymbol?: string
  commandPath?: string
  maxWidth?: number
  preserveColors?: boolean
  timeout?: number
  merge?: boolean | 'no-padding'
  hide?: boolean
  metadata?: Record<string, string>
}
```

### Интерфейс `Widget`

```typescript
interface Widget {
  getDefaultColor(): string
  getDescription(): string
  getDisplayName(): string
  getCategory(): string
  getEditorDisplay(item: WidgetItem): WidgetEditorDisplay
  render(item: WidgetItem, context: RenderContext, settings: Settings): string | null
}
```
