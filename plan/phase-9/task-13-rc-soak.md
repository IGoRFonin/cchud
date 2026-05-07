# Task 13 — `v1.0.0-rc.1` тег + manual soak ≥ 24h × 4 environments

**Цель:** Финальная подготовка к RC. Резолв всех `<SHA>` placeholder'ов в release.yml. Pre-flight verification (npm 2FA, NPM_TOKEN type, dry-run provenance). `git tag v1.0.0-rc.1`. Wait for release.yml зелёный (build × 5 + release + npm-publish + smoke-install × 3). Manual soak ≥ 24h на 4 environments. Лог в `manual-soak-log.md`.

**Files:**
- Modify: `.github/workflows/release.yml` — replace `<SHA>` placeholder'ы реальными SHA-pins.
- Modify: `plan/phase-9/manual-soak-log.md` — заполнить real soak data.

⚠ **Этот task занимает ≥ 24 часа реального времени.** RC soak — обязательный gating step перед `v1.0.0` final.

---

- [ ] **Step 1: Pre-flight checks**

```bash
# 1. npm 2FA включён?
npm profile get 2fa
# Expected: "auth-and-writes"

# 2. NPM_TOKEN тип automation?
# Manual: открыть npm-cli/Settings → Access Tokens → проверить
# что используемый token — type "automation".
# (Web UI; нет CLI команды.)

# 3. GH repo secrets:
gh secret list --repo IGoRFonin/cchud
# Expected: NPM_TOKEN listed.

# 4. cchud имя занято?
npm view cchud --json | jq '.name'
# Если "cchud" — defensive registration уже сделан (или это наш пакет).
# Если 404 — самое то.

# 5. Dry-run provenance на test-package (один раз — пропустить если уже сделано):
mkdir -p /tmp/prov-test && cd /tmp/prov-test
cat > package.json <<'EOF'
{
  "name": "cchud-provenance-test",
  "version": "0.0.1",
  "publishConfig": { "access": "public", "provenance": true }
}
EOF
# (Run только если хочешь действительно проверить provenance pipeline.)
# npm publish --provenance --access public --dry-run
cd /Users/igor/mp/startup/cchud
```

- [ ] **Step 2: Резолвить SHA-pins для всех GH Actions**

```bash
# Сохранить SHA для каждого:
ACT_CHECKOUT=$(gh api repos/actions/checkout/git/refs/tags/v6 --jq '.object.sha')
ACT_TOOLCHAIN=$(gh api repos/dtolnay/rust-toolchain/git/refs/heads/master --jq '.object.sha')
ACT_UPLOAD=$(gh api repos/actions/upload-artifact/git/refs/tags/v4 --jq '.object.sha')
ACT_DOWNLOAD=$(gh api repos/actions/download-artifact/git/refs/tags/v4 --jq '.object.sha')
ACT_GHRELEASE=$(gh api repos/softprops/action-gh-release/git/refs/tags/v2 --jq '.object.sha')
ACT_NODE=$(gh api repos/actions/setup-node/git/refs/tags/v4 --jq '.object.sha')

echo "checkout=$ACT_CHECKOUT"
echo "toolchain=$ACT_TOOLCHAIN"
echo "upload=$ACT_UPLOAD"
echo "download=$ACT_DOWNLOAD"
echo "ghrelease=$ACT_GHRELEASE"
echo "node=$ACT_NODE"
```

(Каждое значение — 40-char hex. Записать в task-файл для аудит-трейла.)

- [ ] **Step 3: Заменить `<SHA>` placeholder'ы в `release.yml`**

