# Фаза 9 — Дистрибуция и стабилизация

**Длительность:** 3–5 дней
**Входные условия:** Фазы 0–8
**Релиз:** **1.0.0**

## Цель

Pre-built бинари для 5 платформ, npm-пакет (bootstrap-style), Homebrew формула, install.sh-скрипт, подписи релизов, финальная документация. Один `brew install` или `npm i -g cchud` на любой системе.

## Шаги

### 9.1. Cross-build на 5 платформ

Решение: **GitHub Actions matrix runners** (проще `cargo-zigbuild` для старта).

`.github/workflows/release.yml`:
```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - { os: macos-latest,   target: aarch64-apple-darwin,         name: cchud-aarch64-apple-darwin }
          - { os: macos-13,       target: x86_64-apple-darwin,          name: cchud-x86_64-apple-darwin }
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu,     name: cchud-x86_64-unknown-linux-gnu }
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-musl,    name: cchud-x86_64-unknown-linux-musl }
          - { os: windows-latest, target: x86_64-pc-windows-msvc,       name: cchud-x86_64-pc-windows-msvc.exe }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - name: Linux musl deps
        if: contains(matrix.target, 'musl')
        run: sudo apt-get install -y musl-tools
      - run: cargo build --release --locked --target ${{ matrix.target }}
      - name: Strip
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/cchud
      - name: Smoke test
        run: |
          ./target/${{ matrix.target }}/release/cchud --version
          cat benches/samples/payload-001.json | ./target/${{ matrix.target }}/release/cchud
      - name: Compute sha256
        run: shasum -a 256 target/${{ matrix.target }}/release/cchud > ${{ matrix.name }}.sha256
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: |
            target/${{ matrix.target }}/release/cchud${{ matrix.os == 'windows-latest' && '.exe' || '' }}
            ${{ matrix.name }}.sha256

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - run: |
          # rename binaries to platform-specific names, gzip them
          for d in cchud-*; do
            BIN=$(find $d -type f -name 'cchud*' -not -name '*.sha256')
            tar czf $d.tar.gz -C $(dirname $BIN) $(basename $BIN)
          done
      - uses: softprops/action-gh-release@v2
        with:
          files: |
            *.tar.gz
            *.sha256
          generate_release_notes: true
```

### 9.2. SHA256 чексуммы и подписи

- Генерируем `.sha256` для каждого артефакта (см. workflow выше).
- Подпись через `cosign sign-blob` (sigstore) — keyless, через GitHub OIDC. Решение OQ-3 в PRD.

```yaml
      - uses: sigstore/cosign-installer@v3
      - name: Sign
        run: cosign sign-blob --yes --output-signature ${{ matrix.name }}.sig --output-certificate ${{ matrix.name }}.pem ${{ matrix.name }}
```

### 9.3. npm bootstrap-пакет

Структура npm-пакета `npm/`:
```
npm/
├── package.json
├── README.md
├── bin/
│   └── cchud.js       # тонкий launcher
├── install.js         # postinstall: качает бинарь
└── .npmignore
```

`npm/package.json`:
```json
{
  "name": "cchud",
  "version": "1.0.0",
  "description": "Fast Rust statusline for Claude Code CLI",
  "bin": { "cchud": "./bin/cchud.js" },
  "scripts": { "postinstall": "node install.js" },
  "engines": { "node": ">=14" },
  "license": "MIT",
  "homepage": "https://github.com/igorfonin/cchud",
  "repository": "github:igorfonin/cchud",
  "files": ["bin/", "install.js", "README.md"]
}
```

`npm/install.js`:
```javascript
#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');
const zlib = require('zlib');
const tar = require('tar');  // или встроенно

const VERSION = require('./package.json').version;

const TARGETS = {
  'darwin-arm64':  'cchud-aarch64-apple-darwin',
  'darwin-x64':    'cchud-x86_64-apple-darwin',
  'linux-x64':     'cchud-x86_64-unknown-linux-gnu',
  // musl детект отдельной проверкой libc
  'win32-x64':     'cchud-x86_64-pc-windows-msvc',
};

const key = `${process.platform}-${process.arch}`;
const target = TARGETS[key];
if (!target) {
  console.error(`cchud: unsupported platform ${key}`);
  process.exit(1);
}

const url = `https://github.com/igorfonin/cchud/releases/download/v${VERSION}/${target}.tar.gz`;
const sha_url = `${url}.sha256`;
// download, verify sha256, extract to ./bin/cchud
// ... ~80 строк
```

`npm/bin/cchud.js`:
```javascript
#!/usr/bin/env node
const path = require('path');
const { spawnSync } = require('child_process');
const bin = path.join(__dirname, process.platform === 'win32' ? 'cchud.exe' : 'cchud');
const result = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
```

> Размер npm tarball ~10 KB. На `npm i -g cchud@1.0.0` — postinstall скачает бинарь ~3 МБ за платформу.

### 9.4. Homebrew tap

Создать публичный repo `igorfonin/homebrew-tap` с формулой:

`Formula/cchud.rb`:
```ruby
class Cchud < Formula
  desc "Fast Rust statusline for Claude Code CLI"
  homepage "https://github.com/igorfonin/cchud"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/igorfonin/cchud/releases/download/v#{version}/cchud-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    end
    on_intel do
      url "https://github.com/igorfonin/cchud/releases/download/v#{version}/cchud-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/igorfonin/cchud/releases/download/v#{version}/cchud-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "..."
  end

  def install
    bin.install "cchud"
  end

  test do
    assert_match "cchud", shell_output("#{bin}/cchud --version")
  end
