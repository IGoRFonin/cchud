# Task 4 — Docs (README + LICENSE + ATTRIBUTION + CHANGELOG + PRD updates)

**Files:**
- Create: `LICENSE`, `README.md`, `ATTRIBUTION.md`, `CHANGELOG.md`
- Modify: `docs/prd-cchud.md` (R1 verify, R2 fix), `docs/widgets.md` (R2 minor)

## Goal

Минимальный набор репо-доков (MIT LICENSE, public-facing README, явная атрибуция upstream, CHANGELOG) + закрытие двух Phase-0 deferred-апдейтов: R1 (PRD §1 baseline) и R2 (точное число виджетов).

## Inputs

- Task 3 завершён: CI/bench/dependabot настроены.
- Текущая дата: 2026-04-26.
- Email: `menotoa1@gmail.com` (auto memory).
- GitHub username каноничный: `IGoRFonin`.

---

- [ ] **Step 1: Создать `LICENSE` (MIT)**

```text
MIT License

Copyright (c) 2026 Igor Fonin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Создать `README.md`**

```markdown
# cchud

> Fast Rust statusline for Claude Code CLI — drop-in port of [ccstatusline](https://github.com/sirmalloc/ccstatusline) targeting < 5 ms cold-start and < 5 MB RSS.

**Status:** WIP — Phase 1 (skeleton + CI). Not yet usable. See [`plan/README.md`](plan/README.md) for roadmap.

## What

`cchud` aims to be a feature-parity Rust port of `ccstatusline`. It reads the JSON payload Claude Code pipes to `statusLine.command` on every prompt, then renders a one-line statusbar — model name, git state, token usage, cost-per-block, etc.

## Why

`ccstatusline` is excellent but Node.js cold-start dominates per-prompt latency. Phase 0 measurement on Apple M4 Pro (node v24.13.0, ccstatusline 2.2.8, 200 iterations hyperfine):

| Mode | Mean [ms] | RSS peak |
|---|---:|---:|
| `npx -y ccstatusline@latest` | 827.7 | — |
| `ccstatusline` (global install) | 246.7 | 97.7 MB |

Claude Code invokes the statusline up to 3 times per second. For 4 parallel sessions that is ≈ 3 calls/s × 247 ms × 4 ≈ 296% of a single core. `cchud` targets ≤ 5 ms cold-start and ≤ 5 MB RSS — a ~50× / ~20× reduction respectively.

## Status

This is a Phase 1 skeleton: project scaffolding, CI matrix on macOS + Ubuntu + Windows, snapshot-test harness over Phase 0 payload fixtures. The binary currently prints `cchud (skeleton) | input bytes: N` — no widgets, no rendering. Real pipeline lands in Phase 2.

## Documents

- **PRD:** [`docs/prd-cchud.md`](docs/prd-cchud.md)
- **Plan (per-phase):** [`plan/README.md`](plan/README.md)
- **Decisions log:** [`docs/DECISIONS.md`](docs/DECISIONS.md)
- **Upstream attribution:** [`ATTRIBUTION.md`](ATTRIBUTION.md)
- **Changelog:** [`CHANGELOG.md`](CHANGELOG.md)

## Build (developer)

```bash
git clone https://github.com/IGoRFonin/cchud
cd cchud
cargo build --release
./target/release/cchud < some-payload.json
```

Requires `rustc >= 1.85` (Edition 2024). MSRV is enforced via `rust-toolchain.toml` for `rustup` users.

## Troubleshooting

- **Windows long-paths:** if cargo build fails with "filename too long", run `git config --system core.longpaths true` once.

## License

MIT — see [`LICENSE`](LICENSE). Original `ccstatusline` is also MIT; see [`ATTRIBUTION.md`](ATTRIBUTION.md) for credits.
```

- [ ] **Step 3: Создать `ATTRIBUTION.md`**

```markdown
# Attribution

`cchud` is a Rust port of [`ccstatusline`](https://github.com/sirmalloc/ccstatusline) by **Matthew Breedlove ([@sirmalloc](https://github.com/sirmalloc))**.

The original `ccstatusline` is published under the MIT License. This project preserves the same license and aims for behavioural parity with `ccstatusline` 2.2.8.

The widget catalogue, configuration format (`~/.config/<name>/settings.json`), TUI design, and Powerline rendering conventions are all derived from `ccstatusline`'s design. `cchud` is an independent reimplementation that aims to deliver the same UX with a different runtime profile (single Rust binary instead of a Node.js process).

## Original project

- Repository: <https://github.com/sirmalloc/ccstatusline>
- Author: Matthew Breedlove (`@sirmalloc`)
- License: MIT
- Reference version: 2.2.8

## This port

- Repository: <https://github.com/IGoRFonin/cchud>
- Author: Igor Fonin (`@IGoRFonin`)
- License: MIT
```

- [ ] **Step 4: Создать `CHANGELOG.md` (N1)**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] — 2026-04-26

### Added

- Initial repository skeleton (`Cargo.toml`, `rust-toolchain.toml`, `.gitignore`).
- Skeleton binary: reads stdin, prints `cchud (skeleton) | input bytes: N`.
- Snapshot-test harness (`tests/snapshots.rs`) over Phase 0 payload fixtures (12 samples).
- GitHub Actions CI matrix (macOS + Ubuntu + Windows): build, test, clippy, fmt; gated by actionlint.
- Hyperfine bench workflow on PR (macOS-only).
- Dependabot for cargo + github-actions, weekly cadence.
- README, LICENSE (MIT), ATTRIBUTION (credits ccstatusline upstream), this CHANGELOG.

### Notes

- No widgets yet. Real pipeline lands in Phase 2.
- Repository is private; switch to public is a manual decision after content review.

[Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
```

- [ ] **Step 5: R1 — verify/update PRD §1 baseline**

Сначала проверить текущее состояние:

```bash
grep -E "828 ms|247 ms|97.7|Phase 0 measurement" docs/prd-cchud.md
```

**Если grep даёт несколько матчей** (PRD §1 уже содержит фактический baseline — обновлено во время Phase 0): R1 закрыто, никаких правок не нужно. Запиши факт в commit message ("R1: PRD §1 already up-to-date from Phase 0 edit").

**Если grep пуст или выдаёт только 1 матч** (старое ожидание «50–150 ms» осталось): открыть `docs/prd-cchud.md`, найти параграф "Resource usage" в секции "## 1. Problem Statement", заменить старый текст на:

> 1. **Resource usage.** The default configuration `npx -y ccstatusline@latest` consumes **828 ms CPU** and ~98 MB RAM on every render (Phase 0 measurement: Apple M4 Pro, node v24.13.0, ccstatusline 2.2.8, 200 iterations hyperfine; global install: 247 ms). On M-chip machines the fan is audible during long sessions; on battery the power budget impact is noticeable. With 4 parallel Claude Code sessions — up to 400% of a single core in the background (≈ 3 calls/sec × 247 ms × 4 sessions).

Цели cchud (`< 5 ms cold-start`, `< 5 MB RSS`) в PRD §2 не меняются — запас остаётся.

- [ ] **Step 6: R2 — fix "60+" → "60" в PRD**

```bash
grep -n "60+ widgets" docs/prd-cchud.md
```

Expected: 1 матч (line 50: `Existing ccstatusline users who value features (Powerline, themes, 60+ widgets)`).

Использовать Edit tool:
- `file_path`: `/Users/igor/mp/startup/cchud/docs/prd-cchud.md`
- `old_string`: `Powerline, themes, 60+ widgets`
- `new_string`: `Powerline, themes, 60 widgets`

Verify: `grep -c "60+ widgets" docs/prd-cchud.md` → 0.

(Замечание: `docs/widgets.md:97` содержит «60+» как мета-комментарий «PRD §2 говорит "60+" — соответствует». Не трогаем — это историческая запись Phase 0.)

- [ ] **Step 7: Обновить `plan/README.md` — отметить начало Фазы 1**

Открыть `plan/README.md`, в секции "Текущий статус" заменить строку:

```
- [ ] Фаза 1 — Init
```

на:

```
- [~] Фаза 1 — Init (in progress)
```

(Финальный `[x]` ставится в Task 5 после зелёного CI.)

- [ ] **Step 8: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное (нет изменений в Rust-коде).

- [ ] **Step 9: Task-specific verification**

```bash
test -f LICENSE && grep -q "MIT" LICENSE && grep -q "2026" LICENSE && grep -q "Igor Fonin" LICENSE && echo "LICENSE ok"
test -f README.md && grep -qi "ccstatusline" README.md && grep -q "IGoRFonin/cchud" README.md && echo "README ok"
test -f ATTRIBUTION.md && grep -q "@sirmalloc" ATTRIBUTION.md && grep -qi "ccstatusline" ATTRIBUTION.md && echo "ATTRIBUTION ok"
test -f CHANGELOG.md && grep -q "## \[0.0.1\]" CHANGELOG.md && echo "CHANGELOG ok"
grep -q "247 ms" docs/prd-cchud.md && grep -q "828 ms" docs/prd-cchud.md && echo "PRD R1 ok"
! grep -q "60+ widgets" docs/prd-cchud.md && echo "PRD R2 ok"
grep -q "Фаза 1 — Init (in progress)" plan/README.md && echo "plan README updated"
```

Expected:
```
LICENSE ok
README ok
ATTRIBUTION ok
CHANGELOG ok
PRD R1 ok
PRD R2 ok
plan README updated
```

- [ ] **Step 10: Commit**

```bash
git add LICENSE README.md ATTRIBUTION.md CHANGELOG.md docs/prd-cchud.md plan/README.md
git commit -m "docs(phase-1): README/LICENSE/ATTRIBUTION/CHANGELOG + PRD R1/R2

- LICENSE: MIT, 2026, Igor Fonin
- README.md: WIP status, Phase 0 baseline table, links to PRD/plan/decisions
- ATTRIBUTION.md: explicit credit to @sirmalloc and ccstatusline upstream
- CHANGELOG.md: Keep-a-Changelog format, 0.0.1 entry for skeleton
- docs/prd-cchud.md: R1 (verify §1 baseline reflects actual measurement),
  R2 (fix '60+ widgets' → '60 widgets' in §3)
- plan/README.md: mark Phase 1 as in-progress

Task 4/5 of Phase 1.
"
```

## Verification (стандартный гейт + task-specific)

См. Step 8 (стандартный) и Step 9 (task-specific).

## Definition of Done

- [ ] `LICENSE`, `README.md`, `ATTRIBUTION.md`, `CHANGELOG.md` существуют и валидируются grep'ом
- [ ] PRD §1 содержит измеренный baseline (828/247 мс, 98 МБ)
- [ ] PRD не содержит фразу "60+ widgets"
- [ ] `plan/README.md` отмечает Фазу 1 как `(in progress)`
- [ ] Один коммит с префиксом `docs(phase-1):`

## Files touched

- `LICENSE` (created)
- `README.md` (created)
- `ATTRIBUTION.md` (created)
- `CHANGELOG.md` (created)
- `docs/prd-cchud.md` (modified, possibly no-op for R1 / fix for R2)
- `plan/README.md` (modified — status line)

## Risks & rollback

- **PRD §1 уже обновлён в Phase 0:** Step 5 проверяет; если grep матчит — пропустить редактирование. Не дублировать.
- **Edit tool падает с "old_string not unique":** для R2 строка `Powerline, themes, 60+ widgets` уникальна (проверь grep'ом). Если внезапно не уникальна — увеличить контекст в `old_string`.
- **Markdown linter (если установлен) ругается на CHANGELOG ссылки:** ссылки `[Unreleased]: ...` указывают на ещё-несуществующие сравнения. Это норма для нового проекта; GitHub их рендерит ок.
- **Rollback:** `git checkout HEAD~1 -- docs/prd-cchud.md plan/README.md && rm LICENSE README.md ATTRIBUTION.md CHANGELOG.md`.
