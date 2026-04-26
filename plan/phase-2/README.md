# Фаза 2 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Первый рабочий бинарь cchud с виджетом `Model`, end-to-end в реальном Claude Code, p95 < 5ms на M-серии. Архитектурный скелет, на который Phases 3–9 наращивают виджеты без переделки ядра.

**Architecture:** 10 последовательных задач, каждая ~1–3 часа, single concern, single git commit. T1–T5 строят walking skeleton (deps → payload → widgets → render → smoke test). T6–T7 добавляют config layer. T8–T9 — `cchud install` + 5 install-тестов. T10 — full-envelope payload + insta snapshots. T11 (бывший T10 в spec) — verification & sign-off.

**Tech Stack:** Rust 1.85 (Edition 2024), serde/serde_json, dirs 6, insta 1.x (insta::glob), assert_cmd 2, predicates 3, serial_test 3 (новая dev-dep), tempfile 3 (новая dev-dep), hyperfine 1.20+.

---

## DAG задач

```
T1 (deps) ──► T2 (payload min) ──► T3 (widget trait) ──► T4 (Model+Plain+main: smoke)
                                                                   │
                                                                   ▼
T5 (config types) ──► T6 (config load + main wired) ──► T7 (install + args)
                                                                   │
                                                                   ▼
T8 (install tests) ──► T9 (full envelope + snapshots) ──► T10 (bench + manual CC + sign-off)
```

Линейная цепочка. Между задачами — review checkpoint.

| # | Файл | Цель | Гейт |
|---|---|---|---|
| 1 | [`task-1-deps.md`](./task-1-deps.md) | Cargo.toml: −lexopt, +serial_test/tempfile (dev), tighten clippy unwrap=deny в hot path | `cargo build --locked` зелёный, `cargo clippy --locked -- -D warnings` зелёный |
| 2 | [`task-2-payload-min.md`](./task-2-payload-min.md) | `src/types/payload.rs` минимальный: StatusPayload + ModelInfo + Workspace + serde-derive + 1 unit-тест парсит реальный семпл | `cargo test --locked test_payload_min` зелёный |
| 3 | [`task-3-widget-trait.md`](./task-3-widget-trait.md) | `src/widgets/mod.rs` — Widget trait + RenderContext + `build_widgets` skeleton (без implementations пока) | `cargo build --locked` зелёный (компиляция) |
| 4 | [`task-4-smoke.md`](./task-4-smoke.md) | `src/widgets/model.rs` + `src/render/mod.rs` (Renderer + Plain) + `src/main.rs` render_pipeline (hardcoded default-line) → end-to-end smoke | `echo '{...}' \| cargo run --release` печатает "Sonnet 4.6" |
| 5 | [`task-5-config-types.md`](./task-5-config-types.md) | `src/types/config.rs`: Settings + Line + WidgetConfig::Model + ThemeConfig (stub) + serde roundtrip-тест | `cargo test --locked test_settings_serde` зелёный |
| 6 | [`task-6-config-load.md`](./task-6-config-load.md) | `src/config/mod.rs`: load + `Settings::default_line` + graceful fallback; `main.rs` использует `config::load()`; 4 fallback-теста | `cargo test --locked config::` зелёный |
| 7 | [`task-7-install.md`](./task-7-install.md) | `src/commands/install.rs` (detect + --force + atomic write) + `main.rs` args dispatch (`--version`, `install`, `--help`) | `cchud --version` печатает версию; `cchud install --help`-стиль (если есть) ОК |
| 8 | [`task-8-install-tests.md`](./task-8-install-tests.md) | `tests/install.rs`: 5 кейсов (creates / refuses / --force / preserves / repeat) с `serial_test` + `tempfile` HOME override | `cargo test --locked --test install` зелёный (5 тестов) |
| 9 | [`task-9-snapshots.md`](./task-9-snapshots.md) | Расширить `payload.rs` до полного envelope (15 полей, heavy → Option<Value>); `tests/snapshots.rs` (insta::glob + AC-007 graceful); `cargo insta review` → committed | `cargo test --locked --test snapshots` зелёный, `.snap` файлы committed |
| 10 | [`task-10-verify.md`](./task-10-verify.md) | hyperfine cchud vs ccstatusline; manual real-CC test; `plan/phase-2/manual-test-log.md`; sign-off Exit Criteria; `git tag phase-2-pipeline` | Все Exit Criteria ✓; tag pushed; plan/README обновлён |

## Pre-flight (требования до старта Task 1)

- **Phase 1 завершена полностью**: Phase 1 Task 5 (publish + 3 зелёных CI run) сделан, `plan/phase-1/` все таски `[x]`, `plan/README.md` отмечает Phase 1 как `[x]`.
- **CI matrix зелёный** на macos-latest + ubuntu-latest + windows-latest на ветке `master`. Проверить: `gh run list --branch master --limit 3` — последние 3 run'а success.
- **Локальный rustc** ≥ 1.85 (есть 1.87 через brew, см. DECISIONS Phase 0).
- **`hyperfine`** ≥ 1.20.0 (Phase 0 артефакт).
- **`gh` CLI** залогинен под `IGoRFonin` (`gh auth status`).
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **`benches/samples/`** содержит ≥ 4 JSON-payload'а (`payload-cchud-*.json`, `payload-posts-*.json` — Phase 0 артефакт).
- **Установлен Claude Code** для manual-теста в Task 10. Запустимый локально, активная сессия не критична.

## Definition of Done всей Фазы 2

- [ ] Все 10 задач завершены, verification-гейты прошли
- [ ] Локально: `cargo build --release --locked && cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check` зелёные
- [ ] `git log --oneline | head -10` содержит коммиты от всех 10 задач, каждый с префиксом `feat(phase-2):` / `test(phase-2):` / `chore(phase-2):`
- [ ] CI matrix зелёный после push'а каждого коммита
- [ ] `cchud --version` печатает версию из `Cargo.toml`
- [ ] `cchud install` обрабатывает три кейса: нет файла / чужой statusLine / cchud уже стоит
- [ ] AC-007 покрыт автоматизированным тестом (битый JSON → exit 0, пустой stdout)
- [ ] AC-008 покрыт install-тестами (preserves unrelated keys)
- [ ] Hyperfine: p95 cchud < 5ms на M-серии, `benches/phase-2.md` сохранён в репо
- [ ] Manual real-CC test пройден ≥ 5 минут, лог в `plan/phase-2/manual-test-log.md`
- [ ] `git tag phase-2-pipeline` создан и запушен
- [ ] `plan/README.md` отмечает Phase 2 как `[x]`, Phase 3 как `[~]`

## Связи

- **Spec этого плана:** [`../../docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md`](../../docs/superpowers/specs/2026-04-26-phase-2-pipeline-design.md)
- **Исходный intent:** [`../phase-2-pipeline.md`](../phase-2-pipeline.md) — оригинальный outline
- **PRD:** [`../../docs/prd-cchud.md`](../../docs/prd-cchud.md) — AC-001, AC-007, AC-008
- **Phase 0 артефакты:** [`../../benches/samples/`](../../benches/samples/), [`../../benches/baseline.md`](../../benches/baseline.md), [`../../docs/upstream-map.md`](../../docs/upstream-map.md)
- **Phase 1 plan:** [`../phase-1/`](../phase-1/) — предшественник, тот же стиль
