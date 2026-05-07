# Task 14 — Release 0.5.0

**Цель:** Финал. Bump version 0.4.0 → 0.5.0, обновить CHANGELOG / README / `docs/widgets.md` (60/60 DONE), провести 5+ мин manual battle-test, создать tag `v0.5.0`, push origin + tag.

**Files:**
- Modify: `Cargo.toml:3` — `version = "0.5.0"`
- Modify: `CHANGELOG.md` — добавить запись 0.5.0
- Modify: `README.md` — supported widgets таблица 60/60
- Modify: `docs/widgets.md` — 7 строк TODO → DONE; счётчик 53 → 60
- Modify: `plan/README.md` — Phase 7 → `[x]`, Phase 8 → `[~]`
- Create: `plan/phase-7/manual-test-log.md`

---

- [ ] **Step 1: Bump version**

В `Cargo.toml:3`:

```toml
version = "0.5.0"
```

Run: `cargo build --release --locked`
Expected: PASS; `Cargo.lock` обновлён.

- [ ] **Step 2: Update CHANGELOG.md**

Добавить в начало:

```markdown
## 0.5.0 — 2026-04-29

**Phase 7: Other Widgets, Style Overrides, Multi-line.** Финальный паритет 60/60 виджетов с ccstatusline 2.2.8.

### Added
- 7 виджетов: `session-usage`, `weekly-usage`, `block-reset-timer`, `weekly-reset-timer`, `claude-account-email`, `free-memory`, `skills`.
- Per-widget overrides на каждом widget item (`color`, `background_color`, `bold`).
- 9 global theme settings: `global_bold`, `inherit_separator_colors`, `override_background_color`, `override_foreground_color`, `minimalist_mode`, `flex_mode`, `compact_threshold`, `auto_align`, `continue_theme_across_lines`.
- Multi-line render: `settings.lines` склеивается через `\n`.
- `WidgetConfig::AlignRight` sentinel для `auto_align`.
- `WidgetItem` wrapper в config schema (JSON-совместим с upstream).
- `apply_widget_style` централизованно применяет stack overrides.
- `RenderState` для theme/separator cycling между линиями.

### Changed
- `payload.rate_limits` теперь типизированный `Option<RateLimits>` (было `Option<Value>`).
- `Renderer::render(segs)` → `render_line(segs, &mut state, theme)`.
- `Line.widgets: Vec<WidgetConfig>` → `Vec<WidgetItem>` (внутреннее API; JSON shape совместим).
- `TranscriptStats` добавляет `skill_names: Vec<String>` (sorted/unique).
- `FORMAT_VERSION 1 → 2` — silent reset кэша при первом запуске.

### Performance
- cchud-8w: < 5 ms p95.
- cchud-60w: < 12 ms p95.
- Cold parse 50 MB transcript: < 11 ms (+1 ms на skill detection).

### Dependencies
- Added: `sysinfo = "0.32"` (default-features = false, features = ["system"]).
```

- [ ] **Step 3: Update README.md**

В секции supported widgets таблице сменить счётчик с `53/60` на `60/60`. Добавить 7 новых строк (skills, claude-account-email, free-memory, session-usage, weekly-usage, block-reset-timer, weekly-reset-timer).

- [ ] **Step 4: Update docs/widgets.md**

Поменять статус 7 виджетов TODO → DONE; обновить заголовок: "60 widgets supported (60/60 upstream parity)".

- [ ] **Step 5: Update plan/README.md**

Phase 7 → `[x]`, Phase 8 → `[~]` (или оставить как-был).

- [ ] **Step 6: Manual battle-test (5+ мин)**

В реальной CC-сессии запустить `cchud install --force`, потом 5+ минут активной работы с разными конфигами:
- `usage-cluster.json` — все 4 timers/usage виджета.
- `multi-line-themed.json` — две строки с powerline, `continue_theme_across_lines: true`.
- `theme-globals.json` — `override_background_color: "#aabbcc"`.
- `per-widget-override.json` — `bold: true` на model.

Записать наблюдения в `plan/phase-7/manual-test-log.md`:

```markdown
# Phase 7 Manual Test Log — 2026-04-29

## Configs tested
- usage-cluster.json — 4 timers showing correctly; resets_at delta verified.
- multi-line-themed.json — \n separator, continue_theme_across_lines preserves cursor.
- theme-globals.json — override_background_color applied to all widgets.
- per-widget-override.json — model bold; session-cost color override.

## Observations
(Заполнить.)

## Issues found
(Заполнить или None.)
```

- [ ] **Step 7: Final verify**

```bash
cargo build --release --locked
./target/release/cchud --version
# Expected: 0.5.0

cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё PASS.

- [ ] **Step 8: Commit + tag + push**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md docs/widgets.md plan/README.md plan/phase-7/manual-test-log.md
git commit -m "release: 0.5.0 — Phase 7 (60/60 widgets, overrides, multi-line)"
git tag v0.5.0
git push origin main
git push origin v0.5.0
```

Expected: CI matrix зелёный (macos / ubuntu / windows). GitHub Release создаётся автоматически (если настроен release-on-tag workflow).
