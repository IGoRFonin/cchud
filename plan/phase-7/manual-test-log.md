# Phase 7 Manual Test Log — 2026-04-29

## Configs tested

- `usage-cluster.json` — 4 timers showing correctly; `resets_at` delta verified; session-usage and weekly-usage render token counts from HTTP API.
- `multi-line-themed.json` — `\n` separator, `continue_theme_across_lines: true` preserves powerline theme and separator state across lines.
- `theme-globals.json` — `override_background_color: "#aabbcc"` applied uniformly to all widgets in powerline mode.
- `per-widget-override.json` — `model` with `bold: true`; `session-cost` with custom `color` override rendering correctly.

## Observations

- All 7 new Phase 7 widgets render without panic on real CC payloads.
- `claude-account-email` reads `~/.claude.json` correctly; graceful fallback to empty string when file absent.
- `free-memory` shows system free memory via sysinfo; macOS sysctl path works.
- `skills` counts unique skill names from transcript JSONL; sorted output stable.
- `session-usage` / `weekly-usage` show `--` when HTTP call fails (offline mode); no crash.
- `block-reset-timer` / `weekly-reset-timer` show countdown correctly; zero-case shows `00:00`.
- Multi-line: `\n` between lines renders cleanly in Ghostty; cursor position correct.
- `continue_theme_across_lines: true` — powerline segments continue without reset glyph between lines.
- `auto_align` with `AlignRight` sentinel pads correctly to terminal width.
- Per-widget `bold: true` on model: bold attribute applied, no bleed to adjacent widgets.
- `override_background_color` in theme-globals: all segments share the background; separator fg adapts.
- `minimalist_mode: true` hides empty widgets without gaps.

## Issues found

None. All 4 test configs passed without visible rendering artifacts.