```bash
sed -i.bak \
  -e "s|actions/checkout@<SHA>|actions/checkout@${ACT_CHECKOUT}|g" \
  -e "s|dtolnay/rust-toolchain@<SHA>|dtolnay/rust-toolchain@${ACT_TOOLCHAIN}|g" \
  -e "s|actions/upload-artifact@<SHA>|actions/upload-artifact@${ACT_UPLOAD}|g" \
  -e "s|actions/download-artifact@<SHA>|actions/download-artifact@${ACT_DOWNLOAD}|g" \
  -e "s|softprops/action-gh-release@<SHA>|softprops/action-gh-release@${ACT_GHRELEASE}|g" \
  -e "s|actions/setup-node@<SHA>|actions/setup-node@${ACT_NODE}|g" \
  .github/workflows/release.yml
rm .github/workflows/release.yml.bak
```

Verify не осталось `<SHA>`:

```bash
rtk grep -n '<SHA>' .github/workflows/release.yml
# Expected: 0 matches
```

- [ ] **Step 4: actionlint clean**

```bash
actionlint .github/workflows/release.yml
```

Expected: clean.

- [ ] **Step 5: Commit SHA-pins**

```bash
git add .github/workflows/release.yml
git commit -m "$(cat <<'EOF'
ci(phase-9): T13 release.yml SHA-pin actions before v1.0.0-rc.1 tag

Резолв всех <SHA> placeholder'ов реальными commit SHAs:
- actions/checkout@<sha>
- dtolnay/rust-toolchain@<sha>
- actions/upload-artifact@<sha>
- actions/download-artifact@<sha>
- softprops/action-gh-release@<sha>
- actions/setup-node@<sha>

Supply-chain mitigation: tag references могут быть переустановлены
malicious commit'ами. SHA refs immutable.
EOF
)"
```

- [ ] **Step 6: Push main branch (если есть unpushed commits)**

```bash
git push origin main
```

- [ ] **Step 7: Tag `v1.0.0-rc.1` + push**

```bash
git tag -a v1.0.0-rc.1 -m "$(cat <<'EOF'
v1.0.0-rc.1 — Phase 9 Release Candidate 1

Distribution + 1.0.0 RC cycle:
- npm Approach 2 (cchud + 5× @cchud/cli-*) с --provenance
- install.sh (POSIX) для Mac/Linux gnu+musl
- self-relocation в ~/.local/bin/cchud (Unix) / %LOCALAPPDATA%\cchud (Win)
- new cchud doctor (9 checks)
- 5 cross-platform releases

Soak ≥ 24h на 4 environments перед v1.0.0 final.
EOF
)"
git push origin v1.0.0-rc.1
```

- [ ] **Step 8: Wait + monitor release.yml**

```bash
# Open GH Actions UI:
gh run watch
# или:
gh run list --workflow=release.yml --limit 5
gh run view <run-id> --log-failed
```

Expected progression:
- `build` (5 jobs) — 3-5 min total. Все зелёные.
- `release` (1 job) — 30 sec. GitHub Release v1.0.0-rc.1 (Pre-release) с 10 файлами (5 .tar.gz + 5 .sha256).
- `npm-publish` (1 job) — 1-2 min. 6 packages опубликованы с `--tag next` + `--provenance`.
- `smoke-install` (3 jobs) — 1-2 min total. Все зелёные.

⚠ Если **любой** job fail — stop. Investigate. Fix. Bump до `v1.0.0-rc.2` (delete `v1.0.0-rc.1` тег + `npm unpublish cchud@1.0.0-rc.1` ВОЗМОЖЕН в течение 72h).

- [ ] **Step 9: Verify публичные artifacts**

```bash
# GitHub Release:
gh release view v1.0.0-rc.1
# Expected: Pre-release flag, 10 assets.

# npm dist-tag:
npm view cchud --json | jq '."dist-tags"'
# Expected: { "next": "1.0.0-rc.1", ... }

# Provenance attestation:
npm view cchud@1.0.0-rc.1 --json | jq '.dist.attestations'
# Expected: not null, with sigstore data.

# Each platform package published:
npm view @cchud/cli-darwin-arm64@1.0.0-rc.1
npm view @cchud/cli-darwin-x64@1.0.0-rc.1
npm view @cchud/cli-linux-x64@1.0.0-rc.1
npm view @cchud/cli-linux-x64-musl@1.0.0-rc.1
npm view @cchud/cli-win32-x64@1.0.0-rc.1
```

