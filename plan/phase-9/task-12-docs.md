# Task 12 — README + MIGRATION + CHANGELOG + ATTRIBUTION

**Цель:** Documentation pass перед `v1.0.0-rc.1` тегом. README incremental update (Install + Verify + Migration + footer). Новый `MIGRATION.md` (~80-120 LOC). `CHANGELOG.md` 1.0.0 entry. Опциональный `ATTRIBUTION.md`.

**Files:**
- Modify: `README.md` — добавить sections Install, Verify, Migration, footer.
- Create: `MIGRATION.md` (repo root)
- Modify: `CHANGELOG.md` — добавить 1.0.0 entry перед существующими entries.
- Create: `ATTRIBUTION.md` (optional; если решено не делать — убрать ссылку в README footer)

---

- [ ] **Step 1: Прочитать current README**

```bash
rtk read README.md
```

Найти существующие секции (hero, comparison, widgets, TUI, Why). Identify оптимальные insertion points:
- "Install" — после Why / перед Quick start.
- "Verify" — сразу после Install.
- "Migrating from ccstatusline" — около widgets-таблицы.
- License footer — в самом низу.

- [ ] **Step 2: Добавить Install section в README**

После секции "Why cchud?" (или перед "Quick start" если такая существует) вставить:

```markdown
## Install

Pick one:

```bash
# Recommended (one-liner via Node):
npx --yes cchud@1.0.0 install

# Without Node (macOS / Linux gnu/musl):
curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | sh
```

Both methods install the binary at `~/.local/bin/cchud` (or
`%LOCALAPPDATA%\cchud\cchud.exe` on Windows) and wire Claude Code via
`cchud install`. The path is stable across `nvm use` and Node-version
switches.

After install:

```bash
cchud doctor       # 9-check health report
cchud configure    # interactive TUI configurator
cchud import       # migrate from ccstatusline (if applicable)
```
```

- [ ] **Step 3: Добавить Verify section**

Сразу после Install:

```markdown
## Verify install (security-conscious users)

Each npm package is published with [npm provenance](https://docs.npmjs.com/generating-provenance-statements):

```bash
npm view cchud@1.0.0 --json | jq .dist.attestations
```

GitHub Releases tarballs include `.sha256` checksums alongside each tarball:

```bash
curl -sSL https://github.com/IGoRFonin/cchud/releases/download/v1.0.0/cchud-darwin-arm64.tar.gz.sha256
```
```

- [ ] **Step 4: Добавить Migration ссылку**

Около secции Widgets (или после неё):

```markdown
## Migrating from ccstatusline

See [MIGRATION.md](./MIGRATION.md) for a step-by-step guide. Quick path:

```bash
npx --yes cchud@1.0.0 install
cchud import      # auto-detects ccstatusline section in ~/.claude/settings.json
```
```

- [ ] **Step 5: Footer License**

В самом низу README:

```markdown
## License

MIT. See [LICENSE](./LICENSE) for the full text. See [ATTRIBUTION.md](./ATTRIBUTION.md) for credits to third-party projects (ccstatusline, ratatui, crossterm, и т.д.).
```

⚠ Если решено НЕ делать `ATTRIBUTION.md` — заменить footer на `MIT. See [LICENSE](./LICENSE).`

- [ ] **Step 6: Создать `MIGRATION.md`**

```markdown
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
| ... (60 entries; full list in [docs/widgets.md](./docs/widgets.md)) |

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
```

- [ ] **Step 7: CHANGELOG entry**

Прочитать текущий `CHANGELOG.md`. Добавить ПЕРЕД существующими entries:

