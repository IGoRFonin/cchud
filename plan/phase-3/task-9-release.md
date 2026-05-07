# Task 9 — Release 0.1.0-alpha

**Files:**
- Modify: `Cargo.toml` (`version = "0.0.1"` → `"0.1.0-alpha"`)
- Modify: `Cargo.lock` (auto-regen после version bump)
- Modify: `CHANGELOG.md` (новая запись `[0.1.0-alpha] — 2026-04-26`)
- Modify: `README.md` (Status WIP → Phase 3 done; widgets table; "How to use" section; warning про CustomCommand env-inheritance)
- Modify: `docs/widgets.md` (Status TODO → DONE для 24 виджетов: 1 Phase 2 Model + 23 Phase 3)
- Modify: `plan/README.md` (Phase 3 → `[x]`, Phase 4 → `[~]`)
- Modify: `plan/phase-3/manual-test-log.md` (заполнить шаблон фактическими наблюдениями после ручного теста)
- New tag: `v0.1.0-alpha` (git tag + push)

## Goal

Sign-off Phase 3:

1. Bump версию `0.0.1` → `0.1.0-alpha`.
2. Обновить README с актуальным статусом + supported widgets table.
3. Добавить запись в CHANGELOG.
4. Обновить `docs/widgets.md` — отметить DONE.
5. Обновить `plan/README.md` (Phase 3 done, Phase 4 next).
6. Manual real-CC test: запустить cchud в реальном Claude Code в worktree с vim mode ≥ 5 минут, заполнить `plan/phase-3/manual-test-log.md`.
7. Создать git tag `v0.1.0-alpha`, запушить.
8. Проверить, что CI matrix зелёный после push'а tag'а.

## Inputs

- T1–T8 закрыты, все 23 виджета с тестами и snapshot-проверкой.
- `cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check` зелёные.
- `benches/phase-3.md` показывает p95 < 5 ms на 22-widget config.
- Установлен Claude Code локально, рабочая сессия возможна.

---

- [ ] **Step 1: Bump version в `Cargo.toml`**

Edit:
- `old_string`: `version = "0.0.1"`
- `new_string`: `version = "0.1.0-alpha"`
- `file_path`: `/Users/igor/mp/startup/cchud/Cargo.toml`

Verify:
```bash
grep -E '^version = ' Cargo.toml
```
Expected: `version = "0.1.0-alpha"`.

```bash
cargo build --release --locked
```
Expected: `--locked` упадёт ("Cargo.lock needs update"). Регенерировать:
```bash
cargo build --release
cargo build --release --locked
```
Второй должен пройти exit 0.

Verify:
```bash
./target/release/cchud --version
```
Expected: `0.1.0-alpha`.

- [ ] **Step 2: Добавить CHANGELOG запись**

Edit `CHANGELOG.md`:

Добавить ПОСЛЕ `## [Unreleased]` и ПЕРЕД `## [0.0.1]`:

Edit:
- `old_string`:
  ```
  ## [Unreleased]

  ## [0.0.1] — 2026-04-26
  ```
- `new_string`:
  ```
  ## [Unreleased]

  ## [0.1.0-alpha] — 2026-04-26

  ### Added

  - 23 widgets (Phase 3 MVP):
    - **Static cluster** (3): `custom-text`, `custom-symbol`, `link` (OSC 8 hyperlink).
    - **Trivial cluster** (5): `version`, `claude-session-id`, `terminal-width`, `output-style`, `vim-mode`.
    - **Session cluster** (3): `session-name`, `session-clock`, `session-cost`.
    - **Context cluster** (6): `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`.
    - **Worktree cluster** (5): `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch`.
    - **Subprocess** (1): `custom-command` (argv-style spawn, configurable timeout).
  - Typed payload sub-structures: `CostInfo`, `ContextWindowInfo`, `CurrentUsage` (untagged enum), `Worktree`, `VimState`, `OutputStyle`.
  - `WidgetConfig` `serde tag = "type", rename_all = "kebab-case"` for parity with upstream `ccstatusline`.
  - `wait-timeout 0.2` runtime dependency (CustomCommand timeout).
  - Synthetic payload fixtures: `payload-synthetic-vim-worktree.json`, `payload-synthetic-current-usage-total.json`.
  - 5 snapshot-test scenarios covering full Phase 3 widget rendering.
  - Hyperfine bench gate: p95 < 5 ms on 22-widget config (M-серия).

  ### Changed

  - `WidgetConfig` JSON-tag сменился с PascalCase на kebab-case (breaking — Phase 2 alpha не имеет пользователей; `cchud import` для миграции с ccstatusline отложен в Phase 9).
  - `payload.cost`, `payload.context_window`, `payload.output_style` теперь типизированы (Phase 2 хранил как `Option<serde_json::Value>`).

  ### Notes

  - `rate_limits`, `effort`, `thinking` остаются `Option<serde_json::Value>` до Phase 6/7.
  - `thinking-effort` widget отложен в Phase 6 (зависит от JSONL-транскрипта).
  - Powerline-renderer и color/bold styling — Phase 4.
  - Git-виджеты (Branch, Status, Stash, etc.) — Phase 5.
  - CustomCommand subprocess inherits parent env; opt-in env-allowlist — Phase 7.

  ## [0.0.1] — 2026-04-26
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/CHANGELOG.md`

