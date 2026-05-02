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

---

## 2026-04-26 — Repo visibility (Phase 1 / Task 5)

**Контекст:** Task 5 публикует skeleton в `IGoRFonin/cchud` на GitHub. PRD не требует public-видимости с момента 0.0.1; имя свободно (Phase 0 OQ-6).

**Решение:** создать репо как **private**. Переключение в public — ручное, после ревью контента (README, ATTRIBUTION, CHANGELOG, lockfile, отсутствие секретов в истории).

**Обоснование:**
- Skeleton не имеет user-facing полезности (binary печатает заглушку); public не несёт ценности до Phase 3 (0.1.0-alpha).
- Снижает риск раннего "discovery" с устаревшим README.
- CI отрабатывает одинаково в private (matrix workers + private actions), бесплатные минуты на personal account достаточны для Phase 1–7.
- При переходе в public — `gh repo edit IGoRFonin/cchud --visibility public --accept-visibility-change-consequences`.

**Последствия:** до Phase 9 распространение через `npx`/`brew` не работает (publish невозможен из private). Phase 9 переключает visibility в первой задаче дистрибуции.


---

## 2026-04-26 — Payload schema is snake_case, not camelCase (Phase 2 / Task 2)

**Контекст:** Task 2 plan требует `#[serde(rename_all = "camelCase")]` на `StatusPayload`, `Workspace`. Реальный payload Claude Code (`benches/samples/payload-cchud-sonnet-xlarge.json`) использует snake_case: `session_id`, `transcript_path`, `current_dir`, `project_dir`, `added_dirs`, `display_name`. С `rename_all = "camelCase"` unit-тест `parses_real_payload_sample` падает на `missing field `currentDir``.

**Решение:** убрать `#[serde(rename_all = "camelCase")]` из `StatusPayload` и `Workspace`. Rust-имена полей и так совпадают с JSON-ключами один-в-один.

**Обоснование:**
- Claude Code эмитит snake_case (зафиксировано в трёх Phase 0 семплах).
- Default serde rename = identity → snake_case Rust = snake_case JSON, аттрибут вреден.
- Альтернатива (явный `#[serde(rename = "...")]` на каждом поле) — лишний бойлерплейт без выгоды.

**Последствия:** Step 6 grep-чек плана (`grep -q rename_all`) и DoD-пункт про camelCase устарели — обновлены в коммите. Task 9 (полный envelope) тоже без `rename_all`.

---

## 2026-04-26 — Phase 2 → pipeline + first real binary

- 6 design decisions locked in spec [`docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md`](./superpowers/specs/2026-04-26-phase-2-pipeline-design.md)
- Hyperfine на Apple M4 Pro (200 runs, --warmup 20, payload-cchud-sonnet-xlarge.json):
  - cchud Mean = **1.3 ms** (min 1.1, max 2.3)
  - ccstatusline 2.2.8 Mean = **230.3 ms** (min 219.5, max 271.7)
  - **~177× speedup**
