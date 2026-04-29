# cchud

> Fast Rust statusline for Claude Code CLI — drop-in port of [ccstatusline](https://github.com/sirmalloc/ccstatusline) targeting < 5 ms cold-start and < 5 MB RSS.

**Status:** 0.4.0 — 54 of 60 upstream widgets supported. Plain and Powerline renderers, 20 git-widgets via gix, 8 transcript-widgets via JSONL cache. See [`plan/README.md`](plan/README.md) for roadmap.

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

54 widgets working end-to-end in Claude Code. Plain renderer (` | ` separator) and Powerline renderer (segmented, 5 built-in themes). 20 git-widgets via `gix 0.81` (pure Rust, no libgit2). 8 transcript-widgets via incremental JSONL cache (`sonic-rs` hot path). Remaining 6 widgets land in Phase 7.

## Supported widgets (54 / 60)

| Source | Widgets |
|---|---|
| Payload (fast) | `model`, `version`, `claude-session-id`, `terminal-width`, `output-style`, `vim-mode`, `session-name`, `session-clock`, `session-cost`, `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`, `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch` |
| Static (config) | `custom-text`, `custom-symbol`, `link` |
| Subprocess | `custom-command` (argv-style, configurable timeout, default 200ms) |
| Git (gix) | `git-branch`, `git-sha`, `git-root-dir`, `git-status`, `git-changes`, `git-staged`, `git-unstaged`, `git-untracked`, `git-conflicts`, `git-insertions`, `git-deletions`, `git-ahead-behind`, `git-origin-owner`, `git-origin-repo`, `git-origin-owner-repo`, `git-upstream-owner`, `git-upstream-repo`, `git-upstream-owner-repo`, `git-is-fork` |
| HTTP | `git-pr` (GitHub API, disk-cached, offline-tolerant) |
| Transcript (JSONL cache) | `tokens-cached`, `tokens-total`, `input-speed`, `output-speed`, `total-speed`, `block-timer`, `session-duration`, `thinking-effort` |

See [`docs/widgets.md`](docs/widgets.md) for the full 60-widget roadmap.

## Configure

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

MIT — see [`LICENSE`](LICENSE). Original `ccstatusline` is also MIT; see [`ATTRIBUTION.md`](ATTRIBUTION.md) for credits.
