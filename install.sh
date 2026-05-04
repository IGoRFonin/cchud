#!/bin/sh
# cchud install script. POSIX sh, macOS + Linux (gnu / musl).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/IGoRFonin/cchud/main/install.sh | CCHUD_VERSION=1.0.0 sh
#   CCHUD_VERSION=1.0.0 sh install.sh
#
# Env vars:
#   CCHUD_VERSION       — release version to install (default: pinned).
#   CCHUD_INSTALL_DIR   — target directory (default: $HOME/.local/bin).
#   CCHUD_BASE_URL      — base URL for tarballs (default: GitHub Releases).
#                         Used by tests to point at a local mock server.
#
set -e

REPO="IGoRFonin/cchud"
VERSION="${CCHUD_VERSION:-1.0.0}"
INSTALL_DIR="${CCHUD_INSTALL_DIR:-$HOME/.local/bin}"
BASE_URL="${CCHUD_BASE_URL:-https://github.com/$REPO/releases/download/v$VERSION}"

# 1. Detect OS-arch + libc.
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$OS-$ARCH" in
  darwin-arm64|darwin-aarch64) TARGET="cchud-darwin-arm64" ;;
  darwin-x86_64) TARGET="cchud-darwin-x64" ;;
  linux-x86_64)
    if ldd --version 2>&1 | grep -qi musl; then
      TARGET="cchud-linux-x64-musl"
    else
      TARGET="cchud-linux-x64"
    fi
    ;;
  *)
    echo "cchud: unsupported $OS-$ARCH" >&2
    exit 1
    ;;
esac

# 2. Download tarball + sha256.
URL="$BASE_URL/${TARGET}.tar.gz"
SHA_URL="${URL}.sha256"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL -o "$TMP/cchud.tar.gz" "$URL"
curl -fsSL -o "$TMP/cchud.sha256" "$SHA_URL"

# 3. Verify sha256 (cross-platform).
if command -v sha256sum >/dev/null 2>&1; then
  HASHER="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  HASHER="shasum -a 256"
else
  echo "cchud: neither sha256sum nor shasum available" >&2
  exit 1
fi
EXPECTED=$(cut -d' ' -f1 < "$TMP/cchud.sha256")
ACTUAL=$(cd "$TMP" && $HASHER cchud.tar.gz | cut -d' ' -f1)
if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "cchud: sha256 mismatch! expected=$EXPECTED actual=$ACTUAL" >&2
  exit 1
fi

# 4. Extract + chmod + move.
tar xzf "$TMP/cchud.tar.gz" -C "$TMP"
mkdir -p "$INSTALL_DIR"
mv "$TMP/cchud" "$INSTALL_DIR/cchud"
chmod 0755 "$INSTALL_DIR/cchud"

# 5. PATH check.
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "  Add to your shell rc:"
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

# 6. Wire Claude Code (idempotent, soft-fail OK).
"$INSTALL_DIR/cchud" install || {
  echo "WARNING: Could not auto-wire Claude Code. Run manually: $INSTALL_DIR/cchud install"
}

echo "cchud $VERSION installed to $INSTALL_DIR/cchud"
