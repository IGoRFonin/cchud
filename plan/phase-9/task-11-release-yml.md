# Task 11 — `.github/workflows/release.yml`

**Цель:** Создать GitHub Actions workflow с 4 sequential jobs: `build` (× 5 платформ) → `release` (single ubuntu, GH Release с prerelease=true для `-rc.*`) → `npm-publish` (single ubuntu, 6 packages с `--provenance --tag <next|latest>`) → `smoke-install` (× 3 OS, npx-based end-to-end). Все actions pinned по 40-char SHA.

**Files:**
- Create: `.github/workflows/release.yml`

---

- [ ] **Step 1: Resolve current SHA для каждого pinned action**

Перед написанием workflow — получить актуальные SHAs (сделать ОДИН РАЗ, перед commit):

```bash
# Бери последний release tag, потом resolve commit:
gh api repos/actions/checkout/commits/v6 --jq '.sha'         # для actions/checkout
gh api repos/dtolnay/rust-toolchain/commits/master --jq '.sha' # для rust-toolchain (rolling)
gh api repos/actions/upload-artifact/commits/v4 --jq '.sha'  # для upload-artifact
gh api repos/actions/download-artifact/commits/v4 --jq '.sha' # для download-artifact
gh api repos/softprops/action-gh-release/commits/v2 --jq '.sha' # для action-gh-release
gh api repos/actions/setup-node/commits/v4 --jq '.sha'       # для setup-node
```

Записать значения в этот task-файл (или в комментарии в workflow). Подставить в `<SHA>` placeholder'ы ниже.

⚠ Если не можешь резолвить через `gh` (например, нет creds) — оставляй `<SHA>` placeholder'ы и финальные SHA-pins получатся в T13 как блокирующее условие перед `git tag v1.0.0-rc.1`.

- [ ] **Step 2: Создать `.github/workflows/release.yml`**

