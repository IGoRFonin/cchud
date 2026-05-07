# Task 14 — `v1.0.0` final release + DoD verification

**Цель:** Bump Cargo версии до `1.0.0`, обновить CHANGELOG date, tag `v1.0.0`, дождаться зелёного `release.yml` (промоушн dist-tag `next` → `latest`), пройти полный DoD checklist, объявить релиз.

**Files:**
- Modify: `Cargo.toml:3` — `version = "1.0.0-rc.1"` → `version = "1.0.0"`
- Modify: `Cargo.lock` — auto-updated by `cargo build`.
- Modify: `CHANGELOG.md` — replace `2026-MM-DD` placeholder реальной датой релиза.

⚠ **Pre-condition:** T13 завершён, `manual-soak-log.md` зелёный для всех 5 environments, прошло ≥ 24h с момента публикации `v1.0.0-rc.1`.

---

- [ ] **Step 1: Pre-flight verification**

```bash
# 1. Soak log заполнен?
rtk grep -c '\[x\]' plan/phase-9/manual-soak-log.md
# Expected: ≥ 25 (5 envs × ≥5 checkbox'ов).

# 2. ≥ 24 часа с публикации rc.1?
date -u
gh release view v1.0.0-rc.1 --json publishedAt --jq '.publishedAt'

# 3. Working tree clean?
git status
```

- [ ] **Step 2: Bump Cargo.toml версии**

В `Cargo.toml`:

```toml
version = "1.0.0"
```

- [ ] **Step 3: Update CHANGELOG date**

В `CHANGELOG.md` найти `## [1.0.0] — 2026-MM-DD` (T12 placeholder) и заменить `MM-DD` реальной датой релиза, например `## [1.0.0] — 2026-05-03`.

- [ ] **Step 4: `cargo build --release --locked` чтобы обновить Cargo.lock**

```bash
cargo build --release --locked
```

Expected: success, Cargo.lock updated.

- [ ] **Step 5: Final test sweep**

```bash
cargo test --locked
cargo test --locked --no-default-features
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features -- -D warnings
cargo fmt --check
```

Expected: всё зелёное.

- [ ] **Step 6: Commit + push**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "$(cat <<'EOF'
release: 1.0.0 — Phase 9 (Distribution + production-grade CLI)

Bump 1.0.0-rc.1 → 1.0.0 после успешного 24h+ soak на 5 environments.

См. CHANGELOG.md и docs/superpowers/specs/2026-05-02-phase-9-distribution-design.md
для full feature list.
EOF
)"
git push origin main
```

- [ ] **Step 7: Tag `v1.0.0` + push**

```bash
git tag -a v1.0.0 -m "$(cat <<'EOF'
v1.0.0 — Production-grade Claude Code statusline

- npx --yes cchud@1.0.0 install — primary install (npm Approach 2 + provenance)
- curl install.sh | sh           — Mac/Linux gnu/musl secondary
- ~/.local/bin/cchud             — stable, nvm-immune
- cchud doctor                   — 9-check environment report
- cchud configure                — interactive TUI (Phase 8)
- cchud import                   — migrate from ccstatusline (Phase 8)
- 5 platform releases: macOS arm64/x64, Linux gnu/musl x64, Windows x64
- npm 2FA + automation token + SHA-pinned actions

Migration path: see MIGRATION.md.
Verify provenance: npm view cchud@1.0.0 --json | jq .dist.attestations
EOF
)"
git push origin v1.0.0
```

- [ ] **Step 8: Watch release.yml**

```bash
gh run watch
```

Expected: те же 5 jobs зелёные (build × 5 → release → npm-publish → smoke × 3). Различие — `dist-tag: latest` (т.к. version = `1.0.0` без `-rc.*` суффикса). GitHub Release НЕ помечен Pre-release.

⚠ Если **любой** smoke job fail на final релизе — `rollback-on-smoke-fail` job сработает и сделает `npm dist-tag rm cchud latest`. **Сам v1.0.0 в registry останется**, но npx `cchud@latest` будет резолвиться на старую версию (или ничего, если первый релиз). Investigate, fix, bump до 1.0.1.

- [ ] **Step 9: Verify публичные artifacts**

```bash
# 1. GitHub Release v1.0.0 stable (не pre-release):
gh release view v1.0.0 --json isPrerelease,assets
# Expected: isPrerelease: false, 10 assets.

# 2. npm dist-tag:
npm view cchud --json | jq '."dist-tags"'
# Expected: { "latest": "1.0.0", "next": "1.0.0-rc.1" } (или next пуст).

