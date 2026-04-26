# cchud

> Fast Rust statusline for Claude Code CLI — drop-in port of [ccstatusline](https://github.com/sirmalloc/ccstatusline) targeting < 5 ms cold-start and < 5 MB RSS.

**Status:** 0.1.0-alpha — 24 of 60 upstream widgets supported (Phase 2 `model` + Phase 3 MVP cluster). Plain renderer; Powerline lands in Phase 4. See [`plan/README.md`](plan/README.md) for roadmap.

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

Alpha release: 24 widgets working end-to-end in Claude Code. Plain renderer (single line, ` | ` separator). Powerline visual parity is Phase 4; full `ccstatusline` widget set lands across Phases 5–7.

## Supported widgets (24 / 60)

| Source | Widgets |
|---|---|
| Payload (fast) | `model`, `version`, `claude-session-id`, `terminal-width`, `output-style`, `vim-mode`, `session-name`, `session-clock`, `session-cost`, `context-length`, `context-percentage`, `context-percentage-usable`, `context-bar`, `tokens-input`, `tokens-output`, `worktree`, `worktree-mode`, `worktree-name`, `worktree-branch`, `worktree-original-branch` |
| Static (config) | `custom-text`, `custom-symbol`, `link` |
| Subprocess | `custom-command` (argv-style, configurable timeout, default 200ms) |

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

### CustomCommand security

`custom-command` spawns the configured binary argv-style (no shell). It **inherits the parent process env**, so any `API_KEY` / secret in your shell is visible to the subprocess. Phase 7 will add opt-in env-allowlist + sandboxing.

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
