# Фаза 7 — Implementation Plan (Other Widgets, Style Overrides, Multi-line, 0.5.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Релиз `0.5.0` cchud — финальный паритет 60/60 виджетов с ccstatusline 2.2.8 + полный паритет per-widget и global theme overrides + multi-line render. Закрывает три бакета работы за один bump: 7 оставшихся виджетов (`skills`, `claude-account-email`, `free-memory`, `session-usage`, `weekly-usage`, `block-reset-timer`, `weekly-reset-timer`), 7.0a per-widget overrides (`color`/`background_color`/`bold`), 7.0b 9 global theme settings + multi-line.

**Architecture:** Линейная цепочка из 14 vertical-slice задач. Слой 1 (T1–T4) — типы и фундамент: `payload::RateLimits` типизация, `WidgetItem` wrapper + `WidgetStyleOverride`, `ThemeConfig` +9 полей + `FlexMode` + `AlignRight` sentinel + `apply_widget_style` + `RenderState`. Слой 2 (T5–T7) — утилиты: `format_memory`/`format_duration_long`, `commands::env_loader`. Слой 3 (T8) — JSONL: `TranscriptStats.skill_names` + `FORMAT_VERSION 2` + parser merge. Слой 4 (T9–T11) — 7 виджетов в трёх файлах. Слой 5 (T12–T13) — render rewrite (`flex.rs`, `Renderer::render_line(segs, state)`, multi-line orchestration в `main.rs`). Слой 6 (T14) — snapshots + bench + release.

**Принципы:** виджеты не знают про overrides, возвращают plain text + `default_style()`; стиль накладывается централизованно в `Renderer::render_line` через `apply_widget_style`. `Renderer::render_line` — single-line; multi-line — caller-side loop в `main.rs`. Никаких новых сетевых I/O, никакого auth/keychain.

**Scope correction (vs `plan/phase-7-other-widgets.md`):** Старый outline содержал устаревшие пункты — виджеты `CustomCommand`, `Link`, `ContextBar`, `VimMode`, `OutputStyle` уже реализованы в Phase 3. Outline также упоминает HTTP-источник для usage-кластера; canonical scope Phase 7 (`docs/widgets.md`) фиксирует **7 оставшихся виджетов**. Source `http` для usage-widgets переклассифицируется в `payload` (CC шлёт `rate_limits` напрямую, подтверждено в `benches/samples/payload-cchud-sonnet-xlarge.json`). Spec единственный источник truth — `docs/superpowers/specs/2026-04-29-phase-7-other-widgets-design.md`.

**Tech Stack:** Rust 1.85 (Edition 2024), `sysinfo = "0.32"` (новая dep, T1, `default-features = false`, features `["system"]`, для `FreeMemory`), `serde`/`serde_json`/`bincode`/`sonic-rs`/`time`/`siphasher`/`dirs` (уже есть). DECISIONS-запись D-2026-04-29 (sysinfo + payload-only usage + WidgetItem wrapper).

---

## DAG задач

```
T1 (setup) ──► T2 (payload types) ──► T3 (WidgetItem wrapper) ──► T4 (theme +9 / flex / align / state)
                                                                              │
                                                                              ▼
                                              T5 (format utils) ──► T6 (env_loader) ──► T7 (cache skills)
                                                                              │
                                                                              ▼
                                                T8 (usage 4w) ──► T9 (env 2w) ──► T10 (skills 1w)
                                                                              │
                                                                              ▼
                                                                T11 (render rewrite)
                                                                              │
                                                                              ▼
                                                                T12 (main multi-line)
                                                                              │
                                                                              ▼
                                                              T13 (snapshots+bench) ──► T14 (release)
```

Линейная цепочка, между задачами — review checkpoint. T1–T4 — типы/фундамент. T5–T7 — utils + cache. T8–T10 — виджеты (после T7 идут параллельно по сути, но в плане последовательно). T11 — render rewrite (риск snapshot regression). T12 — multi-line wiring. T13–T14 — verification + release.