```yaml
name: Release

on:
  push:
    tags: ['v*']

permissions:
  contents: read

jobs:
  # ───────── Job 1: Build matrix × 5 ─────────
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            name: cchud-darwin-arm64
          - os: macos-13
            target: x86_64-apple-darwin
            name: cchud-darwin-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: cchud-linux-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            name: cchud-linux-x64-musl
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: cchud-win32-x64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@<SHA>

      - uses: dtolnay/rust-toolchain@<SHA>
        with:
          toolchain: stable
          targets: ${{ matrix.target }}

      - name: Install musl-tools (musl target only)
        if: contains(matrix.target, 'musl')
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Build release binary
        run: cargo build --release --locked --target ${{ matrix.target }}

      - name: Strip binary (Unix only)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/cchud

      - name: Smoke check (--version + render)
        shell: bash
        run: |
          set -e
          VERSION="${GITHUB_REF#refs/tags/v}"
          # Strip pre-release suffix for grep tolerance.
          VERSION_PREFIX=$(echo "$VERSION" | cut -d- -f1)
          BIN="target/${{ matrix.target }}/release/cchud"
          if [ "${{ matrix.os }}" = "windows-latest" ]; then BIN="${BIN}.exe"; fi
          "$BIN" --version | grep -F "$VERSION_PREFIX"
          "$BIN" doctor --json > /dev/null || true
          # Smoke render via sample payload (если payload присутствует).
          if [ -f benches/samples/payload-cchud-sonnet-xlarge.json ]; then
            "$BIN" < benches/samples/payload-cchud-sonnet-xlarge.json > /dev/null
          fi

      - name: Tar + sha256
        shell: bash
        run: |
          set -e
          BIN_NAME="cchud"
          if [ "${{ matrix.os }}" = "windows-latest" ]; then BIN_NAME="cchud.exe"; fi
          cp "target/${{ matrix.target }}/release/$BIN_NAME" .
          tar czf "${{ matrix.name }}.tar.gz" "$BIN_NAME"
          if command -v sha256sum >/dev/null; then
            sha256sum "${{ matrix.name }}.tar.gz" > "${{ matrix.name }}.tar.gz.sha256"
          else
            shasum -a 256 "${{ matrix.name }}.tar.gz" > "${{ matrix.name }}.tar.gz.sha256"
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@<SHA>
        with:
          name: ${{ matrix.name }}
          path: |
            ${{ matrix.name }}.tar.gz
            ${{ matrix.name }}.tar.gz.sha256
          retention-days: 7

  # ───────── Job 2: GitHub Release ─────────
  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    outputs:
      version: ${{ steps.version.outputs.version }}
      is_prerelease: ${{ steps.version.outputs.is_prerelease }}
    steps:
      - id: version
        shell: bash
        run: |
          V="${GITHUB_REF#refs/tags/v}"
          echo "version=$V" >> "$GITHUB_OUTPUT"
          case "$V" in
            *-rc.*|*-beta.*|*-alpha.*) echo "is_prerelease=true"  >> "$GITHUB_OUTPUT" ;;
            *)                          echo "is_prerelease=false" >> "$GITHUB_OUTPUT" ;;
          esac

      - uses: actions/download-artifact@<SHA>
        with:
          path: artifacts/

      - name: Flatten artifacts
        shell: bash
        run: |
          set -e
          mkdir -p release-files
          find artifacts -name 'cchud-*.tar.gz' -exec mv {} release-files/ \;
          find artifacts -name 'cchud-*.sha256' -exec mv {} release-files/ \;
          ls -la release-files

      - uses: softprops/action-gh-release@<SHA>
        with:
          files: |
            release-files/*.tar.gz
            release-files/*.sha256
          prerelease: ${{ steps.version.outputs.is_prerelease == 'true' }}
          generate_release_notes: true

  # ───────── Job 3: npm publish (6 packages) ─────────
  npm-publish:
    needs: release
    runs-on: ubuntu-latest
    permissions:
      contents: read
      id-token: write   # OIDC for npm provenance
    steps:
      - uses: actions/checkout@<SHA>

      - uses: actions/download-artifact@<SHA>
        with:
          path: artifacts/

      - uses: actions/setup-node@<SHA>
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Generate per-platform npm packages
        env:
          CCHUD_ARTIFACTS_DIR: ./artifacts
        run: ./scripts/build-npm-packages.sh ${{ needs.release.outputs.version }}

      - name: Determine dist-tag
        id: tag
        shell: bash
        run: |
          if [ "${{ needs.release.outputs.is_prerelease }}" = "true" ]; then
            echo "dist_tag=next" >> "$GITHUB_OUTPUT"
          else
            echo "dist_tag=latest" >> "$GITHUB_OUTPUT"
          fi

      - name: Publish 5 platform packages
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        shell: bash
        run: |
          set -e
          for d in npm/cli-*; do
            (cd "$d" && npm publish --provenance --tag "${{ steps.tag.outputs.dist_tag }}" --access public)
          done

      - name: Publish main cchud package
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        shell: bash
        run: |
          set -e
          (cd npm/cchud && npm publish --provenance --tag "${{ steps.tag.outputs.dist_tag }}" --access public)

  # ───────── Job 4: Smoke install matrix × 3 ─────────
  smoke-install:
    needs: npm-publish
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@<SHA>

      - uses: actions/setup-node@<SHA>
        with:
          node-version: '20'

      - name: Wait for npm CDN propagation
        shell: bash
        run: sleep 30

      - name: Install via npx
        shell: bash
        run: |
          set -e
          npx --yes cchud@${{ needs.release.outputs.version }} install

      - name: Verify binary at canonical target
        shell: bash
        run: |
          set -e
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            TARGET="$LOCALAPPDATA/cchud/cchud.exe"
          else
            TARGET="$HOME/.local/bin/cchud"
          fi
          test -x "$TARGET"
          "$TARGET" --version

      - name: Verify Claude settings.json wired
        shell: bash
        run: |
          set -e
          test -f "$HOME/.claude/settings.json"
          grep -q '"command".*cchud' "$HOME/.claude/settings.json"

      - name: Verify render works against sample payload
        shell: bash
        run: |
          set -e
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            TARGET="$LOCALAPPDATA/cchud/cchud.exe"
          else
            TARGET="$HOME/.local/bin/cchud"
          fi
          if [ -f benches/samples/payload-cchud-sonnet-xlarge.json ]; then
            "$TARGET" < benches/samples/payload-cchud-sonnet-xlarge.json > /tmp/render.txt
            test -s /tmp/render.txt
          fi

      - name: Verify cchud doctor exits 0/2 (no fail)
        shell: bash
        run: |
          set +e
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            TARGET="$LOCALAPPDATA/cchud/cchud.exe"
          else
            TARGET="$HOME/.local/bin/cchud"
          fi
          "$TARGET" doctor
          rc=$?
          if [ "$rc" -ne 0 ] && [ "$rc" -ne 2 ]; then
            echo "doctor exited $rc (expected 0 or 2)" >&2
            exit 1
          fi

  # ───────── Job 5: Failure handler (rollback latest tag if smoke fails on stable release) ─────────
  rollback-on-smoke-fail:
    needs: [release, smoke-install]
    if: failure() && needs.release.outputs.is_prerelease == 'false'
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/setup-node@<SHA>
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - name: Roll back npm latest tag pointer
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        shell: bash
        run: |
          set -e
          echo "Smoke install failed for stable release ${{ needs.release.outputs.version }}."
          echo "Removing 'latest' dist-tag to prevent broken installs."
          npm dist-tag rm cchud latest || true
```

- [ ] **Step 3: actionlint check**

```bash
# Установить actionlint если нет:
brew install actionlint || curl -fsSL https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash | bash

actionlint .github/workflows/release.yml
```

Expected: clean. Любые errors — исправить.

⚠ actionlint, скорее всего, ругнётся на `<SHA>` placeholder'ы как на invalid syntax. **Это ожидаемо.** Финальные SHA-pins подставляются в Step 1 этого Task'а как блокирующее условие перед T13.

- [ ] **Step 4: yamllint (optional)**

```bash
yamllint .github/workflows/release.yml || true
```

Expected: minor warnings ОК (line length, trailing whitespace).

- [ ] **Step 5: Verify CI matrix не сломан**

Просто smoke build:

```bash
cargo build --locked --release
```

(Нет смысла тестировать workflow до тега — это будет проверяться в T13.)

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "$(cat <<'EOF'
ci(phase-9): T11 release.yml — 5-job workflow (build × 5 → release → npm-publish → smoke × 3 → rollback)

- Triggered by tags v*. is_prerelease если -rc/-beta/-alpha
- 5 build matrix targets с smoke-render check + tar + sha256
- softprops/action-gh-release с prerelease=$is_prerelease
- npm-publish — id-token: write для provenance, --tag next|latest
- smoke-install × 3 OS: npx → verify ~/.local/bin/cchud → settings.json wired → doctor exits 0/2
- rollback-on-smoke-fail: npm dist-tag rm cchud latest при smoke fail для stable

Все actions помечены <SHA> — финальные SHA-pins резолвятся в T13.
EOF
)"
```
