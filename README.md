# cchud

> Fast Rust statusline for Claude Code CLI — drop-in port of [ccstatusline](https://github.com/sirmalloc/ccstatusline) targeting < 5 ms cold-start and < 5 MB RSS.

**Status:** 1.0.0 — 60 of 60 upstream widgets supported. Interactive TUI configurator (`cchud configure`), ccstatusline import (`cchud import`). Plain and Powerline renderers, 20 git-widgets via gix, 8 transcript-widgets via JSONL cache, 7 env/http widgets. See [`plan/README.md`](plan/README.md) for roadmap.

## What

`cchud` aims to be a feature-parity Rust port of `ccstatusline`. It reads the JSON payload Claude Code pipes to `statusLine.command` on every prompt, then renders a one-line statusbar — model name, git state, token usage, cost-per-block, etc.

## Why

`ccstatusline` is excellent but Node.js cold-start dominates per-prompt latency. Phase 0 measurement on Apple M4 Pro (node v24.13.0, ccstatusline 2.2.8, 200 iterations hyperfine):

| Mode | Mean [ms] | RSS peak |
|---|---:|---:|
| `npx -y ccstatusline@latest` | 827.7 | — |
| `ccstatusline` (global install) | 246.7 | 97.7 MB |

Claude Code invokes the statusline up to 3 times per second. For 4 parallel sessions that is ≈ 3 calls/s × 247 ms × 4 ≈ 296% of a single core. `cchud` targets ≤ 5 ms cold-start and ≤ 5 MB RSS — a ~50× / ~20× reduction respectively.

## Install

One command — wires Claude Code and opens the TUI configurator:

```bash
npx -y @cchud/cchud
```

Without Node (macOS / Linux gnu/musl):

```bash
curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | sh
cchud
```

Both methods install the binary at `~/.local/bin/cchud` (or
`%LOCALAPPDATA%\cchud\cchud.exe` on Windows). The path is stable across
`nvm use` and Node-version switches.

Other entry-points:

```bash
cchud doctor       # 9-check health report
cchud import       # migrate from ccstatusline (if applicable)
cchud install      # re-wire (idempotent; --force to overwrite a non-cchud statusLine)
```

## Verify install (security-conscious users)

Each npm package is published with [npm provenance](https://docs.npmjs.com/generating-provenance-statements):

```bash
npm view @cchud/cchud@1.0.0 --json | jq .dist.attestations
```

GitHub Releases tarballs include `.sha256` checksums alongside each tarball:

```bash
curl -sSL https://github.com/IGoRFonin/cchud/releases/download/v1.0.0/cchud-darwin-arm64.tar.gz.sha256
```

## Status

60 widgets working end-to-end in Claude Code — full parity with ccstatusline 2.2.8. Plain renderer (` | ` separator) and Powerline renderer (segmented, 5 built-in themes). 20 git-widgets via `gix 0.81` (pure Rust, no libgit2). 8 transcript-widgets via incremental JSONL cache (`sonic-rs` hot path). Per-widget style overrides, 9 global theme settings, multi-line render.

## Supported widgets (60 / 60)

| Source | Widgets |
|---|---|
| Payload (fast) | `model`, `version`, `claude-session-id`, `terminal-width`, `output-style`, `vim-mode`, `session-name`, `session-clock`, `session-cost`, `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`, `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch` |
| Static (config) | `custom-text`, `custom-symbol`, `link` |
| Subprocess | `custom-command` (argv-style, configurable timeout, default 200ms) |
| Git (gix) | `git-branch`, `git-sha`, `git-root-dir`, `git-status`, `git-changes`, `git-staged`, `git-unstaged`, `git-untracked`, `git-conflicts`, `git-insertions`, `git-deletions`, `git-ahead-behind`, `git-origin-owner`, `git-origin-repo`, `git-origin-owner-repo`, `git-upstream-owner`, `git-upstream-repo`, `git-upstream-owner-repo`, `git-is-fork` |
| HTTP | `git-pr` (GitHub API, disk-cached, offline-tolerant), `session-usage`, `weekly-usage`, `block-reset-timer`, `weekly-reset-timer` |
| Transcript (JSONL cache) | `tokens-cached`, `tokens-total`, `input-speed`, `output-speed`, `total-speed`, `block-timer`, `session-duration`, `thinking-effort`, `skills` |
| Env / system | `claude-account-email`, `free-memory` |

See [`docs/widgets.md`](docs/widgets.md) for the full widget reference.

## Migrating from ccstatusline

See [MIGRATION.md](./MIGRATION.md) for a step-by-step guide. Quick path:

```bash
npx -y @cchud/cchud
cchud import      # auto-detects ccstatusline section in ~/.claude/settings.json
```

## TUI Configurator

Run `cchud configure` to open the interactive 4-panel TUI (requires a real terminal):

```bash
cchud configure
```

**Panels:**
- **Lines** — add/remove/reorder lines and widgets (Arrow keys, `a` to add, `d` to delete, `Alt+↑↓` to reorder).
- **Palette** — 60-widget palette with live filter (`/`), 11 categories.
- **Settings** — per-widget overrides: color picker (16 ANSI + custom hex), background_color, bold.
- **Preview** — live statusbar preview using sample data.

**Keybindings:** `Tab`/`Shift+Tab` — move between panels · `t` — themes overlay · `?` — help · `Ctrl+S` — save · `q` — quit (confirm if dirty).

### Migrate from ccstatusline

```bash
# Auto-detect ccstatusline section in ~/.claude/settings.json
cchud import

# From a specific file, then open TUI
cchud import --from /path/to/settings.json --then-configure

# Force overwrite existing config (creates backup)
cchud import --from /path/to/settings.json --force
```

## Configure (manual)

`cchud install` wires cchud into `~/.claude/settings.json` as `statusLine.command`. Widget list lives in a **separate** file: `~/.config/cchud/settings.json` (auto-created with defaults on first run):

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model"},
      {"type": "session-cost"},
      {"type": "context-percentage"},
      {"type": "context-bar", "width": 10},
      {"type": "worktree-name"},
      {"type": "vim-mode"}
    ]
  }],
  "theme": {}
}
```

### Git widgets quickstart

```json
{
  "lines": [{
    "widgets": [
      { "type": "model" },
      { "type": "separator" },
      { "type": "git-branch" },
      { "type": "git-status" },
      { "type": "separator" },
      { "type": "context-percentage" },
      { "type": "session-cost" }
    ]
  }]
}
```

`git-pr` requires a GitHub remote and uses `GITHUB_TOKEN` or `gh auth token` for auth. Results are disk-cached at `~/.cache/cchud/pr-cache.bincode` (TTL 30 s). Missing token → anonymous (60 req/h limit).

### Powerline

Set `theme.kind` to `powerline` to enable the segmented Powerline style. A Nerd Font (or compatible) must be installed in your terminal for the separator glyphs to render correctly.

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model"},
      {"type": "session-cost"},
      {"type": "context-bar", "width": 10}
    ]
  }],
  "theme": {
    "kind": "powerline",
    "theme_name": "default"
  }
}
```

