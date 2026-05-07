# Фаза 3 — Implementation Plan (MVP Widgets, 0.1.0-alpha)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Релиз `0.1.0-alpha` cchud — 23 payload/env/static-виджета поверх Phase 2 walking skeleton, p95 < 5 ms на 23-widget config, без git/transcript/HTTP. Архитектурный задел Phase 5 (git), Phase 6 (transcript) и Phase 7 (env/http) остаётся нетронут.

**Architecture:** Линейная цепочка из 9 vertical-slice задач. T1 (setup) лочит payload sub-types, kebab-case, synthetic-семплы, util-skeleton. T2–T7 — 7 кластерных задач, каждая даёт работающий end-to-end набор виджетов (static → trivial → context → cost → worktree → custom-command). T8 — snapshots+hyperfine gate. T9 — release (alpha tag, README, CHANGELOG, manual real-CC test). Один файл = один кластер виджетов; cross-widget хелперы изолированы в `src/util/`.

**Tech Stack:** Rust 1.85 (Edition 2024), serde/serde_json (Phase 2), `wait-timeout 0.2` (новая dep, T1, runtime), insta 1.x + assert_cmd 2 (Phase 2 dev-deps), hyperfine 1.20+, `cargo-insta` локально, `cargo install ccstatusline` для baseline-сравнения (опционально, T8).

---

## DAG задач

```
T1 (setup) ──► T2 (static) ──► T3 (trivial) ──► T4 (context) ──► T5 (cost)
                                                                     │
                                                                     ▼
                                        T7 (custom-command) ◄── T6 (worktree)
                                                  │
                                                  ▼
                                        T8 (snapshots+bench) ──► T9 (release)
```

Линейная цепочка. Между задачами — review checkpoint (как в Phase 2). Каждая T2–T7 даёт работающий кластер (тесты зелёные, виджеты подключены через `widgets::build_one`).

| # | Файл | Цель | Виджеты | Гейт |
|---|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | `wait-timeout 0.2` dep; payload sub-types (CostInfo, ContextWindowInfo, CurrentUsage untagged, Worktree, VimState, OutputStyle); 2 synthetic-семпла; `WidgetConfig` rename_all=kebab-case + Phase 2 retrofit; 2 DECISIONS-записи; `src/util/mod.rs` skeleton | 0 (инфраструктура) | `cargo test types::payload` + `cargo test types::config` зелёные; Phase 2 snapshot'ы зелёные после kebab-retrofit |
| 2 | [`task-2-static.md`](./task-2-static.md) | `widgets/static_text.rs` (CustomText, CustomSymbol, Link); 3 enum-варианта; OSC-8 unit-тест на корректные escape-байты | 3 | `cargo test widgets::static_text` зелёный; widgets подключены в `build_one` |
| 3 | [`task-3-trivial.md`](./task-3-trivial.md) | `widgets/trivial.rs` (Version, ClaudeSessionId, TerminalWidth, OutputStyle, VimMode); `widgets/session.rs` (SessionName); 6 enum-вариантов | 6 | `cargo test widgets::trivial` + `widgets::session` зелёные |
| 4 | [`task-4-context.md`](./task-4-context.md) | `widgets/context.rs` (6 виджетов); `util/model_context_size.rs` (prefix-match + 5+ unit-тестов); `util/ascii_bar.rs` (formatter + edge tests) | 6 | `cargo test widgets::context` + `util::model_context_size` + `util::ascii_bar` зелёные |
| 5 | [`task-5-cost.md`](./task-5-cost.md) | `widgets/session.rs` дополняется (SessionClock, SessionCost); `util/duration.rs` (HH:MM:SS / MM:SS) | 2 | `cargo test widgets::session` + `util::duration` зелёные |
| 6 | [`task-6-worktree.md`](./task-6-worktree.md) | `widgets/worktree.rs` (5 виджетов); тесты используют `payload-synthetic-vim-worktree.json` | 5 | `cargo test widgets::worktree` зелёный |
| 7 | [`task-7-custom-command.md`](./task-7-custom-command.md) | `widgets/custom_command.rs` (subprocess + `wait-timeout`); cross-platform тесты (Unix only через `#[cfg(unix)]`); diagnostics policy | 1 | `cargo test widgets::custom_command` зелёный (Unix) |
| 8 | [`task-8-snapshots-bench.md`](./task-8-snapshots-bench.md) | `tests/snapshots.rs` расширен ≥5 сценариями; hyperfine на 23-widget config; `benches/phase-3.md`; CustomCommand bench-config с `echo` | 0 | p95 cchud-23w < 5 ms; insta review committed |
| 9 | [`task-9-release.md`](./task-9-release.md) | README обновление (таблица supported widgets 24/60); `CHANGELOG.md` 0.1.0-alpha; `Cargo.toml` version bump; `cargo build --release --locked`; `cchud --version` → `0.1.0-alpha`; `git tag v0.1.0-alpha`; `plan/README.md` Phase 3 → `[x]`, Phase 4 → `[~]`; manual-test-log.md шаблон | 0 | Tag запушен; CI matrix зелёный; `cchud --version` показывает alpha |