- AC-001 / AC-007 / AC-008 covered by automated tests (`tests/install.rs`, `tests/snapshots.rs`)
- Manual real-CC test: deferred (см. `plan/phase-2/manual-test-log.md`)
- Removed: `lexopt` (Phase 2 doesn't need named flags)
- Added: `serial_test 3`, `tempfile 3` (dev-only)
- Bin size release: 468 592 bytes (~458 KB; target < 5 MB ✓)
- Phase 2 tag: `phase-2-pipeline` (annotated, unsigned)

---

## 2026-04-26 — WidgetConfig serde tag = kebab-case

`#[serde(rename_all = "kebab-case")]` на `WidgetConfig`. Phase 2 ожидал
`"type": "Model"` (PascalCase, default serde). Phase 3 переключает на
kebab-case для паритета с upstream ccstatusline (REQ-006 — будущая
команда `cchud import` мигрирует существующие пользовательские
`~/.claude/settings.json` со статуслайном ccstatusline). Phase 2 unit-тест
`parses_minimal_cchud_block` ретрофитнут (`"Model"` → `"model"`); CI
зелёный. Пользовательские конфиги Phase 2 alpha — не существуют, потому
breaking change безопасен.

## 2026-04-26 — Phase 3 ускоряет типизацию cost/context_window

Phase 2 spec обещал "Phase 6 типизирует cost, context_window,
rate_limits". Phase 3 типизирует cost и context_window раньше — 14 из
23 виджетов Phase 3 их читают; держать их `Option<serde_json::Value>`
означало бы `Value::pointer` everywhere в widget-коде, без compile-time
защиты от опечаток в именах полей. `rate_limits` остаётся `Value` до
Phase 6 (Phase 3 не имеет виджета, который её читает). Также типизирован
`output_style` (виджет OutputStyle читает только `name`). Структуры:
`CostInfo`, `ContextWindowInfo`, `CurrentUsage` (untagged enum
number|object), `Worktree`, `VimState`, `OutputStyle` — в
`src/types/payload.rs`.

---

## D-2026-04-27 — gix vs git2 для Phase 5

**Контекст:** Phase 5 требует читать git state (HEAD, status, diff, remotes, tracking). Два кандидата:
- `git2` (libgit2 bindings) — зрелый, но требует libgit2 + cmake systemstack, сложности на Windows и в `cargo install`-сценарии.
- `gix` (pure Rust) — без C зависимостей, быстрый, но API менее стабилен (breaking changes между minor).

**Решение:** `gix = "=0.81.0"`, `default-features = false`, `features = ["max-performance-safe", "sha1"]`.

**Обоснование:**
1. **Дистрибуция через `cargo install` / npm-loader**: pure Rust → бинарь без рантайм-зависимостей. libgit2 в WASM/musl-сборках доставляет проблем.
2. **Cold-start budget < 8 ms p95**: gix `discover()` ~100 µs, status ~1-2 ms на репе среднего размера. git2 сопоставим, но имеет startup overhead на загрузке libgit2.so.
3. **Pin на точную версию**: gix меняет `gix::head::Head` API между minor; обновление gix → отдельный PR со smoke-тестом фикстур из `git::fixture`.
4. **Фичи**: `default-features = false` исключает worktree/transport/protocol — мы только читаем локальное состояние. `sha1` нужен явно при `default-features = false`.

**Нюанс реализации:** gix-actor 0.40.1 (вышел после gix 0.81.0) мигрировал на winnow 1.0, тогда как gix-object 0.58.0 ещё на winnow 0.7. Cargo.lock пинит `gix-actor = "=0.40.0"` через `cargo update gix-actor --precise 0.40.0`.

**Trade-offs приняты:**
- Зависимость от ручного bump'а gix. Митигация: CI-бенч на каждом обновлении.
- Если gix окажется неподходящим (например, регрессии в diff API) — fallback на `git2` остаётся опцией; интерфейс `GitInfo` спрятан за `pub(crate)` `repo: gix::Repository`.

**Альтернативы рассмотрены:**
- shell-out на `git` CLI: ~2-5 ms на каждый вызов из-за subprocess fork; неприемлемо для 6+ status-виджетов.
- кастомный read-only git parser: переизобретение, не оправдано.

**Owner:** Igor Fonin

---

## D-2026-04-28 — sonic-rs vs serde_json для transcript JSONL

**Контекст:** Phase 6 парсит JSONL-транскрипты CC размером до 50 МБ. PRD NFR §6 требует cold parse < 10 ms, warm + 1 МБ append < 3 ms. На построчном JSONL парсинг — главный bottleneck (≈80% wall-clock cold-path).

**Кандидаты:**
- `serde_json` — уже в deps; универсально; ~150-200 MB/s throughput на наших структурах.
- `sonic-rs` — SIMD-ускоренный; ≈3× быстрее на построчном чтении; ~5 MB крейт; pure Rust runtime.

**Решение:** `sonic-rs = "0.5"`.

**Обоснование:**
1. **Perf headroom**: 50 МБ / 200 MB/s = 250 ms на serde_json — выше budget. sonic-rs даёт ~80 ms cold parse и < 5 ms на 1 МБ append. Оба в budget, но sonic-rs оставляет запас на будущие виджеты.
2. **Зависимость уже изолирована**: только в `src/cache/parser.rs`. Если потребуется fallback — точечная замена через `cfg`.
3. **Bincode остаётся `serde_json`-совместимым**: `TranscriptStats` сериализуется в bincode (T4), на чтение — sonic-rs. Один формат на серде, разные движки.

**Trade-offs приняты:**
- Дополнительная dep ~0.5 MB binary. Митигация: lto+strip профиль release.
- API менее стабилен. Митигация: pin `sonic-rs = "0.5"` (major); breaking changes — отдельный PR.
- Windows CI: исторически были баги в SIMD-кодгене на MSVC. Митигация: если CI windows-latest red — добавить `#[cfg(windows)]` ветку в `parser::parse_line` (10 строк); план держится в Step 8 рисках T1.

**Альтернативы рассмотрены:**
- `simd-json` — мощнее на больших документах, но overhead выше для построчного < 1 KB JSONL.
- кастомный JSON parser — переизобретение, не оправдано.

**Owner:** Igor Fonin

**Дополнение (2026-04-28):** `MessagePayload.content` типизировано как `Option<serde_json::Value>`.
sonic-rs остался в `Cargo.toml` как deps, но парсинг полей ведётся через `serde_json::from_str`,
поэтому `content` нативно получает `serde_json::Value`. Phase 7 (Skills widget) будет работать
с `serde_json::Value` напрямую — дополнительных конвертаций не требуется.

---

## D-2026-04-29 — Phase 7 ключевые решения

**Контекст:** Реализация Phase 7 (релиз 0.5.0): 7 виджетов + per-widget overrides + 9 global theme settings + multi-line.

**Решения:**
1. **Источник для usage-кластера — payload-only** (`rate_limits.{five_hour, seven_day}`). Без HTTP/auth/keychain. CC ≥ 2.1.x шлёт `rate_limits` напрямую (подтверждено в `benches/samples/payload-cchud-sonnet-xlarge.json`).
2. **Per-widget overrides — `WidgetItem` wrapper.** `Vec<WidgetItem>` вместо `Vec<WidgetConfig>` в `Line`. Один flatten на конфиг.
3. **Multi-line — caller-loop в `main.rs`.** `Renderer::render_line(segs, &mut state)` остаётся single-line.
4. **`auto_align` — sentinel `WidgetConfig::AlignRight`.** 61-й вариант enum, не считается в "60 widgets".
5. **`compact_threshold < term_width` форсит `minimalist_mode`.**
6. **Skills tracking — `TranscriptStats.skill_names: Vec<String>`** (sorted/unique). `FORMAT_VERSION 1 → 2` (silent reset).
7. **`sysinfo = "0.32"`** для `FreeMemory`. `default-features = false`, `features = ["system"]`. `refresh_memory()` точечно (~50 µs macOS).
8. **`OnceLock<Option<ClaudeJson>>`** для `claude_account_email()`. Файл ≤ 10 KB; читается один раз за процесс.

**Связанные документы:** `docs/superpowers/specs/2026-04-29-phase-7-other-widgets-design.md`, `plan/phase-7-other-widgets.md` (outline).

---

## D-2026-05-01 — Phase 8 ключевые решения

**Контекст:** Реализация Phase 8 (релиз 0.9.0): `cchud configure` (interactive TUI) + `cchud import` (CLI миграция с ccstatusline).

**Решения:**

1. **Pure ratatui + crossterm.** Без `ratatui-interact`, без `tui-input`, без `ansi-to-tui`. Собственный `widgets_ui::input.rs` (~50 LOC) и `style_map::to_span` после refactor `compose_line` устраняют необходимость в этих хелперах.
2. **Feature flag `default = ["tui"]`.** `cargo install cchud --no-default-features` собирает минимальный бинарь (~8 MB). `commands::{configure, import}` + весь `tui/` под `#[cfg(feature = "tui")]`. Без feature `cchud configure`/`cchud import` печатают error + exit 2.
3. **`ratatui = "=0.30.0"`, `crossterm = "=0.29.0"`.** Pin patch — minor breaking changes защищены. crossterm 0.30.0 не существует; ratatui 0.30.0 использует crossterm 0.29.x. Rust MSRV поднят до 1.86 (ratatui 0.30.0 требует 1.86+).
4. **`compose_line` refactor.** Pure-функция в `Renderer` возвращает `Vec<StyledSegment>` (после flex/separator/auto-align/apply_widget_style). `Renderer::render_line = compose_line + emit_ansi`. TUI live preview = `compose_line + style_map::to_span`. Single source of truth — drift между TUI preview и реальным рендером невозможен.
5. **`PartialEq` на Settings chain.** Derived на `Settings`/`Line`/`WidgetItem`/`WidgetConfig`/`WidgetStyleOverride`/`ThemeConfig`/`ModelParams`/`CustomTextParams`/`CustomSymbolParams`/`LinkParams`/`CustomCommandParams`/`ContextBarParams`/`PowerlineTheme`. `App.dirty()` = `editable != initial`. Никаких runtime-изменений.
6. **Save target — `~/.config/cchud/settings.json`.** Тот же путь, что `config::load()`. Backup `<path>.bak.<unix-ts-ms>`. Atomic write `<path>.tmp` → `rename`.
7. **`cchud import` best-effort + warn.** Unknown widget type → пропуск + stderr warn. Если ноль валидных widgets — exit 1. `~/.claude/settings.json` → автодетект секции `ccstatusline`; `--from <path>` — explicit.
8. **Reducer pure (no IO).** `handle_key(&mut App, KeyEvent) -> ReducerEffect`. Save/Quit/Discard поднимаются как `ReducerEffect`, обрабатываются в event loop. ≥15 unit-тестов без `TestBackend`.
9. **`tempfile::NamedTempFile` для sample transcript.** Owned `App` через `Option<NamedTempFile>` (RAII) — fixture-файл живёт ровно столько, сколько TUI.
10. **Sample payload — захардкоженный inline.** `tui::sample::payload() -> (StatusPayload, NamedTempFile)`. Никаких runtime-аллокаций.

**Связанные документы:** `docs/superpowers/specs/2026-05-01-phase-8-tui-design.md`, `plan/phase-8-tui.md` (outline).

---

## D-2026-05-02 — Phase 9: Distribution + 1.0.0

**Context:** Шипим `cchud 1.0.0` как production-grade CLI tool с двумя установочными каналами.

**Decisions:**

1. **Channels:** npm (primary, через `npx --yes cchud@1.0.0 install`) + `install.sh` (Mac/Linux secondary). Drop Homebrew (PAT-based formula auto-bump painful), drop crates.io (defer Phase 10).
2. **5 cross-build targets:** `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`. ARM Linux — defer Phase 10.
3. **npm Approach 2 (`optionalDependencies` per platform):** 6 пакетов (`cchud` + 5× `@cchud/cli-<platform>`). Без `postinstall` script — нет network call на install, работает с `--ignore-scripts`.
4. **`npx cchud@<version> install` — primary entry point.** Не `npm i -g cchud`. JS shim spawn'ит native binary с `["install"]`, native binary self-relocates в `~/.local/bin/cchud`.
5. **Self-relocation:** `cchud install` копирует себя в `~/.local/bin/cchud` (Unix) или `%LOCALAPPDATA%\cchud\cchud.exe` (Windows). Идемпотентен. `--no-relocate` flag для разработчиков.
6. **Pin версии в README:** ВСЕГДА конкретная версия (`@1.0.0`), никогда `@latest` — supply-chain mitigation.
7. **Multi-layer security:** npm `--provenance` (sigstore attestations) + npm 2FA (`auth-and-writes`) + `NPM_TOKEN` тип `automation` + GH Actions SHA-pinning.
8. **RC soak gating:** `v1.0.0-rc.1` → npm `dist-tag: next` → ≥ 24h manual soak × 4 envs → `v1.0.0` → npm `dist-tag: latest`. Workflow `npm dist-tag rm cchud latest` rollback при smoke fail.
9. **Smoke-install matrix × 3 (mac/linux/win)** в release.yml — gate между `published` и `actually works`.
10. **`cchud doctor` (9 checks, без Powerline-font detect):** version, binary path, platform target, color level, hyperlinks, cache dir, claude settings, cchud config, gh CLI (conditional). Exit 0/1/2.
11. **Без `install.ps1`** — Windows users имеют npm. PowerShell-ports defer Phase 10.
12. **Reuse Phase 8 `atomic_save` pattern** для settings.json backup `<path>.bak.<unix-ms>`.
