# Фаза 9 — Implementation Plan (Distribution + 1.0.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Каноническая копия плана:** [`docs/superpowers/plans/2026-05-02-phase-9-distribution.md`](../../docs/superpowers/plans/2026-05-02-phase-9-distribution.md). Этот README дублирует high-level + содержит ссылки на все 14 task-файлов в этой директории.

**Goal:** Релиз `1.0.0` cchud — `npx --yes cchud@1.0.0 install` (primary), `curl install.sh | sh` (secondary), self-relocation в `~/.local/bin/cchud`, новая команда `cchud doctor`, RC soak gating.

**Architecture:** 14 vertical-slice tasks, TDD. Слой 1 (T1) — bump + scaffolding. Слой 2 (T2–T4) — расширение `cchud install`. Слой 3 (T5–T6) — `cchud doctor`. Слой 4 (T7–T9) — npm-пакетинг. Слой 5 (T10) — install.sh. Слой 6 (T11) — release.yml. Слой 7 (T12) — docs. Слой 8 (T13–T14) — RC soak → 1.0.0.

**Tech Stack:** Rust 1.86 (без новых deps), Node 20 для JS shim (~50 LOC, без deps), POSIX sh (`install.sh` ~80 LOC), GitHub Actions matrix × 5 платформ.

---

## DAG задач

```
T1 ──► T2 ──► T3 ──► T4 ──► T5 ──► T6 ──► T7 ──► T8 ──► T9 ──► T10 ──► T11 ──► T12 ──► T13 (RC soak ≥24h) ──► T14 (1.0.0)
```

| # | Файл | Цель | Гейт |
|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | Cargo `1.0.0-rc.1`; scaffolding `src/commands/doctor.rs`, `npm/`, `scripts/`; DECISIONS | оба `cargo build` зелёные |
| 2 | [`task-2-relocate-helpers.md`](./task-2-relocate-helpers.md) | `canonical_target_path`/`same_file`/`relocate_to`/`check_path_or_warn` + tests | ≥8 unit tests PASS |
| 3 | [`task-3-install-integration.md`](./task-3-install-integration.md) | `cchud install` self-relocates в `~/.local/bin/cchud`; `--no-relocate` flag | idempotent re-run проходит |
| 4 | [`task-4-settings-backup.md`](./task-4-settings-backup.md) | `<path>.bak.<unix-ms>` перед write settings.json | backup test PASS |
| 5 | [`task-5-doctor-module.md`](./task-5-doctor-module.md) | `src/commands/doctor.rs` с 9 проверками; exit codes 0/1/2 | ≥6 unit tests PASS |
| 6 | [`task-6-doctor-wiring.md`](./task-6-doctor-wiring.md) | `cchud doctor` subcommand в main.rs + help text | smoke `cchud doctor --help` |
| 7 | [`task-7-npm-templates.md`](./task-7-npm-templates.md) | `npm/cchud/`, `npm/cli-*/` × 5 + JS shim ~50 LOC | `node bin/cchud.js` smoke OK |
| 8 | [`task-8-shim-tests.md`](./task-8-shim-tests.md) | `tests/npm_shim.rs` ≥3 кейсов | tests PASS |
| 9 | [`task-9-build-script.md`](./task-9-build-script.md) | `scripts/build-npm-packages.sh` | dry-run zelёный |
| 10 | [`task-10-install-sh.md`](./task-10-install-sh.md) | `install.sh` POSIX + `tests/install_sh.rs` | ≥4 tests PASS |
| 11 | [`task-11-release-yml.md`](./task-11-release-yml.md) | `.github/workflows/release.yml` 4 jobs + SHA-pinned actions | actionlint clean |
| 12 | [`task-12-docs.md`](./task-12-docs.md) | README + `MIGRATION.md` + CHANGELOG + ATTRIBUTION | links resolve |
| 13 | [`task-13-rc-soak.md`](./task-13-rc-soak.md) | tag `v1.0.0-rc.1` → npm `next` + manual soak ≥ 24h × 4 envs | soak log зелёный |
| 14 | [`task-14-final-release.md`](./task-14-final-release.md) | tag `v1.0.0` → npm `latest` + DoD verification | DoD pass |

## Pre-flight (требования до старта Task 1)

- **Phase 8 завершена** — `git tag --list 'v0.9.0'` показывает тег.
- **Текущая директория:** `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое:** `git status` пусто.
- **`rustc` ≥ 1.86**.
- **GitHub secrets:** `NPM_TOKEN` (тип `--type=automation`).
- **npm 2FA:** `npm profile enable-2fa auth-and-writes` для `IGoRFonin`.
- **Имя `cchud` доступно на npm:** `npm view cchud` → 404. Если занято — defensive registration ДО старта T1.

## Definition of Done всей Фазы 9

См. [`docs/superpowers/plans/2026-05-02-phase-9-distribution.md`](../../docs/superpowers/plans/2026-05-02-phase-9-distribution.md) § "Definition of Done".

Краткая сводка:

- [ ] `release.yml` зелёный на тегах `v1.0.0-rc.1` и `v1.0.0`.
- [ ] `npx --yes cchud@1.0.0 install` работает на mac/linux/windows.
- [ ] `curl install.sh | sh` работает на mac/linux gnu/musl.
- [ ] `cchud doctor` exits 0 на свежем install.
- [ ] `~/.local/bin/cchud` стабилен через nvm switches.
- [ ] `cargo test --locked` зелёный (≥230 tests).
- [ ] `cargo clippy --locked --all-targets -- -D warnings` зелёный.
- [ ] CI matrix зелёный (mac/linux/win).
- [ ] Manual soak ≥ 24h × 4 environment'ах — лог в [`./manual-soak-log.md`](./manual-soak-log.md).
- [ ] npm 2FA + `--type=automation` token + SHA-pinned actions + `--provenance`.
- [ ] `git tag v1.0.0` пушнут.

## Связи

- **Spec:** [`docs/superpowers/specs/2026-05-02-phase-9-distribution-design.md`](../../docs/superpowers/specs/2026-05-02-phase-9-distribution-design.md).
- **Canonical plan:** [`docs/superpowers/plans/2026-05-02-phase-9-distribution.md`](../../docs/superpowers/plans/2026-05-02-phase-9-distribution.md).
- **Outline:** [`plan/phase-9-distribution.md`](../phase-9-distribution.md).
- **PRD:** [`docs/prd-cchud.md`](../../docs/prd-cchud.md).
- **Predecessor:** [`plan/phase-8/`](../phase-8/) — 0.9.0 (TUI + import).
- **Successor:** [`plan/phase-10-future.md`](../phase-10-future.md) — auto-update, winget/scoop, AUR, Nix, ARM Linux, code-signing, install.ps1.