Также добавить ссылки в footer:

Edit:
- `old_string`:
  ```
  [Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.0.1...HEAD
  [0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
  ```
- `new_string`:
  ```
  [Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.1.0-alpha...HEAD
  [0.1.0-alpha]: https://github.com/IGoRFonin/cchud/releases/tag/v0.1.0-alpha
  [0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/CHANGELOG.md`

- [ ] **Step 3: Обновить README**

Edit 1 (Status):
- `old_string`: `**Status:** WIP — Phase 1 (skeleton + CI). Not yet usable. See [\`plan/README.md\`](plan/README.md) for roadmap.`
- `new_string`: `**Status:** 0.1.0-alpha — 24 of 60 upstream widgets supported (Phase 2 \`model\` + Phase 3 MVP cluster). Plain renderer; Powerline lands in Phase 4. See [\`plan/README.md\`](plan/README.md) for roadmap.`
- `file_path`: `/Users/igor/mp/startup/cchud/README.md`

Edit 2 (заменить Status section "This is a Phase 1 skeleton" на актуальный):
- `old_string`:
  ```
  ## Status

  This is a Phase 1 skeleton: project scaffolding, CI matrix on macOS + Ubuntu + Windows, snapshot-test harness over Phase 0 payload fixtures. The binary currently prints `cchud (skeleton) | input bytes: N` — no widgets, no rendering. Real pipeline lands in Phase 2.
  ```
- `new_string`:
  ```
  ## Status

  Alpha release: 24 widgets working end-to-end in Claude Code. Plain renderer (single line, ` | ` separator). Powerline visual parity is Phase 4; full `ccstatusline` widget set lands across Phases 5–7.

  ## Supported widgets (24 / 60)

  | Source | Widgets |
  |---|---|
  | Payload (fast) | `model`, `version`, `claude-session-id`, `terminal-width`, `output-style`, `vim-mode`, `session-name`, `session-clock`, `session-cost`, `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`, `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch` |
  | Static (config) | `custom-text`, `custom-symbol`, `link` |
  | Subprocess | `custom-command` (argv-style, configurable timeout, default 200ms) |

  See [`docs/widgets.md`](docs/widgets.md) for the full 60-widget roadmap.

  ## Configure

  `cchud install` wires cchud into `~/.claude/settings.json`. Widget list lives in the `cchud` block (kebab-case `type`):

  ```json
  {
    "cchud": {
      "version": 1,
      "lines": [{
        "widgets": [
          {"type": "model"},
          {"type": "session-cost"},
          {"type": "context-percentage"},
          {"type": "context-bar", "width": 10},
          {"type": "worktree-name"},
          {"type": "vim-mode"}
        ]
      }],
      "theme": {}
    }
  }
  ```

  ### CustomCommand security

  `custom-command` spawns the configured binary argv-style (no shell). It **inherits the parent process env**, so any `API_KEY` / secret in your shell is visible to the subprocess. Phase 7 will add opt-in env-allowlist + sandboxing.
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/README.md`

- [ ] **Step 4: Обновить `docs/widgets.md` — отметить DONE**

Read `docs/widgets.md` целиком, в таблице для каждого из 24 виджетов поменять `Status` колонку с `TODO` (или `WIP`) на `DONE`. Виджеты к обновлению (по `name`):

- `model` (Phase 2)
- Phase 3 (23): `output-style`, `vim-mode`, `version`, `claude-session-id`, `terminal-width`, `session-name`, `session-clock`, `session-cost`, `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`, `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch`, `custom-text`, `custom-symbol`, `link`, `custom-command`.

(`thinking-effort` ОСТАЁТСЯ TODO — отложен в Phase 6.)

Способ — `sed` через Edit tool на каждой строке (23 + 1 = 24 edits) — нудно. Альтернатива: один глобальный `Edit` с `replace_all` на правильно-уникальной paire.

Альтернативный быстрый способ: открыть `docs/widgets.md` и сделать построчный замен для каждого `name`. Каждая строка имеет уникальный `name`:

Edit pattern:
- `old_string`: `| \`<name>\` | 3 | payload | low | TODO |` → `| \`<name>\` | 3 | payload | low | DONE |`

Применить 23 Edit'а (или больше — для всех Phase 3 виджетов). И для `model`:
- `old_string`: `| \`model\` | 2 | payload | low | TODO | sentinel — первый виджет |`
- `new_string`: `| \`model\` | 2 | payload | low | DONE | sentinel — первый виджет |`

