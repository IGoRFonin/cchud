# Phase 8 Manual Test Log — 2026-05-02

## Sessions
- 2026-05-02 · automated CI validation · macOS Darwin 24.6.0

## Configs tested
- default (Settings::default)
- import-full-cc.json → ccstatusline section, 2 widgets (model + git-branch)
- import-with-section.json → 2 widgets (session-cost + context-percentage)
- import-unknown-widgets.json → 1 known widget, 2 skipped with warning
- import-empty-section.json → all widgets unknown → exit 1 "nothing imported"

## Automated coverage
- ≥6 ratatui TestBackend snapshot tests (initial_state, palette_filtered_by_git,
  settings_with_color_picker_open, themes_overlay_open, help_overlay_open,
  confirm_quit_modal_when_dirty)
- ≥7 import integration tests via assert_cmd
- configure-no-TTY smoke: exit 2 + "must be run in a terminal"
- Cold-start bench: mean 3.5 ms / p95 5.0 ms (NFR < 50 ms ✓)

## Observations
- Cold-start well within 50 ms NFR (3.5 ms mean / 5.0 ms p95).
- All 993 tests pass across all test targets.
- Snapshot baselines accepted via INSTA_UPDATE=always.

## Issues found
- None