| # | Файл | Цель | Виджеты | Гейт |
|---|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | sysinfo dep + skeleton модулей (`widgets/env`, `widgets/usage`, `commands/env_loader`, `render/flex`, `util/format_memory`, `util/format_duration_long`) + DECISIONS | 0 | `cargo build --release` зелёный; binary < 8.5 MB |
| 2 | [`task-2-payload-types.md`](./task-2-payload-types.md) | `payload::RateLimits + RateBucket` (заменяет `Option<Value>`) | 0 | `cargo test types::payload` зелёный |
| 3 | [`task-3-widget-item.md`](./task-3-widget-item.md) | `WidgetItem` wrapper + `WidgetStyleOverride` + `parse_color`; `Vec<WidgetConfig>` → `Vec<WidgetItem>` (механическая миграция тестов) | 0 | `cargo test types::config` зелёный + Phase 4 snapshot'ы byte-identical |
| 4 | [`task-4-theme-globals.md`](./task-4-theme-globals.md) | `ThemeConfig` +9 полей + `FlexMode` enum + `WidgetConfig::AlignRight` sentinel + `apply_widget_style` + `RenderState` | 0 | `cargo test types::config render::` зелёный |
| 5 | [`task-5-format-utils.md`](./task-5-format-utils.md) | `util/format_memory.rs` (auto-unit Kb/Mb/Gb/Tb) + `util/format_duration_long.rs` (`format_short`/`format_long`) | 0 | `cargo test util::format_memory util::format_duration_long` зелёный |
| 6 | [`task-6-env-loader.md`](./task-6-env-loader.md) | `commands/env_loader.rs` — `ClaudeJson` + `OnceLock` + `claude_account_email()` + test API | 0 | `cargo test commands::env_loader` зелёный |
| 7 | [`task-7-cache-skills.md`](./task-7-cache-skills.md) | `TranscriptStats.skill_names: Vec<String>` + `FORMAT_VERSION 1 → 2` + `parser::apply_entry` skill detection + `merge_stats` union+sort+dedup | 0 | `cargo test cache::` зелёный; format mismatch silent reset |
| 8 | [`task-8-usage.md`](./task-8-usage.md) | `widgets/usage.rs` — 4 виджета + 4 enum-варианта + 4 case в `build_one` + ≥8 unit-тестов | 4 | `cargo test widgets::usage` зелёный |
| 9 | [`task-9-env.md`](./task-9-env.md) | `widgets/env.rs` — `ClaudeAccountEmail` + `FreeMemory` + 2 enum-варианта + 2 case в `build_one` + tests | 2 | `cargo test widgets::env` зелёный |
| 10 | [`task-10-skills.md`](./task-10-skills.md) | `widgets/transcript_meta.rs` — `Skills` widget + `WidgetConfig::Skills` + build_one + tests | 1 | `cargo test widgets::transcript_meta::tests::skills_*` зелёный |
| 11 | [`task-11-render.md`](./task-11-render.md) | `render/flex.rs` (`flex_budget`/`truncate_to_budget`) + `Segment.align_marker` + `Renderer::render_line(segs, state, theme)` + Plain/Powerline globals + AlignRight | 0 | `cargo test render::` зелёный; Phase 4 snapshot'ы byte-identical |
| 12 | [`task-12-main-multiline.md`](./task-12-main-multiline.md) | `main.rs` multi-line orchestration loop + `continue_theme_across_lines` + flex truncate + `build_widgets` для всех lines | 0 | `cargo run` зелёный на multi-line config |
| 13 | [`task-13-snapshots-bench.md`](./task-13-snapshots-bench.md) | `tests/snapshots_phase7.rs` ≥6 фикстур + `benches/configs/phase-7-60w.json` + `benches/phase-7.md` + hyperfine targets | 0 | `cchud-8w < 5ms p95`, `cchud-60w < 12ms p95`, `cold parse 50MB < 11ms` |
| 14 | [`task-14-release.md`](./task-14-release.md) | `Cargo.toml` 0.5.0 + CHANGELOG + `docs/widgets.md` 7 строк TODO → DONE + tag `v0.5.0` + manual battle-test | 0 | `cchud --version` показывает `0.5.0` |

**Итого:** 7 виджетов + 9 theme settings + per-widget overrides + multi-line + AlignRight, ~5–7 рабочих дней.

## Pre-flight (требования до старта Task 1)

- **Phase 6 завершена** — релиз 0.4.0 запушен, `git tag --list 'v0.4.0'` показывает тег.
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое**: `git status` пусто.
- **`rustc` ≥ 1.85** (см. `rust-toolchain.toml`).
- **`hyperfine` ≥ 1.20.0**.
- **`cargo-insta`** установлен (`cargo install cargo-insta`).
- **`benches/samples/payload-cchud-sonnet-xlarge.json`** содержит реальное `rate_limits` поле (подтверждено в spec § Decision 1).
- **`~/.claude.json`** существует или отсутствует — оба сценария тестируются.
- **`du -sh ~/.cache/cchud/`** — может быть любой; format-bump (1→2) валидируется в T7.

## Definition of Done всей Фазы 7

