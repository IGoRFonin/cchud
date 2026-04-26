---
title: "PRD: cchud — Rust statusline for Claude Code"
author: igorfonin
date: 2026-04-26
status: Draft
version: 1.1
license: MIT
package_name: cchud
upstream_parity_target: ccstatusline 2.2.8
---

# PRD: cchud

**Author:** igorfonin
**Date:** 2026-04-26
**Status:** Draft
**Version:** 1.1 (Phase 0 update: 2026-04-26)
**License:** MIT

## 1. Problem Statement

Claude Code CLI invokes the statusline command up to 3 times per second (once every 300 ms). The dominant utility `ccstatusline` (8.3k⭐, npm) introduces two problems:

1. **Resource usage.** The default configuration `npx -y ccstatusline@latest` consumes **828 ms CPU** and ~98 MB RAM on every render (Phase 0 measurement: Apple M4 Pro, node v24.13.0, ccstatusline 2.2.8, 200 iterations hyperfine; global install: 247 ms). On M-chip machines the fan is audible during long sessions; on battery the power budget impact is noticeable. With 4 parallel Claude Code sessions — up to 400% of a single core in the background (≈ 3 calls/sec × 247 ms × 4 sessions).

2. **Supply-chain.** `@latest` effectively checks for new npm versions every 300 ms. Any compromise of the upstream author's npm account or transitive deps reaches execution within 300 ms with access to `~/.ssh`, `~/.aws/credentials`, `~/.claude.json` (Anthropic API key).