Expected: all 5 platform packages exist + provenance attestations.

- [ ] **Step 10: Manual soak: Environment 1 — macOS Apple Silicon (host)**

```bash
# Свежий shell, чистый ~/.local/bin/cchud:
rm -f ~/.local/bin/cchud
mv ~/.claude/settings.json ~/.claude/settings.json.pre-rc-soak  # bкап реальной конфигурации

npx --yes cchud@1.0.0-rc.1 install
# Expected: "cchud installed: ~/.local/bin/cchud"
ls -la ~/.local/bin/cchud  # mode 0755
~/.local/bin/cchud --version
# Expected: 1.0.0-rc.1
~/.local/bin/cchud doctor
echo "exit=$?"
# Expected: exit 0 или 2 (warn без fail OK).
~/.local/bin/cchud configure
# 5+ минут реальной сессии: add/delete/reorder, save, restart Claude Code.
```

Заполнить checklist в `plan/phase-9/manual-soak-log.md` Environment 1. Восстановить настоящие settings:

```bash
mv ~/.claude/settings.json.pre-rc-soak ~/.claude/settings.json
```

- [ ] **Step 11: Environment 2 — macOS Intel**

Можно через VM (Parallels/UTM) или старый Mac. Тот же checklist.

- [ ] **Step 12: Environment 3 — Linux gnu (Ubuntu 22.04 VM)**

```bash
# В VM:
npx --yes cchud@1.0.0-rc.1 install     # Через npm
# Затем:
rm ~/.local/bin/cchud
curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | CCHUD_VERSION=1.0.0-rc.1 sh
# Дважды — npx и curl, оба должны работать.
~/.local/bin/cchud doctor
```

Expected: оба channels exit 0/2, doctor зелёный.

- [ ] **Step 13: Environment 4 — Linux musl (Alpine 3.20 docker)**

```bash
docker run -it --rm alpine:3.20 sh -c '
  apk add --no-cache nodejs npm curl
  npx --yes cchud@1.0.0-rc.1 install
  ~/.local/bin/cchud doctor
  ~/.local/bin/cchud --version
'
```

Expected: exit 0, version 1.0.0-rc.1.

- [ ] **Step 14: Environment 5 — Windows (Win 11 VM, PowerShell)**

```powershell
# Если Defender SmartScreen блокирует — "More info" → "Run anyway".
npx --yes cchud@1.0.0-rc.1 install
& "$env:LOCALAPPDATA\cchud\cchud.exe" --version
& "$env:LOCALAPPDATA\cchud\cchud.exe" doctor
```

Expected: exit 0, version 1.0.0-rc.1.

- [ ] **Step 15: Заполнить `manual-soak-log.md` для всех 5 environments**

Checklist'и в каждой секции — все ✓. Notes — реальные наблюдения (Defender screen? Render speed? Что-то странное?).

- [ ] **Step 16: Commit soak log**

```bash
git add plan/phase-9/manual-soak-log.md
git commit -m "$(cat <<'EOF'
docs(phase-9): T13 manual soak log — v1.0.0-rc.1 verified on 5 environments

24h+ soak completed на:
- macOS Apple Silicon (host)
- macOS Intel (VM)
- Linux gnu (Ubuntu 22.04 VM)
- Linux musl (Alpine 3.20 docker)
- Windows 11 (VM, PowerShell)

Все environments: install OK, --version OK, doctor exit 0/2, configure 5+ min OK,
real Claude Code session statusline renders correctly.

Cleared для v1.0.0 final tag (T14).
EOF
)"
git push
```

- [ ] **Step 17: Decision gate — go/no-go for T14**

Если **любой** environment failed → stop. Не tag'аем v1.0.0. Bump до `v1.0.0-rc.2`, fix issue, повторить с Step 7.

Если все 5 environments зелёные ≥ 24h → proceed to T14.
