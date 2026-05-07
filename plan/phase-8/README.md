# Фаза 8 — Implementation Plan (TUI Configurator + Import, 0.9.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Каноническая копия плана:** [`docs/superpowers/plans/2026-05-01-phase-8-tui-design.md`](../../docs/superpowers/plans/2026-05-01-phase-8-tui-design.md). Этот README дублирует high-level + содержит ссылки на все 14 task-файлов в этой директории.

**Goal:** Релиз `0.9.0` cchud — `cchud configure` (interactive ratatui TUI) и `cchud import` (CLI миграция с ccstatusline).

**Architecture:** Линейная цепочка из 14 vertical-slice задач. Слой 1 (T1–T2) — фундамент: feature flag + skeleton + `PartialEq` derive chain + **критический refactor `compose_line`** (STOP-gate на Phase 4 + Phase 7 snapshot regression). Слой 2 (T3–T5) — pure helpers. Слой 3 (T6–T7) — TUI core (App + reducer + ≥15 unit-тестов). Слой 4 (T8–T10) — UI: embedded widgets + 4 panels + 3 overlays. Слой 5 (T11–T13) — IO: atomic_save + event loop + commands::{configure, import}. Слой 6 (T14) — verification + release.

**Tech Stack:** Rust 1.85, `ratatui = "=0.30.0"` + `crossterm = "=0.29.0"` (optional, feature `tui`, default), `tempfile = "3"` (optional, поднято из dev-deps).

---

## DAG задач

```
T1 ──► T2 (compose_line STOP-gate) ──► T3 (style_map) ──► T4 (sample) ──► T5 (widget_meta)
                                                                                  │
                                                                                  ▼
                                                                T6 (App) ──► T7 (reducer + 15 tests)
                                                                                  │
                                                                                  ▼
                                                       T8 (widgets_ui) ──► T9 (panels) ──► T10 (overlays + ui::draw)
                                                                                  │
                                                                                  ▼
                                                T11 (atomic_save) ──► T12 (event loop + run_configure)
                                                                                  │
                                                                                  ▼
                                                       T13 (configure + import + main.rs)
                                                                                  │
                                                                                  ▼
                                                       T14 (snapshots + import tests + release 0.9.0)
```

| # | Файл | Цель | Гейт |
|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | Cargo deps + features + skeleton + PartialEq chain + DECISIONS | оба `cargo build` зелёные; PartialEq compiles |
| 2 | [`task-2-compose-line.md`](./task-2-compose-line.md) | Extract `Renderer::compose_line` pure | **Phase 4 + Phase 7 snapshots byte-identical** (STOP) |
| 3 | [`task-3-style-map.md`](./task-3-style-map.md) | `tui::style_map::to_span` | 6 unit-tests PASS |
| 4 | [`task-4-sample.md`](./task-4-sample.md) | `tui::sample::payload` + tempfile transcript | 4 unit-tests PASS |
| 5 | [`task-5-widget-meta.md`](./task-5-widget-meta.md) | `tui::widget_meta::ALL_KINDS` 60 entries | 5 tests PASS; len == 60 |
| 6 | [`task-6-app-state.md`](./task-6-app-state.md) | `tui::app::{App, Mode, Pane, EditField}` | 7 tests PASS; dirty diff via PartialEq |
| 7 | [`task-7-reducer.md`](./task-7-reducer.md) | Pure `handle_key` + ≥15 unit-тестов | ≥18 tests PASS |
| 8 | [`task-8-widgets-ui.md`](./task-8-widgets-ui.md) | input/tri_bool/color_picker/number_input/list_editor | unit tests PASS |
| 9 | [`task-9-panels.md`](./task-9-panels.md) | Lines/Palette/Settings/Preview panels | `cargo build --features tui` PASS |
| 10 | [`task-10-overlays.md`](./task-10-overlays.md) | Themes/Help/Modal + `ui::draw` | `cargo build --features tui` PASS |
| 11 | [`task-11-save.md`](./task-11-save.md) | `atomic_save` + backup | 5 tempdir tests PASS |
| 12 | [`task-12-event-loop.md`](./task-12-event-loop.md) | TerminalGuard + run_event_loop + run_configure | `cargo build --features tui` PASS |
| 13 | [`task-13-commands.md`](./task-13-commands.md) | configure + import + main.rs branching | smoke: `cchud configure < /dev/null` exit 2 |
| 14 | [`task-14-tests-release.md`](./task-14-tests-release.md) | TUI snapshots ≥6 + import ≥6 + bench + release 0.9.0 | `git tag v0.9.0`; binary < 10 MB |

## Pre-flight (требования до старта Task 1)

- **Phase 7 завершена** — релиз `0.5.0` запушен, `git tag --list 'v0.5.0'` показывает тег.
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое**: `git status` пусто.
- **`rustc` ≥ 1.85** (см. `rust-toolchain.toml`).
- **`hyperfine` ≥ 1.20.0** (для T14).
- **`cargo-insta`** установлен (`cargo install cargo-insta`).
- **Phase 4 snapshot baseline**: `cargo test --test snapshots` зелёный.
- **Phase 7 snapshot baseline**: `cargo test --test snapshots_phase7` зелёный.
- **`benches/samples/payload-cchud-sonnet-xlarge.json`** — на месте.

## Definition of Done всей Фазы 8

См. [`docs/superpowers/plans/2026-05-01-phase-8-tui-design.md`](../../docs/superpowers/plans/2026-05-01-phase-8-tui-design.md) § "Definition of Done".

Краткая сводка:

- [ ] `cchud configure` запускает 4-панельный TUI; add/delete/reorder работают.
- [ ] Settings panel: color picker (named + hex), tri-state bold, custom params.
- [ ] Themes/Help overlays работают; ConfirmQuit modal на dirty.
- [ ] `cchud import` импортирует ccstatusline (best-effort + warn); `--from`/`--then-configure`/`--force`.
- [ ] **Phase 4 + Phase 7 snapshots byte-identical** после `compose_line` refactor.
- [ ] ≥15 reducer tests, ≥6 TUI snapshot tests, ≥7 import scenarios, configure-no-TTY smoke.
- [ ] `cargo build --release --locked` зелёный (binary < 10 MB).
- [ ] `cargo build --release --locked --no-default-features` зелёный (binary < 8.5 MB).
- [ ] `cargo test --locked` зелёный (≥220 tests).
- [ ] `cargo clippy --locked --all-targets -- -D warnings` зелёный.
- [ ] CI matrix зелёный (macos / ubuntu / windows).
- [ ] Никаких `unwrap`/`expect` в `src/tui/`, `src/commands/{configure, import}.rs`, `src/render/mod.rs::compose_line`.
- [ ] Manual battle-test ≥ 5 мин — лог в [`./manual-test-log.md`](./manual-test-log.md).
- [ ] `git tag v0.9.0`.

## Связи

- **Spec:** [`docs/superpowers/specs/2026-05-01-phase-8-tui-design.md`](../../docs/superpowers/specs/2026-05-01-phase-8-tui-design.md).
- **Outline:** [`plan/phase-8-tui.md`](../phase-8-tui.md).
- **PRD:** [`docs/prd-cchud.md`](../../docs/prd-cchud.md) — REQ-100.
- **Predecessor:** [`plan/phase-7/`](../phase-7/) — 60/60 widgets + per-widget/global overrides + multi-line (0.5.0).
- **Successor:** [`plan/phase-9-distribution.md`](../phase-9-distribution.md) — npm/brew/install.sh + cross-platform release.
