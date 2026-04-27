# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/IGoRFonin/cchud/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/IGoRFonin/cchud/compare/v0.1.0-alpha...v0.2.0
[0.1.0-alpha]: https://github.com/IGoRFonin/cchud/releases/tag/v0.1.0-alpha
[0.0.1]: https://github.com/IGoRFonin/cchud/releases/tag/v0.0.1