- [ ] 7 новых виджетов реализованы; каждый покрыт ≥2 unit-тестами (happy + 1+ edge)
- [ ] `WidgetItem` wrapper заменил `Vec<WidgetConfig>` в `Line.widgets`; existing JSON-конфиги парсятся без изменений
- [ ] `payload::rate_limits: Option<RateLimits>` типизирован; CC < 2.1.x → виджеты graceful no-op
- [ ] `ThemeConfig` поддерживает все 9 новых полей; defaults сохраняют Phase 4 поведение
- [ ] `apply_widget_style` применяет per-widget override → theme global override (точно по upstream семантике)
- [ ] `Renderer::render_line(segs, &mut state, &theme)` — новая сигнатура; `RenderState` хранит cursor для theme/separator cycling
- [ ] `main.rs` склеивает `settings.lines` через `\n`; `continue_theme_across_lines` контролирует reset/continue cursor
- [ ] `auto_align` работает через `WidgetConfig::AlignRight` sentinel + `align_marker: bool` в `Segment`
- [ ] `flex_mode` truncate с ellipsis (`…`); `compact_threshold < term_width` форсит `minimalist_mode`
- [ ] `Skills` widget читает `TranscriptStats.skill_names` (Vec<String>, sorted/unique); `FORMAT_VERSION 1→2` silent reset
- [ ] `claude_account_email()` через `OnceLock<Option<ClaudeJson>>`; missing `~/.claude.json` → `None`
- [ ] `cargo build --release --locked` зелёный; binary < 9 MB
- [ ] `cargo test --locked` зелёный (≥190 тестов суммарно)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] Phase 4 snapshot'ы (`tests/snapshots.rs`) — byte-identical после миграции (regression alarm)
- [ ] Phase 6 snapshot'ы (`tests/snapshots_transcript.rs`) — byte-identical
- [ ] CI matrix зелёный (macos / ubuntu / windows)
- [ ] Hyperfine: cchud-8w < 5 ms p95, cchud-60w < 12 ms p95, cold parse 50 MB < 11 ms, warm + 1 MB append < 3.5 ms
- [ ] Никаких `unwrap`/`expect` в `src/widgets/{env,usage}.rs`, `src/commands/env_loader.rs`, `src/render/flex.rs`
- [ ] Manual battle-test ≥ 5 мин, лог в `plan/phase-7/manual-test-log.md`
- [ ] `git tag v0.5.0` создан и запушен
- [ ] `docs/widgets.md` 60/60 DONE; README таблица supported 60/60

## Self-Review Notes

**Spec coverage:** все 14 решений из spec § "Дизайн-решения" покрыты задачами:
- Decision 1 (payload-only) → T2 + T8.
- Decision 2 (Full scope, 0.5.0) → T1–T14.
- Decision 3 (Skills как simple counter) → T7 + T10.
- Decision 4 (WidgetItem wrapper) → T3.
- Decision 5 (caller-loop multi-line) → T11 + T12.
- Decision 6 (formats) → T5 + T8 + T9 + T10.
- Decision 7 (override priority) → T4 (apply_widget_style).
- Decision 8 (AlignRight sentinel) → T4 + T11.
- Decision 9 (compact_threshold) → T11 (Plain + Powerline render_line).
- Decision 10 (skill_names как Vec<String>) → T7.
- Decision 11 (FORMAT_VERSION 1→2) → T7.
- Decision 12 (OnceLock<ClaudeJson>) → T6.
- Decision 13 (sysinfo) → T1 + T9.
- Decision 14 (flex_mode truncate с …) → T11.

**Type consistency:**
- `WidgetItem.kind` / `WidgetItem.style` consistent across T3/T4/T11/T12.
- `WidgetStyleOverride` поля `color`/`background_color`/`bold` consistent с upstream `WidgetItem`.
- `apply_widget_style` signature consistent T4 → T11 → T12.
- `Renderer::render_line(segs, &mut state, &theme)` — единая сигнатура T11/T12/T13.
- `Segment.align_marker` — добавлено в T11; используется в T11 (Powerline) + T12 (main.rs).

**Risks (см. spec § Risks):** snapshot regression на Phase 4 — T11 step 12 включает явный гейт; если diff — STOP. `unwrap`/`expect` lint enforce — на уровне модуля в `src/widgets/{env,usage}.rs`, `src/commands/env_loader.rs`, `src/render/flex.rs` (`#![deny(clippy::unwrap_used, clippy::expect_used)]`). Format-bump silent reset — Phase 6 уже умеет, тест existing продолжает работать после bump 1→2.

## Связи

- **Spec (источник truth):** [`docs/superpowers/specs/2026-04-29-phase-7-other-widgets-design.md`](../../docs/superpowers/specs/2026-04-29-phase-7-other-widgets-design.md)
- **Outline (predecessor):** [`plan/phase-7-other-widgets.md`](../phase-7-other-widgets.md) — оригинальный outline; scope скорректирован 15+ → 7 widgets + theme overrides.
- **PRD:** [`docs/prd-cchud.md`](../../docs/prd-cchud.md) — REQ-001 (60+ widgets parity), NFR §1 (cold start < 5ms p95).
- **Widgets registry:** [`docs/widgets.md`](../../docs/widgets.md) — canonical список 60 виджетов.
- **Predecessor:** [`plan/phase-6/`](../phase-6/) — JSONL cache + 8 transcript widgets (0.4.0).
- **Successor:** Phase 8 (TUI configurator) — поднимает `WidgetItem` + theme overrides в редактор.
