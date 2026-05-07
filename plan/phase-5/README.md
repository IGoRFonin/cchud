# Фаза 5 — Implementation Plan (Git widgets, 0.3.0)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Релиз `0.3.0` cchud — 20 git-виджетов поверх Phase 3 walking skeleton, p95 < 8 ms на 20-widget config (cache-hit GitPr), один lazy `GitInfo` через `gix 0.81`, `GitPr` с дисковым кэшем `bincode` и offline-fallback. Архитектурный задел Phase 6 (transcript) и Phase 7 (env/http) остаётся нетронутым.

**Architecture:** Линейная цепочка из 9 vertical-slice задач. T1 (setup) лочит `gix` dep, `src/git/mod.rs` lazy `GitInfo`, `RenderContext::git()` через `OnceCell`, фикстура-helper для тестов. T2–T6 — 5 кластерных задач с git-виджетами (head → status → diff → tracking → remote). T7 — `GitPr` (HTTP + auth + bincode-кэш + TTL). T8 — snapshots+hyperfine gate с git-фикстурами. T9 — релиз 0.3.0. Один файл = один кластер виджетов; cross-widget хелперы изолированы в `src/git/`.

**Scope correction:** 5 worktree-виджетов (`Worktree`, `WorktreeMode`, `WorktreeName`, `WorktreeBranch`, `WorktreeOriginalBranch`) уже реализованы в Phase 3 через `payload.worktree`. Phase 5 их не дублирует. Итого **20 git-виджетов**, не 24 как в `phase-5-git-widgets.md`. См. `docs/widgets.md`.

**Tech Stack:** Rust 1.85 (Edition 2024), `gix 0.81` (новая dep, T1, runtime — pure Rust git без libgit2), `ureq 2` (T7, sync HTTP), `bincode 1` (T7, кэш-сериализация), `serde` features `derive` (уже подключён в Phase 2), `tempfile 3` (Phase 3 dev-dep, T1 фикстуры), `mockito 1` (новый dev-dep, T7 GitHub API mock).

---

## DAG задач

```
T1 (setup) ──► T2 (head) ──► T3 (status) ──► T4 (diff) ──► T5 (tracking) ──► T6 (remote)
                                                                                  │
                                                                                  ▼
                                                                              T7 (pr)
                                                                                  │
                                                                                  ▼
                                                                  T8 (snapshots+bench) ──► T9 (release)
```

Линейная цепочка. Между задачами — review checkpoint. Каждая T2–T7 даёт работающий кластер (тесты зелёные, виджеты подключены через `widgets::build_one`).