**Итого:** 23 виджета, ~20–26 часов, ~3–5 рабочих дней — соответствует "1 неделя" из `phase-3-mvp.md`.

## Pre-flight (требования до старта Task 1)

- **Phase 2 завершена полностью**: `plan/phase-2/` все таски `[x]`, `phase-2-pipeline` git tag запушен, CI matrix зелёный 3 раза подряд на ветке `master`. Проверить: `gh run list --branch master --limit 3` — последние 3 run'а success.
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое**: `git status` пусто.
- **Локальный rustc** ≥ 1.85 (см. `rust-toolchain.toml`).
- **`hyperfine`** ≥ 1.20.0 (Phase 0 артефакт).
- **`cargo-insta`** установлен (`cargo install cargo-insta`) — для интерактивного `cargo insta review` в T8.
- **`gh` CLI** залогинен под `IGoRFonin` (`gh auth status`).
- **`benches/samples/`** содержит 4 Phase-0 JSON-payload'а (`payload-cchud-*.json`, `payload-posts-*.json`).
- **Установлен Claude Code** для manual-теста в Task 9. Активная сессия в worktree с включённым vim mode желательна; шаблон лога создаётся в Task 1.

## Definition of Done всей Фазы 3

- [ ] 23 виджета реализованы; каждый покрыт unit-тестами (happy + edge, минимум 2 теста на виджет)
- [ ] `cargo build --release --locked` зелёный; binary < 5 MB
- [ ] `cargo test --locked` зелёный (unit + integration + snapshot, ≥80 тестов суммарно)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] `cargo insta review` — все snapshot'ы committed без `.snap.new`
- [ ] CI matrix зелёный (macos/ubuntu/windows; `#[cfg(unix)]` на CustomCommand тестах)
- [ ] Hyperfine p95 < 5 ms на 23-widget config; `benches/phase-3.md` сохранён в репо
- [ ] `cchud --version` печатает `0.1.0-alpha`
- [ ] `cchud install` продолжает работать (Phase 2 регрессия не сломана)
- [ ] AC-007 покрыт автоматизированным тестом для каждого нового payload sub-type
- [ ] Manual real-CC test пройден ≥ 5 минут с активным vim+worktree, лог в `plan/phase-3/manual-test-log.md`
- [ ] `git tag v0.1.0-alpha` создан и запушен
- [ ] `plan/README.md` отмечает Phase 3 как `[x]`, Phase 4 как `[~]`
- [ ] README таблица "supported widgets" с 24/60 галочками
- [ ] `CHANGELOG.md` запись 0.1.0-alpha
- [ ] 2 записи в `docs/DECISIONS.md` (kebab-case retrofit, Phase 3 типизация cost/context_window)

## Связи

- **Spec этого плана:** [`../../docs/superpowers/specs/2026-04-26-phase-3-mvp-design.md`](../../docs/superpowers/specs/2026-04-26-phase-3-mvp-design.md)
- **Исходный intent:** [`../phase-3-mvp.md`](../phase-3-mvp.md) — оригинальный outline (расходится в scope: 10→23 виджетов, JSONL-кэш отложен в Phase 6)
- **PRD:** [`../../docs/prd-cchud.md`](../../docs/prd-cchud.md) — AC-003, AC-007, REQ-006, REQ-102, REQ-103
- **Predecessor plan:** [`../phase-2/`](../phase-2/) — стиль и pre-flight конвенции
- **Predecessor spec:** [`../../docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md`](../../docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md)
- **Widgets registry:** [`../../docs/widgets.md`](../../docs/widgets.md) — canonical список 60 виджетов; Phase 3 = 23 строки
- **Upstream map:** [`../../docs/upstream-map.md`](../../docs/upstream-map.md) — StatusJSON/Settings/Widget схемы
- **Phase 0 артефакты:** [`../../benches/samples/`](../../benches/samples/), [`../../docs/DECISIONS.md`](../../docs/DECISIONS.md)
- **Future phases:**
  - Phase 4 — Powerline-рендерер + `ThemeConfig` для тех же 23 виджетов
  - Phase 5 — `git: OnceCell<Option<GitInfo>>` + 20 git-виджетов
  - Phase 6 — типизация `rate_limits`, JSONL-кэш, `thinking-effort`, 8 transcript-виджетов
  - Phase 7 — env/http-кластер (6 виджетов), формат-строки, OSC 8 graceful detect