```markdown
## [1.0.0] — 2026-MM-DD

> **Production-grade release.** Two install channels (`npx` + `install.sh`), self-relocation, `cchud doctor`, RC soak gating, npm provenance + 2FA + SHA-pinned actions.

### Added
- **`npx --yes cchud@1.0.0 install`** — primary install channel via npm Approach 2 (`optionalDependencies` per platform). 6 packages: `cchud` + `@cchud/cli-{darwin-arm64,darwin-x64,linux-x64,linux-x64-musl,win32-x64}`. Published with `--provenance` (sigstore attestations).
- **`curl install.sh | sh`** — secondary channel. POSIX sh, sha256 verify, glibc/musl detection.
- **Self-relocation:** `cchud install` копирует себя в `~/.local/bin/cchud` (Unix) или `%LOCALAPPDATA%\cchud\cchud.exe` (Windows). Idempotent. Stable через `nvm use`. `--no-relocate` flag для разработки.
- **`cchud doctor`** — 9-check environment report (version, binary path, platform, color, hyperlinks, cache, claude settings, cchud config, gh CLI). Exit 0/1/2. `--json` для скриптинга.
- **5 cross-platform releases:** macOS arm64, macOS x64, Linux x64 (gnu + musl), Windows x64.
- **`MIGRATION.md`** — step-by-step guide для пользователей ccstatusline.
- **PATH check + shell-specific suggestion** при `cchud install` (zsh/bash/fish).

### Changed
- **Cargo version:** 0.9.0 → 1.0.0 через 1.0.0-rc.1 RC cycle.
- **`~/.claude/settings.json` backup:** перед write создаётся `<path>.bak.<unix-ms>` (best-effort, не fatal). Reuse паттерна Phase 8 atomic_save.
- **README:** добавлены Install, Verify, Migration sections. Версия в одной строке (pinned, никогда `@latest` — supply-chain mitigation).

### Security
- npm 2FA (`auth-and-writes`).
- `NPM_TOKEN` тип `--type=automation` (scoped, не имеет права менять профиль).
- All GitHub Actions pinned по 40-char SHA (не tag).
- npm publish с `--provenance` flag — sigstore attestations доступны через `npm view cchud@1.0.0 --json | jq .dist.attestations`.

### CI
- New `.github/workflows/release.yml` — 5-job pipeline (build × 5 → GitHub Release → npm publish 6 packages → smoke-install × 3 → rollback handler).
- Smoke-install matrix gate: реальный `npx --yes cchud@<version> install` против live npm registry на свежих GH-managed runners (mac/linux/win) перед declarated success.

### Tests
- +8 install_relocate (canonical_target_path, same_file, relocate_to с симлинками, idempotent overwrite, mode 0755, shell-specific PATH hints).
- +6 doctor (all-pass, invalid claude settings, missing binary fail, gh skip без git-pr widget, exit 2 on warn, JSON serialize).
- +4 install_sh (extract+chmod, sha256 mismatch, idempotent re-run, unreachable url).
- +3 npm_shim (spawn fixture, missing native package, bogus CCHUD_NPM_PACKAGE).
- Total ≥ 230 tests.
```

- [ ] **Step 8 (optional): `ATTRIBUTION.md`**

Если решено создать:

```markdown
# Attributions

cchud builds on the work of:

- **[ccstatusline](https://github.com/sirmalloc/ccstatusline)** — original Claude Code statusline; cchud's widget set, themes, and config schema are inspired by it. We re-implemented from scratch in Rust for performance.
- **[ratatui](https://github.com/ratatui-org/ratatui)** — the TUI library used in `cchud configure`.
- **[crossterm](https://github.com/crossterm-rs/crossterm)** — terminal control crate.
- **[serde](https://github.com/serde-rs/serde)** — serialization framework.
- **[clap-style argv parsing]** — we hand-rolled to keep binary small; inspired by clap.

Logo / brand: independent. ccstatusline trademarks belong to their owners.

License: MIT. See [LICENSE](./LICENSE).
```

(Если не создаём — убрать ссылку из README footer.)

- [ ] **Step 9: Verify links**

Quick check, что внутренние ссылки работают:

```bash
ls README.md MIGRATION.md CHANGELOG.md
ls LICENSE  # должен существовать ещё с Phase 0
ls docs/widgets.md  # если ссылка использована
```

- [ ] **Step 10: Commit**

```bash
git add README.md MIGRATION.md CHANGELOG.md
# Если создал ATTRIBUTION.md:
git add ATTRIBUTION.md
git commit -m "$(cat <<'EOF'
docs(phase-9): T12 README + MIGRATION + CHANGELOG — 1.0.0 release docs

README incremental:
- Install section (npx + curl, pinned 1.0.0)
- Verify section (provenance + sha256)
- Migration section + ссылка на MIGRATION.md
- License footer + ATTRIBUTION ссылка

MIGRATION.md (~120 LOC):
- TL;DR
- 60-entry side-by-side widget map (PascalCase → kebab-case)
- Behavioural diffs (Powerline, custom-cmd argv, AlignRight deprecated, settings location)
- Rollback procedure (settings.json.bak.<unix-ms> recovery)
- Known limitations (Windows, ARM Linux, backup accumulation)

CHANGELOG.md 1.0.0 entry: full feature list + Security + CI + Tests sections.
EOF
)"
```