| # | Файл | Цель | Виджеты | Гейт |
|---|---|---|---|---|
| 1 | [`task-1-setup.md`](./task-1-setup.md) | `gix 0.81` dep; `src/git/mod.rs` (`GitInfo`, `Head`, `GitStatusCounts`, `RemoteInfo`); `RenderContext::git()` через `OnceCell<Option<GitInfo>>`; `src/git/fixture.rs` (test helper для tempfile-репо); 1 DECISIONS-запись (gix vs git2) | 0 | `cargo test git::` зелёные; binary < 6 MB; Phase 3 snapshot'ы зелёные |
| 2 | [`task-2-head.md`](./task-2-head.md) | `widgets/git_head.rs` (`GitBranch`, `GitSha`, `GitRootDir`); 3 enum-варианта; gix HEAD parsing; короткий SHA = 7 символов | 3 | `cargo test widgets::git_head` зелёный |
| 3 | [`task-3-status.md`](./task-3-status.md) | `widgets/git_status.rs` (`GitStatus`, `GitChanges`, `GitStaged`, `GitUnstaged`, `GitUntracked`, `GitConflicts`); один gix status вызов кэшируется в `GitInfo::status_counts: OnceCell` | 6 | `cargo test widgets::git_status` зелёный; status считается ровно 1 раз на 6 виджетов |
| 4 | [`task-4-diff.md`](./task-4-diff.md) | `widgets/git_diff.rs` (`GitInsertions`, `GitDeletions`); `GitInfo::diff_stat: OnceCell` ленивый (вызывается только если виджет есть в строке) | 2 | `cargo test widgets::git_diff` зелёный |
| 5 | [`task-5-tracking.md`](./task-5-tracking.md) | `widgets/git_tracking.rs` (`GitAheadBehind`); читает upstream tracking из gix; формат `↑3↓1` или None если нет upstream | 1 | `cargo test widgets::git_tracking` зелёный |
| 6 | [`task-6-remote.md`](./task-6-remote.md) | `src/git/remote.rs` `parse_url` (hand-parser, 4 формата); `widgets/git_remote.rs` (`GitOriginOwner`, `GitOriginRepo`, `GitOriginOwnerRepo`, `GitUpstreamOwner`, `GitUpstreamRepo`, `GitUpstreamOwnerRepo`, `GitIsFork`) | 7 | `cargo test git::remote` зелёный; `cargo test widgets::git_remote` зелёный |
| 7 | [`task-7-pr.md`](./task-7-pr.md) | `ureq`/`bincode`/`mockito` deps; `src/git/pr.rs` (`PrCache`, `lookup_or_fetch`, `github_token` priority); `widgets/git_pr.rs` (`GitPr`); кэш `~/.cache/cchud/pr-cache.bincode`, TTL 30s, timeout 200ms; offline soft-fail | 1 | `cargo test git::pr` зелёный (mockito); offline-тест без сети ≤ 200 ms |
| 8 | [`task-8-snapshots-bench.md`](./task-8-snapshots-bench.md) | `tests/snapshots_git.rs` ≥5 git-сценариев (clean / dirty / conflicts / fork / detached HEAD); hyperfine на 20-widget config; `benches/phase-5.md`; GitPr cache-hit в bench | 0 | p95 cchud-20w < 8 ms (cache-hit GitPr); insta review committed |
| 9 | [`task-9-release.md`](./task-9-release.md) | README обновление (таблица supported widgets 46/60); `CHANGELOG.md` 0.3.0; `Cargo.toml` version bump; `cargo build --release --locked`; `cchud --version` → `0.3.0`; `git tag v0.3.0`; `plan/README.md` Phase 5 → `[x]`, Phase 6 → `[~]`; `manual-test-log.md` Phase-5-specific шаблон | 0 | Tag запушен; CI matrix зелёный; `cchud --version` показывает `0.3.0` |

**Итого:** 20 виджетов, ~25–35 часов, ~3–5 рабочих дней.

## Pre-flight (требования до старта Task 1)

- **Phase 4 завершена** — Powerline merged, ветка main стабильна, релиз 0.2.0 запушен (`git tag --list 'v0.2.0'`).
- **Текущая директория**: `/Users/igor/mp/startup/cchud`.
- **Рабочее дерево чистое**: `git status` пусто.
- **`rustc` ≥ 1.85** (см. `rust-toolchain.toml`).
- **`hyperfine` ≥ 1.20.0**.
- **`cargo-insta`** установлен (`cargo install cargo-insta`).
- **`gh` CLI** залогинен под `IGoRFonin` для T7 manual-test (`gh auth status`).
- **`git` CLI** установлен (T1 фикстуры используют `git init` через `Command`).
- **`benches/samples/`** содержит 6 Phase-0/3 JSON-payload'ов.
- **Доступ в интернет** для T7 manual-теста против `api.github.com`. CI offline test использует mockito, не требует сети.

## Definition of Done всей Фазы 5