**Эффективнее:** прочитать файл, сделать минимальное число edit'ов с уникальными подстроками `| ${name} | 3 | ... | TODO |`. Где `payload` или `static` — игнорируем, главное `TODO` → `DONE` для конкретного name.

Verify после всех edits:
```bash
grep -c 'DONE' docs/widgets.md
grep 'TODO' docs/widgets.md | head -5
```

Expected:
- `DONE` count: 24 (1 Phase 2 + 23 Phase 3)
- Оставшиеся `TODO` — Phase 4–7 виджеты, включая `thinking-effort`, git-кластер, env-кластер, etc.

- [ ] **Step 5: Обновить `plan/README.md`**

Edit:
- `old_string`:
  ```
  - [x] Фаза 2 — Pipeline
  - [~] Фаза 3 — MVP (next)
  - [ ] Фаза 4 — Powerline
  ```
- `new_string`:
  ```
  - [x] Фаза 2 — Pipeline
  - [x] Фаза 3 — MVP
  - [~] Фаза 4 — Powerline (next)
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/plan/README.md`

- [ ] **Step 6: Standard gate перед manual-тестом**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё exit 0. Если падает — фикси перед тегом.

- [ ] **Step 7: Manual real-CC test**

1. Установить cchud в Claude Code:
```bash
./target/release/cchud install
```

2. Прописать config в `~/.claude/settings.json` блок `cchud`:
```json
{
  "cchud": {
    "version": 1,
    "lines": [{
      "widgets": [
        {"type": "model"},
        {"type": "version"},
        {"type": "vim-mode"},
        {"type": "worktree-name"},
        {"type": "worktree-branch"},
        {"type": "worktree-mode"},
        {"type": "session-cost"},
        {"type": "session-clock"},
        {"type": "context-percentage"},
        {"type": "context-bar", "width": 10},
        {"type": "claude-session-id"},
        {"type": "custom-text", "text": "demo"},
        {"type": "custom-command", "command": "echo", "args": ["phase-3"], "timeout_ms": 200}
      ]
    }],
    "theme": {}
  }
}
```

3. Создать worktree:
```bash
git worktree add /tmp/cchud-wt -b feature/phase-3-demo
cd /tmp/cchud-wt
```

4. Запустить Claude Code в worktree-директории, включить vim mode (`/vim` или через config).

5. Поработать ≥ 5 минут (запросы, чтение файлов, тесты). Наблюдать statusline.

6. Заполнить `plan/phase-3/manual-test-log.md` фактическими наблюдениями (10 чекбоксов из шаблона T1):
   - Statusline без артефактов
   - Worktree-name/branch правильные
   - Vim mode переключается
   - SessionCost растёт
   - SessionClock тикает
   - ContextPercentage обновляется
   - ContextBar заполняется
   - CustomCommand "echo phase-3" → "phase-3"
   - Лагов на UI нет
   - stderr CC чист

7. Также записать `cchud --version` (`0.1.0-alpha`) и hyperfine p95 в Метрики секцию.

**Что делать при отклонении:**
- `vim`/`worktree` поля payload не приходят, как ожидалось → обновить synthetic-семплы в T1 + соответствующие unit-тесты + новый snapshot scenario в T8 (но уже как hotfix-PR в Phase 4 prep).
- Лаги на UI → проверить `cargo build --release` не работает в Rosetta (`file ./target/release/cchud` должен показать `arm64`).
- `cchud:` warning в stderr CC → читать вывод; обычно "invalid config" или "invalid payload"; залогировать в test-log.

- [ ] **Step 8: Commit non-tag changes (cargo bump + README + CHANGELOG + widgets.md + plan/README + manual-log)**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md \
        docs/widgets.md plan/README.md plan/phase-3/manual-test-log.md
git commit -m "chore(phase-3): release 0.1.0-alpha — 23 widgets done

- Cargo.toml: version 0.0.1 → 0.1.0-alpha
- CHANGELOG.md: 0.1.0-alpha entry (added 23 widgets, typed sub-payloads,
  kebab-case retrofit, hyperfine gate p95 < 5ms)
- README.md: Status update + supported widgets table (24/60) +
  CustomCommand env-inheritance warning + configure example
- docs/widgets.md: 24 entries TODO → DONE
- plan/README.md: Phase 3 → [x], Phase 4 → [~]
- plan/phase-3/manual-test-log.md: filled from real CC session