end
```

Auto-update формулы — отдельный workflow в основном repo, который на release делает PR в tap (или коммитит напрямую через PAT).

### 9.5. Install скрипт

`install.sh` в корне repo:
```bash
#!/bin/sh
set -e

REPO="igorfonin/cchud"
VERSION="${CCHUD_VERSION:-latest}"
INSTALL_DIR="${CCHUD_INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$OS-$ARCH" in
  darwin-arm64|darwin-aarch64) TARGET="cchud-aarch64-apple-darwin" ;;
  darwin-x86_64) TARGET="cchud-x86_64-apple-darwin" ;;
  linux-x86_64)
    if ldd --version 2>&1 | grep -qi musl; then
      TARGET="cchud-x86_64-unknown-linux-musl"
    else
      TARGET="cchud-x86_64-unknown-linux-gnu"
    fi
    ;;
  *) echo "cchud: unsupported $OS-$ARCH"; exit 1 ;;
esac

# Resolve version
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep -m1 tag_name | cut -d'"' -f4)
fi

URL="https://github.com/$REPO/releases/download/$VERSION/$TARGET.tar.gz"
TMP=$(mktemp -d)
curl -fsSL -o "$TMP/cchud.tar.gz" "$URL"
curl -fsSL -o "$TMP/cchud.sha256" "$URL.sha256"

# Verify
(cd "$TMP" && sha256sum -c cchud.sha256) || { echo "checksum mismatch"; exit 1; }

mkdir -p "$INSTALL_DIR"
tar xzf "$TMP/cchud.tar.gz" -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/cchud"

echo "cchud installed to $INSTALL_DIR/cchud ($VERSION)"
echo "Run: cchud install"
```

Установка одной командой:
```bash
curl -fsSL https://raw.githubusercontent.com/igorfonin/cchud/main/install.sh | sh
```

### 9.6. `cchud doctor` команда

Проверяет окружение:
```bash
cchud doctor
```

Выводит:
- Version
- Path
- Settings.json: exists/syntax-valid/cchud-section-present
- Powerline font: detected/not detected
- Color level: truecolor/256/basic
- Platform target
- Cache dir: writable
- gh auth: ok/missing (если есть GitPr виджет)

### 9.7. README финальный

- Hero: цели, цифры
- Сравнительная таблица cchud vs ccstatusline (cold-start, RAM, CPU фон, supply-chain)
- Install: 4 способа (brew/npm/curl/cargo)
- `cchud install` для автонастройки Claude Code
- Asciinema-демо TUI
- Виджеты: ссылка на `docs/widgets.md`
- Migration from ccstatusline
- License + ATTRIBUTION

### 9.8. Финальные бенчи в README

```
| Metric                  | ccstatusline (npx) | ccstatusline (global) | cchud  |
|-------------------------|--------------------|-----------------------|--------|
| Cold-start p95 (M2)     | 134 ms             | 78 ms                 | 3 ms   |
| RSS peak                | 95 MB              | 92 MB                 | 4 MB   |
| 4× sessions CPU bg      | 118%               | 54%                   | 3%     |
| Binary size             | 3 MB JS + Node     | 3 MB + Node           | 4.2 MB |
| Supply-chain (@latest)  | yes                | no                    | no     |
```

(Цифры заменить реальными после бенчей.)

### 9.9. Релиз 1.0.0

```bash
git tag v1.0.0
git push --tags
```

CI:
1. Cross-build × 5 платформ
2. Upload .tar.gz + .sha256 + (опц.) cosign-сигнатуры в Releases
3. Auto-bump формулы Homebrew (PR в `igorfonin/homebrew-tap`)
4. Publish npm `cchud@1.0.0`
5. Публикация на crates.io (опционально, для тех, кто `cargo install`)

### 9.10. Объявление

- Пост в r/ClaudeAI и r/rust
- Issue в ccstatusline upstream с упоминанием альтернативы (good-citizen)
- Twitter/X пост
- Hacker News (если уместно)

## Exit Criteria

- [ ] 5 бинарей на GitHub Releases с SHA256
- [ ] (опц.) cosign-подписи
- [ ] `npm i -g cchud@1.0.0` ставится на macOS arm/intel, Linux gnu/musl, Windows
- [ ] `brew install igorfonin/tap/cchud` работает на macOS
- [ ] `curl ... install.sh | sh` работает на macOS и Linux
- [ ] `cchud doctor` выдаёт зелёный отчёт
- [ ] README с финальными бенчами
- [ ] **AC-001 — AC-012 из PRD** все зелёные
- [ ] Релиз **1.0.0** опубликован

## Связи

- **Фаза 10** — длинный хвост (TOML, schemars, auto-update, ...)

## Риски

- **npm tarball > expected size** — debug в `.npmignore`, исключить лишнее.
- **Homebrew bottle conflict** — не делаем bottles в 1.0, only formula с download.
- **Подпись cosign — kid TXT** требуется → делать через GitHub OIDC keyless.
- **Release workflow падает на одной платформе** — артефакты остальных всё равно публикуются (`fail-fast: false` в matrix).
- **Имя `cchud` оказалось занято** на crates.io / npm — fallback на `cchud-cli` (решено в Фазе 0, но проверить дважды перед публикацией).
