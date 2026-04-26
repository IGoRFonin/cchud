# Фаза 1 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Создать публикуемый skeleton-репозиторий cchud — Cargo-проект на MSRV 1.85, с CI на 3 OS, snapshot-тестами на payload-семплах из Фазы 0, документацией и приватным репозиторием на GitHub с зелёным CI.

**Architecture:** Пять последовательных задач. Каждая = checkpoint, изолирована в своём файле, заканчивается verification-гейтом и git commit'ом. Финальная задача (publish) — единственная с stop-and-confirm перед действием с удалённым state.

**Tech Stack:** Rust 1.85 (Edition 2024), Cargo, GitHub Actions, hyperfine (bench), actionlint (yaml-lint), Dependabot, gh CLI.

---

## DAG задач

```
Task 1 (Bootstrap) ──► Task 2 (Skeleton) ──► Task 3 (CI) ──► Task 4 (Docs) ──► Task 5 (Publish)
```

Линейная цепочка. Между задачами — review-checkpoint.

| # | Файл | Цель | Гейт |
|---|---|---|---|
| 1 | [`task-1-bootstrap.md`](./task-1-bootstrap.md) | Cargo проект, deps, toolchain, .gitignore | `cargo build --release --locked` зелёный |
| 2 | [`task-2-skeleton.md`](./task-2-skeleton.md) | Skeleton main.rs + 12 snapshot-тестов на payload-семплах | `cargo test --locked` все 12 тестов зелёные |
| 3 | [`task-3-ci.md`](./task-3-ci.md) | GitHub Actions CI matrix + bench + dependabot + actionlint | `actionlint .github/workflows/*.yml` exit 0 + `bash benches/run.sh` отрабатывает |
| 4 | [`task-4-docs.md`](./task-4-docs.md) | README, LICENSE, ATTRIBUTION, CHANGELOG + правки PRD §1/§2 (R1/R2) + R3 fix регистра | grep-проверки доков пройдены |
| 5 | [`task-5-publish.md`](./task-5-publish.md) | git push в `IGoRFonin/cchud` (private) + DECISIONS update + ждать зелёный CI | 3 зелёных CI run'а (macos/ubuntu/windows) |

## Pre-flight (требования до старта Task 1)

- Локальный rustc ≥ 1.85 (есть 1.87 через brew — DECISIONS Phase 0).
- `gh` CLI установлен и залогинен под `IGoRFonin` (`gh auth status`).
- `hyperfine` установлен (DECISIONS Phase 0: 1.20.0).
- `actionlint` установить: `brew install actionlint` (Task 3 step 1 это явно делает).
- Текущая директория: `/Users/igor/mp/startup/cchud`.
- `benches/samples/` содержит ≥ 12 JSON-payload'ов (Phase 0 артефакт).

## Definition of Done всей Фазы 1

- [ ] Все 5 задач завершены, verification-гейты прошли
- [ ] Локально: `cargo build --release --locked && cargo test --locked && cargo clippy --locked -- -D warnings && cargo fmt --check` зелёные
- [ ] `git log --oneline` содержит коммиты от всех 5 задач
- [ ] Удалённо: GitHub Actions workflow `CI` зелёный на macos-latest + ubuntu-latest + windows-latest
- [ ] `docs/DECISIONS.md` дополнен записью «Phase 1 → private repo, public migration manual»
- [ ] PRD §1 содержит фактический baseline (246.7 / 827.7 мс), PRD §5 не содержит «60+ widgets»
- [ ] `plan/README.md` отмечает Фазу 1 как `[x]`

## Связи

- **Spec этого плана:** [`../../docs/superpowers/specs/2026-04-26-phase-1-writing-plan-design.md`](../../docs/superpowers/specs/2026-04-26-phase-1-writing-plan-design.md)
- **Исходный intent:** [`../phase-1-init.md`](../phase-1-init.md) — оригинальный детальный план (этот writing-plan его реализует)
- **Phase 0 артефакты:** [`../../benches/samples/`](../../benches/samples/), [`../../benches/baseline.md`](../../benches/baseline.md), [`../../docs/DECISIONS.md`](../../docs/DECISIONS.md)
- **PRD:** [`../../docs/prd-cchud.md`](../../docs/prd-cchud.md)