Phase 3 sign-off complete; v0.1.0-alpha tag follows.
"
```

- [ ] **Step 9: Создать git tag и push**

```bash
git tag -a v0.1.0-alpha -m "Phase 3 MVP — 23 widgets, plain renderer"
git push origin master
git push origin v0.1.0-alpha
```

Verify:
```bash
git tag -l | grep v0.1.0-alpha
git log --oneline | head -5
```

Expected: `v0.1.0-alpha` в списке тегов; commit `chore(phase-3): release 0.1.0-alpha` в head'е.

- [ ] **Step 10: Проверить CI matrix после push**

```bash
gh run list --branch master --limit 3
```

Expected: последний workflow run после push'а tag'а — success на all 3 platforms (macos, ubuntu, windows).

Если падает на Windows из-за `#[cfg(unix)]` тестов — это OK (тесты skip), компиляция должна пройти.

Если падает на macOS/ubuntu — что-то пошло не так в Step 6; rollback tag и фикси:
```bash
git tag -d v0.1.0-alpha
git push origin :refs/tags/v0.1.0-alpha
# fix the issue, repeat from Step 6.
```

- [ ] **Step 11: Verification — task-specific gate**

```bash
./target/release/cchud --version
grep -E '^version = ' Cargo.toml
grep -c '^## \[0.1.0-alpha\]' CHANGELOG.md
grep -c 'Phase 3 — MVP' README.md
grep -c 'DONE' docs/widgets.md
grep -E '^- \[x\] Фаза 3' plan/README.md
git tag -l v0.1.0-alpha
```

Expected:
```
0.1.0-alpha
version = "0.1.0-alpha"
1
≥1                  (mention в README)
≥24
- [x] Фаза 3 — MVP
v0.1.0-alpha
```

И:
```bash
grep -c 'TBD' plan/phase-3/manual-test-log.md
```

Expected: `0` (все TBD заменены реальными значениями).

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
./target/release/cchud --version    # 0.1.0-alpha
```

## Definition of Done

- [ ] `Cargo.toml` `version = "0.1.0-alpha"`; `Cargo.lock` обновлён
- [ ] `cchud --version` печатает `0.1.0-alpha`
- [ ] `CHANGELOG.md` содержит запись `## [0.1.0-alpha] — 2026-04-26` с Added/Changed/Notes секциями
- [ ] `README.md` обновлён: Status section + supported widgets table (24/60) + Configure example + CustomCommand security note
- [ ] `docs/widgets.md` отметка DONE для 24 виджетов
- [ ] `plan/README.md` Phase 3 → `[x]`, Phase 4 → `[~]`
- [ ] Manual real-CC test пройден ≥ 5 минут с активным vim+worktree, `plan/phase-3/manual-test-log.md` заполнен (no `TBD` остались)
- [ ] `git tag v0.1.0-alpha` создан и запушен
- [ ] CI matrix зелёный после push'а tag'а на macOS/Linux/Windows
- [ ] Один commit `chore(phase-3): release 0.1.0-alpha ...` (manual-log + bump + докум.)

## Files touched

- `Cargo.toml` (modified)
- `Cargo.lock` (auto-regenerated)
- `CHANGELOG.md` (modified)
- `README.md` (modified)
- `docs/widgets.md` (modified)
- `plan/README.md` (modified)
- `plan/phase-3/manual-test-log.md` (modified — заполнено)
- git tag `v0.1.0-alpha` (created + pushed)

## Risks & rollback

- **Manual real-CC test показывает регрессии**: rollback tag (см. Step 10), фиксы в hotfix-задачах после-T9. Если регрессия НЕ блокирующая — занести в `docs/issues/` и закрыть тегом alpha (alpha-релиз ожидаем кривым).
- **`cargo build --release --locked` падает после version bump**: запустить `cargo build --release` без --locked, потом снова --locked. Если упорствует — `cargo update -p cchud` явно.
- **Tag уже существует**: `git tag -d v0.1.0-alpha && git push origin :refs/tags/v0.1.0-alpha` перед re-tag.
- **`gh run list` показывает зелёный CI до push'а тега, но красный после**: tag-push триггерит release-workflow (если есть). Проверить `.github/workflows/`. Phase 1 не упоминал release-workflow; если его нет — push tag безопасен.
- **`docs/widgets.md` имеет уникальные `name`-строки, но edit-цепочка нудная**: возможна ошибка с пропуском какого-то виджета. Verify через `grep -c 'DONE' docs/widgets.md` и пересчёт вручную. Если расхождение — добавить недостающий edit.
- **README "How to use" скрин не работает в реальной CC**: Manual test в Step 7 ловит. Если поломка — фикси перед тегом.
- **`CHANGELOG.md` markdown не валиден (двойные `[Unreleased]`)**: `npm run check-changelog` если есть; иначе глазами.
- **Rollback всей T9**:
  ```bash
  git reset --hard HEAD~1                       # снимает chore commit
  git tag -d v0.1.0-alpha                       # локально
  git push origin :refs/tags/v0.1.0-alpha       # remote
  ```
  T1–T8 commit'ы остаются нетронутыми.
