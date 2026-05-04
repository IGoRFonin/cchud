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
