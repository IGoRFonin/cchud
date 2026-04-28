# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] — 2026-04-28

### Added (Phase 5 — Git widgets)

20 git-виджетов через `gix 0.81` (pure Rust, без libgit2):

**Head (3):** `git-branch`, `git-sha` (7-char short), `git-root-dir`

**Status (6):** `git-status` (summary `M1 ~2 ?3 ✗4`), `git-changes`, `git-staged`, `git-unstaged`, `git-untracked`, `git-conflicts`

**Diff stat (2):** `git-insertions` (`+N`), `git-deletions` (`-N`) — lazy, считаются только при наличии в строке.

**Tracking (1):** `git-ahead-behind` (`↑3↓1`)

**Remote (7):** `git-origin-{owner,repo,owner-repo}`, `git-upstream-{owner,repo,owner-repo}`, `git-is-fork`. Hand-parser URL без regex (4 формата: SSH short/explicit, HTTPS с/без `.git`).

**HTTP (1):** `git-pr` через GitHub API. Auth priority: `GITHUB_TOKEN` → `gh auth token` → анонимно. Дисковый кэш `~/.cache/cchud/pr-cache.bincode` с TTL 30 s, hard timeout 200 ms, offline soft-fail.

### Performance

- p95 < 8 ms на 21-widget config (включая `git-pr` cache-hit) — см. `benches/phase-5.md`
- `git-pr` cache-hit overhead: ~0.4 ms vs baseline без HTTP виджета
- Один `gix::status` вызов на 6 status-виджетов через `OnceCell`

### Internals

- `src/git/mod.rs`: `GitInfo` lazy через `OnceCell` для `status_counts`/`diff_stat`/`tracking`
- `RenderContext::git()` — единственная точка входа, `discover()` максимум один раз за render
- `src/git/remote.rs`: hand-parser, экономит ~300 KB бинаря vs `regex` dep
- `src/git/pr.rs`: schema-versioned `PrCache` (mismatch → silent reset)

### Dependencies

- `gix = "=0.81.0"` (pinned exact, runtime)
- `ureq = "2"` with `rustls-tls` (runtime — для GitPr)
- `bincode = "1"` (runtime — кэш)
- `mockito = "1"` (dev-dep, GitHub API mock)

### Decisions

См. `docs/DECISIONS.md` D-2026-04-27 — gix vs git2.

### Scope notes

5 worktree-виджетов (`worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch`) уже были реализованы в Phase 3 через `payload.worktree`. Phase 5 их не дублирует. Total widget coverage: 46/60.

## [0.2.0] — 2026-04-27

- Powerline-рендеринг (`theme.kind: powerline`) с 5 встроенными темами
  (default, dracula, solarized-dark, nord, gruvbox-dark).
- OSC 8 hyperlinks для `link`-виджета — runtime detect.
- Цветовой downgrade Rgb → Ansi256 → None по уровню терминала.
- ANSI strip + visible width helper (`util::ansi`).
- Trait `Widget::default_style` — upstream-паритет цветов для Model/Worktree*/SessionCost/ContextBar/VimMode/OutputStyle.
- Schema `theme.{kind,theme_name,custom,separators,start_caps,end_caps,color_level}`.
- Удалены unused deps `anstream`, `nu-ansi-term`. Добавлены `anstyle-parse`, `supports-hyperlinks`.

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

[Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/IGoRFonin/cchud/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/IGoRFonin/cchud/compare/v0.1.0-alpha...v0.2.0
[0.1.0-alpha]: https://github.com/IGoRFonin/cchud/releases/tag/v0.1.0-alpha
[0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
