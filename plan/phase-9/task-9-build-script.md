# Task 9 — `scripts/build-npm-packages.sh`

**Цель:** POSIX shell script, который принимает `<version>` (e.g. `1.0.0-rc.1`) и выполняет: (1) substitutes `0.0.0-PLACEHOLDER` → `<version>` во всех 6 `npm/*/package.json`, (2) копирует native binaries из downloaded GitHub artifacts в `npm/cli-*/bin/cchud[.exe]`, (3) chmod 0755 на Unix.

**Files:**
- Create: `scripts/build-npm-packages.sh` (executable)
- Create: `scripts/README.md` (опционально — объяснение)

---

- [ ] **Step 1: `scripts/build-npm-packages.sh`**

```bash
#!/bin/sh
# Phase 9 — build-npm-packages.sh
#
# Substitutes 0.0.0-PLACEHOLDER → <version> in all 6 npm/*/package.json files
# and copies per-platform native binaries from artifact directories into
# npm/cli-*/bin/cchud[.exe].
#
# Usage:
#   ./scripts/build-npm-packages.sh <version>
#
# Inputs:
#   $1                — version string (e.g. "1.0.0-rc.1" or "1.0.0").
#   ./artifacts/      — flat or nested directory containing 5 GitHub artifacts:
#                       cchud-darwin-arm64.tar.gz, cchud-darwin-x64.tar.gz,
#                       cchud-linux-x64.tar.gz, cchud-linux-x64-musl.tar.gz,
#                       cchud-win32-x64.tar.gz
#                     (Path can be overridden via $CCHUD_ARTIFACTS_DIR.)
#
# Outputs:
#   npm/cchud/package.json                    — version substituted
#   npm/cli-<platform>/package.json           — version substituted
#   npm/cli-<platform>/bin/cchud[.exe]        — extracted from tarball
#
# Errors out (set -e) if any tarball is missing or extraction fails.

set -e

if [ -z "$1" ]; then
  echo "usage: $0 <version>" >&2
  exit 2
fi

VERSION="$1"
ARTIFACTS_DIR="${CCHUD_ARTIFACTS_DIR:-./artifacts}"

# Mapping: artifact basename (without .tar.gz) → npm package directory.
PLATFORMS="darwin-arm64 darwin-x64 linux-x64 linux-x64-musl win32-x64"

substitute_version() {
  pkg="$1"
  file="npm/$pkg/package.json"
  if [ ! -f "$file" ]; then
    echo "missing: $file" >&2
    exit 1
  fi
  # POSIX-compatible in-place substitution via tempfile.
  tmp="$file.tmp.$$"
  sed "s/0\\.0\\.0-PLACEHOLDER/${VERSION}/g" "$file" > "$tmp"
  mv "$tmp" "$file"
  echo "✓ substituted version in $file"
}

extract_binary() {
  platform="$1"
  artifact="$ARTIFACTS_DIR/cchud-${platform}.tar.gz"
  # GH actions/download-artifact@v4 без `merge-multiple: true` создаёт subdir
  # для каждого artifact; пробуем оба варианта.
  if [ ! -f "$artifact" ]; then
    artifact_subdir="$ARTIFACTS_DIR/cchud-${platform}/cchud-${platform}.tar.gz"
    if [ -f "$artifact_subdir" ]; then
      artifact="$artifact_subdir"
    fi
  fi
  if [ ! -f "$artifact" ]; then
    echo "missing artifact: cchud-${platform}.tar.gz (looked in $ARTIFACTS_DIR)" >&2
    exit 1
  fi

  pkg_dir="npm/cli-${platform}"
  bin_dir="$pkg_dir/bin"
  rm -rf "$bin_dir"
  mkdir -p "$bin_dir"

  # Tarball содержит cchud (или cchud.exe для Windows) в корне (см. release.yml T11).
  tar xzf "$artifact" -C "$bin_dir"

  if [ "$platform" = "win32-x64" ]; then
    if [ ! -f "$bin_dir/cchud.exe" ]; then
      echo "missing cchud.exe in $artifact" >&2
      exit 1
    fi
  else
    if [ ! -f "$bin_dir/cchud" ]; then
      echo "missing cchud in $artifact" >&2
      exit 1
    fi
    chmod 0755 "$bin_dir/cchud"
  fi

  echo "✓ extracted binary into $bin_dir/"
}

# Step 1: substitute versions in main package + 5 platform packages.
substitute_version "cchud"
for p in $PLATFORMS; do
  substitute_version "cli-${p}"
done

# Step 2: extract per-platform binaries.
for p in $PLATFORMS; do
  extract_binary "$p"
done

echo
echo "✓ All 6 npm packages ready (version: $VERSION)"
echo "  → cd npm/cchud && npm publish --provenance --access public --tag <next|latest>"
echo "  → cd npm/cli-<platform> && npm publish --provenance --access public --tag <next|latest>"
```

Сохранить и сделать executable:

```bash
chmod +x scripts/build-npm-packages.sh
```

- [ ] **Step 2: Dry-run smoke**

Создать fake artifact directory:

```bash
mkdir -p /tmp/cchud-art-test
cd /tmp/cchud-art-test
echo "fake binary darwin-arm64" > cchud
tar czf cchud-darwin-arm64.tar.gz cchud
echo "fake binary darwin-x64" > cchud
tar czf cchud-darwin-x64.tar.gz cchud
echo "fake binary linux-x64" > cchud
tar czf cchud-linux-x64.tar.gz cchud
echo "fake binary linux-x64-musl" > cchud
tar czf cchud-linux-x64-musl.tar.gz cchud
echo "fake binary win32-x64" > cchud.exe
tar czf cchud-win32-x64.tar.gz cchud.exe
rm cchud cchud.exe
cd /Users/igor/mp/startup/cchud

CCHUD_ARTIFACTS_DIR=/tmp/cchud-art-test ./scripts/build-npm-packages.sh 1.0.0-rc.1
```

Expected output: `✓ substituted version` × 6, `✓ extracted binary` × 5, итог.

Проверить:

```bash
jq -r .version npm/cchud/package.json
# 1.0.0-rc.1
ls -la npm/cli-darwin-arm64/bin/
# cchud (mode 0755, не пустой)
cat npm/cli-darwin-arm64/bin/cchud
# fake binary darwin-arm64
```

⚠ После dry-run **обязательно** revert: 

```bash
git checkout -- npm/cchud/package.json npm/cli-*/package.json
rm -rf npm/cli-*/bin
rm -rf /tmp/cchud-art-test
```

(Чтобы PLACEHOLDER не попал в commit и binaries не оказались в git tree.)

- [ ] **Step 3: shellcheck (optional but recommended)**

```bash
shellcheck scripts/build-npm-packages.sh
```

Expected: clean. Если warnings — исправить.

- [ ] **Step 4: Commit**

```bash
git add scripts/build-npm-packages.sh
git commit -m "$(cat <<'EOF'
build(phase-9): T9 build-npm-packages.sh — version substitution + artifact extraction

POSIX sh script для release.yml::npm-publish job:
- $1 = version, $CCHUD_ARTIFACTS_DIR = ./artifacts (override-able)
- Substitutes 0.0.0-PLACEHOLDER → version в 6 package.json
- Extracts cchud[.exe] из 5 tarballs в npm/cli-*/bin/
- chmod 0755 на Unix; .exe для Windows
- set -e fail-fast если artifact отсутствует

Dry-run на /tmp/cchud-art-test prouvé. Revert после.
EOF
)"
```