# 3. Provenance:
npm view cchud@1.0.0 --json | jq '.dist.attestations'
# Expected: not null.

# 4. Каждая platform package:
for pkg in cli-darwin-arm64 cli-darwin-x64 cli-linux-x64 cli-linux-x64-musl cli-win32-x64; do
  npm view @cchud/$pkg@1.0.0 --json | jq '.name + " " + .version'
done
```

- [ ] **Step 10: Real-world install verification**

В свежем shell на dev-машине:

```bash
rm -f ~/.local/bin/cchud
npx --yes cchud@1.0.0 install
~/.local/bin/cchud --version
# Expected: 1.0.0
~/.local/bin/cchud doctor
echo "exit=$?"
# Expected: 0 или 2.
```

- [ ] **Step 11: DoD checklist (full sweep)**

См. spec § "Definition of Done" — пройти каждый чек:

### Code
- [ ] `release.yml` зелёный на `v1.0.0` теге.
- [ ] `release.yml` smoke-install matrix зелёный.
- [ ] `install.sh` работает на macOS arm + Linux gnu/musl.
- [ ] `npm/` директория с 6 sub-packages — published.
- [ ] `npm/cchud/bin/cchud.js` JS shim ~50 LOC, без external deps.
- [ ] `scripts/build-npm-packages.sh` собирает 6 npm packages.
- [ ] `src/commands/install.rs` self-relocation + PATH check + backup.
- [ ] `src/commands/doctor.rs` 9 checks, exit 0/1/2.
- [ ] `src/main.rs` `cchud doctor` subcommand + help.

### Documentation
- [ ] README: Install + Verify + Migration + footer.
- [ ] `MIGRATION.md` создан.
- [ ] `CHANGELOG.md` 1.0.0 entry с реальной датой.
- [ ] `Cargo.toml` version: `1.0.0`.
- [ ] `docs/DECISIONS.md` D-2026-05-02.
- [ ] `plan/phase-9/manual-soak-log.md` заполнен.

### Tests
- [ ] `cargo test --locked` зелёный (≥230 tests).
- [ ] `cargo test --locked --no-default-features` зелёный.
- [ ] `cargo clippy --locked --all-targets -- -D warnings` зелёный.
- [ ] `cargo fmt --check` зелёный.
- [ ] CI matrix зелёный.
- [ ] Никаких unwrap/expect в `src/commands/{install,doctor}.rs`.

### Release artifacts
- [ ] `v1.0.0-rc.1` тег → 5 платформ Pre-release + 6 npm packages `--tag next`.
- [ ] Manual soak ≥ 24h × 4 envs (5 включая Windows).
- [ ] `v1.0.0` тег → промоушн `--tag latest` + GitHub stable.
- [ ] `npx --yes cchud@1.0.0 install` работает на mac/linux/windows.
- [ ] `curl install.sh | sh` работает на mac/linux.
- [ ] `cchud doctor` exits 0 на свежем install.
- [ ] Binary size: < 10 MB на всех платформах.
  ```bash
  for f in cchud-darwin-arm64 cchud-darwin-x64 cchud-linux-x64 cchud-linux-x64-musl cchud-win32-x64; do
    size=$(curl -sI "https://github.com/IGoRFonin/cchud/releases/download/v1.0.0/${f}.tar.gz" | grep -i content-length | awk '{print $2}' | tr -d '\r')
    echo "$f: $size bytes"
  done
  ```

### Security
- [ ] npm 2FA включён.
- [ ] `NPM_TOKEN` тип `--type=automation`.
- [ ] All GH Actions pinned по 40-char SHA.
- [ ] `npm publish --provenance` — attestations доступны.
- [ ] README "Verify install" документирует provenance check.

- [ ] **Step 12: Announce / wrap-up**

Опционально:
- Update `plan/README.md` с пометкой "Phase 9 shipped 2026-05-XX".
- Tweet / blog / Slack — ссылка на GitHub Release + `npx --yes cchud@1.0.0 install`.

```bash
# Final commit (если были touch'ы):
git status
# Если что-то в working tree (например plan/README.md update):
git add plan/README.md
git commit -m "docs: mark Phase 9 (1.0.0) as shipped 2026-05-XX"
git push
```

- [ ] **Step 13: Backlog handoff**

Phase 10 backlog уже задокументирован в `plan/phase-10-future.md`:
- auto-update mechanism (10.3)
- winget/scoop (10.8) / AUR (10.9) / Nix (10.10)
- ARM Linux target
- code-signing (Windows, macOS)
- install.ps1
- ATTRIBUTION.md (если не сделан)
- `cchud uninstall`

Phase 9 — done. 🎉