Built-in themes: `default`, `dracula`, `solarized-dark`, `nord`, `gruvbox-dark`.

> **Note:** Powerline separators require a [Nerd Font](https://www.nerdfonts.com/) or a terminal that ships its own powerline glyphs (e.g. Ghostty, Warp). Without one, you will see `?` boxes instead of arrows.

### Transcript widgets quickstart

```json
{
  "version": 1,
  "lines": [{
    "widgets": [
      {"type": "model"},
      {"type": "git-branch"},
      {"type": "tokens-total"},
      {"type": "block-timer"},
      {"type": "thinking-effort"}
    ]
  }]
}
```

Transcript widgets read the Claude Code JSONL transcript file and cache parsed stats in `~/.cache/cchud/transcript-<hash>.bincode` (incremental tail-merge, < 2 ms warm hit). No configuration required — cchud auto-discovers the transcript path from the payload.

### CustomCommand security

`custom-command` spawns the configured binary argv-style (no shell). It **inherits the parent process env**, so any `API_KEY` / secret in your shell is visible to the subprocess. Phase 7 will add opt-in env-allowlist + sandboxing.

## Performance

Hyperfine p95 on Apple M4 Pro (cold-start, release binary):

| Config | p95 |
|---|---:|
| 22-widget plain (Phase 3) | < 5 ms |
| 21-widget git+PR cache-hit (Phase 5) | < 8 ms |
| 8-widget transcript cold-parse 50 MB (Phase 6) | ~70 ms |
| 8-widget config (Phase 7) | < 5 ms |
| 60-widget full config (Phase 7) | < 12 ms |

## Dependencies

Runtime: `gix = "=0.81.0"` (pure Rust git, no libgit2), `ureq 2` + `rustls-tls` (git-pr HTTP), `bincode 1` (PR disk cache), `sonic-rs 0.5` (JSONL transcript parse), `siphasher 1` (cache-key), `time 0.3` (ISO-8601 → ms).

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

MIT. See [LICENSE](./LICENSE) for the full text. See [ATTRIBUTION.md](./ATTRIBUTION.md) for credits to third-party projects (ccstatusline, ratatui, crossterm, и т.д.).