- [ ] 20 git-виджетов реализованы; каждый покрыт unit-тестами (happy + 1+ edge, минимум 2 теста на виджет)
- [ ] Один lazy `GitInfo` (через `OnceCell`) переиспользуется всеми виджетами; gix-вызовы идемпотентны
- [ ] `cargo build --release --locked` зелёный; binary < 7 MB (gix добавит ~1.5 MB к базовым 5 MB)
- [ ] `cargo test --locked` зелёный (≥120 тестов суммарно)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] `cargo insta review` — все snapshot'ы committed без `.snap.new`
- [ ] CI matrix зелёный (macos/ubuntu/windows; T1 фикстуры cross-platform через `git` CLI)
- [ ] Hyperfine p95 < 8 ms на 20-widget config (cache-hit GitPr); `benches/phase-5.md` сохранён в репо
- [ ] `cchud --version` печатает `0.3.0`
- [ ] `GitPr` корректно auth через `gh auth token` и `GITHUB_TOKEN` (T7 manual-test pass)
- [ ] Кэш PR с TTL 30 s, timeout 200 ms, offline не блокирует pipeline
- [ ] Snapshot'ы Phase 5 покрывают 5 git-сценариев (clean / dirty / conflicts / fork / detached HEAD)
- [ ] Manual real-CC test пройден ≥ 5 минут на этом репозитории с активным `GitPr`, лог в `plan/phase-5/manual-test-log.md`
- [ ] `git tag v0.3.0` создан и запушен
- [ ] `plan/README.md` отмечает Phase 5 как `[x]`, Phase 6 как `[~]`
- [ ] `docs/widgets.md`: 20 git-виджетов из TODO → DONE
- [ ] README таблица "supported widgets" с 46/60 галочками
- [ ] `CHANGELOG.md` запись 0.3.0
- [ ] 1+ запись в `docs/DECISIONS.md` (gix vs git2)

## Связи

- **Spec этого плана:** [`../phase-5-git-widgets.md`](../phase-5-git-widgets.md) — оригинальный outline (расходится в scope: 24→20 виджетов после удаления Phase 3 worktree-дублей)
- **PRD:** [`../../docs/prd-cchud.md`](../../docs/prd-cchud.md) — git-виджеты, AC-перформанс
- **Predecessor plan:** [`../phase-3/`](../phase-3/) — конвенции стиля, pre-flight, gate-структура
- **Phase 4** (powerline) — параллельно/завершён, не блокирует Phase 5
- **Widgets registry:** [`../../docs/widgets.md`](../../docs/widgets.md) — canonical список 60 виджетов; Phase 5 = 20 строк `git-*` (worktree-* остаются Phase 3)
- **Future phases:**
  - Phase 6 — JSONL-кэш + 8 transcript-виджетов
  - Phase 7 — env/http-кластер (6 виджетов)
  - Phase 9 — README, CHANGELOG, дистрибуция, паритет 60/60

## Риски и митигации

- **gix breaking changes между minor** — pin к `gix = "=0.81.0"` (точная версия), обновление gix — отдельный PR со smoke-тестом в CI.
- **GitHub API rate limit** для `GitPr` без токена (60 req/h) — кэш TTL 30 s + offline fallback (None — виджет молча скрывается, pipeline не падает); README документирует поведение.
- **Большой монорепо** — `gix status` тяжёлый. Документировать опцию отключить status-виджеты; smoke-тест не на монорепо.
- **`gix::diff` производительность** — diff lazy: считается только если `GitInsertions`/`GitDeletions` есть в строке. Без diff-виджетов — нулевая стоимость.
- **CI Windows** — `gix` работает на Windows, но фикстуры через `git init` требуют git в PATH; CI matrix `windows-latest` включает git по умолчанию.
- **`bincode` schema breakage** — кэш-формат версионируется через wrapper `{ version: u8, entries: ... }`; mismatch → молча reset кэша.
- **`ureq` blocking call в hot-path** — таймаут 200 ms (твёрдый), сетевой запрос только при cache-miss; cache-hit < 1 ms.
- **Worktree double-impl** — если по неосторожности дописать `GitWorktree` через gix, он конфликтует с Phase 3 payload-варианта. Гейт: `WidgetConfig` enum *не* получает новых `Worktree*` вариантов.