Issue [anthropics/claude-code#10162](https://github.com/anthropics/claude-code/issues/10162) requesting a persistent statusline daemon **was closed NOT_PLANNED on 2026-01-13**. Cold-start remains a fundamental constraint and must be addressed at the utility level itself.

## 2. Goals & Success Metrics

| Goal | Metric | Target |
|---|---|---|
| Reduce cold-start CPU (M-series) | hyperfine p95 | **< 5 ms** |
| Reduce cold-start CPU (Linux x86_64) | hyperfine p95 | **< 8 ms** |
| Reduce cold-start CPU (Windows) | hyperfine p95 | **< 12 ms** |
| Reduce RAM | peak RSS / process | **< 5 MB** |
| Reduce binary size | release-stripped size | **< 5 MB** |
| Remove background load from 4 parallel sessions | sustained CPU | **< 5%** of a single core |
| Eliminate supply-chain `@latest` | runtime npm resolves per render | **0** |
| Achieve feature parity | ccstatusline 2.2.8 widget coverage | **100%** (60) |

## 3. Target Users

**Primary:** Power-user of Claude Code on M-series Mac, keeps 2–4 split-panel sessions open, notices ccstatusline's impact on battery/fan.

**Secondary:**
- Linux/Windows Claude Code users
- Supply-chain paranoids (require pinned versions and signatures)
- Existing ccstatusline users who value features (Powerline, themes, 60 widgets) but don't want to pay the resource cost

**Non-targets:**
- Users of bash scripts like `claude-lens` — their use case is already covered by minimalism
- Those who run a single session infrequently — savings are negligible

## 4. User Stories

- **US-1:** As a power-user, I want to install cchud with a single command (`brew`/`npm`/`curl`) so I don't have to build from source.
- **US-2:** As an existing ccstatusline user, I want to import my old config so I don't have to reconfigure everything from scratch.
- **US-3:** As a Powerline-theme user, I want to see the same visual output as ccstatusline.
- **US-4:** As a TUI-configurator user, I want to compose the status line via `cchud configure` without editing JSON by hand.
- **US-5:** As a paranoid user, I want signed releases with SHA256 checksums and the ability to install an exact version.
- **US-6:** As a Windows user, I want colors and Powerline to work correctly in Windows Terminal.
- **US-7:** As a split-panel user, I want 4 simultaneous sessions to not heat up my MacBook.

## 5. Functional Requirements

### P0 — Must Have (1.0)

- **REQ-001:** The binary shall read Claude Code's JSON payload from stdin and write the status line to stdout.
- **REQ-002:** The binary shall read `~/.config/cchud/settings.json` and apply the config (analogous to upstream ccstatusline, which stores config in `~/.config/ccstatusline/settings.json`, not in `~/.claude/settings.json`).
- **REQ-003:** 100% widget parity with ccstatusline 2.2.8 (59 file-based + 1 built-in `separator` = 60).
- **REQ-004:** Powerline rendering with themes shall be visually identical to ccstatusline.
- **REQ-005:** Support cchud config migrations from older versions via a `migrate_vN_to_vN+1` chain.
- **REQ-006:** Import config from ccstatusline (TS format) via `cchud import` or autodetect.
- **REQ-007:** Subcommand `cchud configure` launches a TUI configurator with live preview.
- **REQ-008:** Pre-built binaries on GitHub Releases for: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`.
- **REQ-009:** SHA256 checksums for every release artifact.
- **REQ-010:** npm package `cchud` (bootstrap-style: postinstall downloads the binary from Releases by platform/arch).
- **REQ-011:** Homebrew formula via tap `igorfonin/homebrew-tap`.
- **REQ-012:** Install script `curl -fsSL .../install.sh | sh`, detects platform and downloads binary.
- **REQ-013:** MIT license, correct attribution to upstream ccstatusline.
- **REQ-014:** GitHub Actions CI: build, test, clippy, fmt on 3 platforms (macOS, Linux, Windows).
- **REQ-015:** `cchud install` shall write the command to `~/.claude/settings.json` without overwriting other keys.
- **REQ-016:** `cchud --version` shall print semver, git-sha, and target-triple.

### P1 — Should Have

- **REQ-100:** The `GitPr` widget should cache GitHub API responses to disk with a 30-second TTL.
- **REQ-101:** JSONL transcript cache with mtime+size invalidation; on-disk bincode format.
- **REQ-102:** `CustomCommand` executes with a 200 ms timeout by default (protection against hangs).
- **REQ-103:** Hyperlinks (OSC 8) with graceful fallback in terminals that don't support them.
- **REQ-104:** Auto-detect color level (truecolor / 256 / basic / none) via `supports-color`.
- **REQ-105:** Bash/Zsh/Fish completions via `clap_complete` (for the configuration subcommand).
- **REQ-106:** `cchud doctor` — diagnostics: Powerline font, color, paths, version, permissions.

### P2 — Nice to Have (for 1.x)

- **REQ-200:** TOML config as an optional alternative to JSON.
- **REQ-201:** Schemars-based JSON schema generation for editors.
- **REQ-202:** Auto-update via `cchud update`.
- **REQ-203:** Prometheus metrics about own performance.
- **REQ-204:** Back-port improvements to ccstatusline upstream.

## 6. Non-Functional Requirements

### Performance
- Cold-start p95: **< 5 ms** (M-series), **< 8 ms** (Linux x86_64), **< 12 ms** (Windows).
- Peak RSS: **< 5 MB**.
- 4 parallel sessions: **< 5%** of a single core sustained.
- JSONL parsing: transcript up to 50 MB — **< 10 ms** with cold cache, **< 2 ms** with warm cache.
- Release binary size: **< 5 MB** (stripped, lto, single codegen unit, panic=abort).

### Security
- 0 runtime npm/registry resolves per render.
- Releases with SHA256 checksums; optionally `cosign`/sigstore.
- HTTP calls (Anthropic Usage, GitHub PR) only in optional widgets, disableable in config.
- No telemetry / analytics / phone-home.
- `--frozen` lockfile on CI; `cargo audit` in pre-release.

### Compatibility
- macOS 12+, Linux glibc 2.31+ (Ubuntu 20.04+), Linux musl (Alpine), Windows 10 1809+.
- Rust MSRV 1.85 (edition 2024).
- Claude Code statusline contract: payload via stdin, statusline to stdout (first line), exit code 0.

### Maintainability
- All widgets under the `Widget` trait with unit tests.
- Snapshot tests on ≥ 10 real payloads via `insta`.
- Documentation in `docs/widgets.md` with a table `ccstatusline-name → cchud-name → owner`.
- `cargo doc` published to docs.rs.

## 7. Acceptance Criteria

- **AC-001:** Given empty config + default payload, when running cchud, then output is `Model | Context X% | Branch Y`.
- **AC-002:** Given an existing ccstatusline config, when running `cchud import`, then config converts without loss and the same payload renders visually identically.
- **AC-003:** Given hyperfine 200 iterations cold-start on M2, then p95 < 5 ms.
- **AC-004:** Given 4 parallel Claude Code sessions in `top`, then total CPU < 5% of a single core sustained.
- **AC-005:** Given release 1.0.0, then 5 binaries are available, npm `cchud@1.0.0` installs on macOS/Linux/Windows, `brew install igorfonin/tap/cchud` installs.
- **AC-006:** Given snapshot tests on ≥ 10 different payloads, then all outputs are stable between runs.
- **AC-007:** Given broken JSON in settings, then cchud prints the default line and does not crash Claude Code (exit 0).
- **AC-008:** Given an offline machine, then no blocking network calls in the hot path; HTTP widgets timeout in < 200 ms with graceful fallback.
- **AC-009:** Given Windows Terminal, then Powerline glyphs and colors render correctly.
- **AC-010:** Given `cchud configure`, then you can compose a status line from scratch, save it, restart Claude Code, and see the result.
- **AC-011:** Given the binary after strip+lto, then size < 5 MB on all 5 target triples.
- **AC-012:** Given `cchud doctor`, then it checks fonts/color/paths and produces an actionable report.

## 8. Design Considerations

- **TUI configurator** built with ratatui+crossterm. Mirrors the ccstatusline TUI structure: Lines panel, Widget palette, Settings panel, Live preview.
- **CLI**: `cchud` (default render), `cchud configure`, `cchud import [--from <path>]`, `cchud install`, `cchud doctor`, `cchud --version`.
- **Powerline glyphs** — same Unicode code points as ccstatusline (visual compatibility).
- **Color themes** — port of `powerline-theme-index.ts` to `themes.toml` (or hardcoded with override support).
- **Health checks**: `cchud doctor` — checks Powerline font (Nerd Font), color level, paths, version, settings.json permissions.

## 9. Technical Considerations

### Stack (locked in `Cargo.toml`)
- CLI args: `lexopt` (37 KiB overhead) for the hot path; `clap` optionally for configuration subcommands.
- JSON: `serde_json` for config and payload, `sonic-rs` for transcript JSONL.
- HTTP: `ureq` (no tokio), TLS via `rustls`.
- Git: `gix` (pure Rust, 2–10× faster than libgit2) with feature-set `max-performance-safe + status + revision + zlib-rs + sha1`.
- Terminal: `anstyle` + `nu-ansi-term` + `crossterm` + `unicode-width` + `unicode-segmentation` + `terminal_size` + `supports-color`.
- TUI: `ratatui` + `crossterm`. Optionally `ratatui-interact` if a focus/click manager is needed.
- Cache: `bincode 2.x` for on-disk JSONL cache.

### Build profile
```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"
opt-level = 3
```

### Architecture
- `RenderContext` — single contextual state: payload, settings, lazy git, lazy transcript, lazy http-cache.
- `Widget` trait — each widget in `src/widgets/<name>.rs`.
- `Renderer` — Plain / Powerline.
- Lazy git: `OnceCell<GitInfo>` opened on first git-widget access.
- Lazy transcript: `OnceCell<TranscriptCache>` parsed on first transcript-widget access.

### Distribution
- GitHub Releases — single source of truth.
- npm `cchud`: tarball ~10 KB, postinstall.js downloads the appropriate binary by `process.platform/arch` and `process.env.npm_config_cchud_version`.
- Homebrew tap: formula downloads binary and verifies SHA256.
- Install script: `curl -fsSL https://raw.githubusercontent.com/igorfonin/cchud/main/install.sh | sh` detects platform and downloads binary.

### Cross-build
- `cargo-zigbuild` (proven in astral-sh/uv, ruff) or GitHub Actions matrix runners.
- Targets: 5 platforms.
- Smoke test after build: run binary with `--version` and a test payload.

## 10. Out of Scope

- Persistent daemon mode (rejected by Anthropic, issue #10162 NOT_PLANNED).
- Replacing the Claude Code statusline mechanism itself.
- Bun runtime support (we are a Rust binary, no JS runtime needed).
- Widgets for other CLI agents (Cursor, Cline, aider) — separate project if needed.
- GUI configurator (TUI only for 1.0).
- Auto-import from arbitrary JSON formats other than ccstatusline.
- Mobile Claude Code clients (if they appear in the future).
- Telemetry / usage analytics.
- Support for terminals without ANSI (cmd.exe older than Windows 10 1809).

## 11. Open Questions

- **OQ-1:** ~~Exact JSON format of the cchud section in `~/.claude/settings.json` — shared with ccstatusline (for seamless migration) or its own section?~~ **CLOSED (Phase 0):** cchud uses `~/.config/cchud/settings.json` — a separate file, analogous to upstream. Details in `docs/DECISIONS.md`.
- **OQ-2:** Conflict strategy when `npm i -g ccstatusline && npm i -g cchud` (simultaneous install). Resolved in **Phase 9**.
- **OQ-3:** Release signing — `cosign` (sigstore) or GPG? Resolved in **Phase 9**.
- **OQ-4:** Support `gh auth token` as fallback for `GitPr` widget — needed, or restrict to `GITHUB_TOKEN` env?
- **OQ-5:** Add feature-flag `tui` (optional build without TUI) so CI can build a minimal binary for the most performance-sensitive users?
- **OQ-6:** ~~Is the name `cchud` available on crates.io / npm?~~ **CLOSED (Phase 0):** Available on crates.io, npm, GitHub `IGoRFonin/cchud` and `IGoRFonin/homebrew-tap`. No fallback needed.

## 12. Dependencies & Risks

| Risk | Impact | Mitigation |
|---|---|---|
| `sonic-rs` API changes (young crate) | medium | Pin minor version, abstract in `utils/jsonl.rs`, fallback to `serde_json` behind feature flag |
| `gix` API changes in 0.x | medium | Track CHANGELOG, pin minor, smoke-test git-widgets in CI |
| `ratatui-interact` loses support | low | Standard ratatui without `-interact` is always stable; refactor in 1–2 days |
| Anthropic changes Claude Code JSON payload | high | Snapshot tests on ≥ 10 payloads; widget reads only required fields, ignores the rest |
| ccstatusline changes settings format → import breaks | medium | Config versioning, migration chain, explicit `--from-version` flag in `cchud import` |
| Cross-build issues on Windows | medium | `cargo-zigbuild` or GHA Windows runners, smoke test on every release |
| Name `cchud` already taken | low | Checked in Phase 0, fallback: `cchud-cli`, `claude-hud`, `ccline` |
| Low adoption: users reluctant to switch | medium | `cchud import` in one command, README with cold-start numbers vs ccstatusline |
| Performance regression when adding widgets | high | Bench in CI on every PR; budget < 5 ms is fixed, regression > 10% blocks merge |
| TUI configurator delays 1.0 | medium | Feature-freeze TUI in phase 8 (1–2 weeks hard cap); if not ready — release 0.9 without TUI and follow up in 1.0 hotfix |
