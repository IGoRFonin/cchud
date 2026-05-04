# Migrating from ccstatusline to cchud

## TL;DR

```bash
npx --yes cchud@1.0.0 install   # 1. install (self-relocates to ~/.local/bin/cchud)
cchud import                    # 2. migrate ccstatusline config
```

After migration, your `~/.claude/settings.json` is updated to use cchud.
ccstatusline can be left installed (idle) or uninstalled (`npm uninstall -g ccstatusline`).

## Side-by-side widget map

cchud uses kebab-case widget names. ccstatusline uses PascalCase or camelCase.
`cchud import` performs the rename automatically.

| ccstatusline name           | cchud name                  |
|---|---|
| Model                       | `model`                     |
| Version                     | `version`                   |
| GitBranch                   | `git-branch`                |
| GitChanges                  | `git-changes`               |
| GitStatus                   | `git-status`                |
| GitDiff                     | `git-diff`                  |
| GitTracking                 | `git-tracking`              |
| GitRemote                   | `git-remote`                |
| GitPullRequest              | `git-pr`                    |
| ContextPercentage           | `context-percentage`        |
| ContextPercentageUsable     | `context-percentage-usable` |
| ContextBar                  | `context-bar`               |
| ContextLength               | `context-length`            |
| TokensInput                 | `tokens-input`              |
| TokensOutput                | `tokens-output`             |
| TokensCacheRead             | `tokens-cache-read`         |
| TokensCacheCreated          | `tokens-cache-created`      |
| TokensTotal                 | `tokens-total`              |
| BlockTimer                  | `block-timer`               |
| Cost                        | `cost`                      |
| ResponseTime                | `response-time`             |
| ResponseTimeAvg             | `response-time-avg`         |
| ThinkingTokens              | `thinking-tokens`           |
| OutputStyle                 | `output-style`              |
| Skills                      | `skills`                    |
| Env                         | `env`                       |
| Custom                      | `custom`                    |
| CustomCommand               | `custom-command`            |
| Spacer                      | `spacer`                    |
| Separator                   | `separator`                 |
| ... (60 entries; full list in [docs/widgets.md](./docs/widgets.md)) | |

## Behavioural differences

- **Powerline themes:** identical 5 builtins, byte-compatible. Theme switching через `cchud configure` overlay (`t`).
- **Custom commands:** cchud uses argv-style array (`["my-tool", "--flag"]`) for safety. ccstatusline uses single string. `cchud import` auto-converts.
- **AlignRight widget:** ccstatusline supports it; **cchud does not** (deprecated 0.5.0). Use auto-align widget instead. `cchud import` warns + skips.
- **Settings location:** ccstatusline reads from `~/.claude/settings.json`. cchud uses `~/.config/cchud/settings.json` (XDG-style) and reads only the `statusLine.command` field from `~/.claude/settings.json`.

## Rolling back to ccstatusline

```bash
rm ~/.local/bin/cchud
# manually edit ~/.claude/settings.json: revert statusLine.command to ccstatusline path
# (a backup was saved as ~/.claude/settings.json.bak.<unix-ms> when cchud install ran)
```

`cchud uninstall` is on the Phase 10 backlog.

## Known limitations

- Windows: native binary works; install via `npx --yes cchud@1.0.0 install`. install.sh is macOS/Linux only.
- ARM Linux: not yet supported (Phase 10 backlog). Use `cargo install --git https://github.com/IGoRFonin/cchud` until then.
- Backup of `~/.claude/settings.json` is best-effort (`.bak.<unix-ms>`); we never delete previous backups so they accumulate over time. Periodic cleanup is up to you.
