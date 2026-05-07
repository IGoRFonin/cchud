# Task 7 — npm templates + JS shim

**Цель:** Создать структуру `npm/` с 6 sub-packages: `cchud` (главный с JS shim) + 5× `@cchud/cli-<platform>` (per-platform native binary). package.json — templates с PLACEHOLDER версией (заполнит build script в T9). JS shim ~50 LOC, без external deps, поддерживает glibc/musl detection.

**Files:**
- Create: `npm/cchud/package.json`
- Create: `npm/cchud/bin/cchud.js`
- Create: `npm/cli-darwin-arm64/package.json`
- Create: `npm/cli-darwin-x64/package.json`
- Create: `npm/cli-linux-x64/package.json`
- Create: `npm/cli-linux-x64-musl/package.json`
- Create: `npm/cli-win32-x64/package.json`
- Create: `npm/README.md` (объяснение для contributors что такое 6-packages structure)
- Modify: `.gitignore` — добавить `npm/*/bin/cchud*` и `npm/*/node_modules/`

---

- [ ] **Step 1: `npm/cchud/package.json`**

```json
{
  "name": "cchud",
  "version": "0.0.0-PLACEHOLDER",
  "description": "Fast Rust statusline for Claude Code CLI",
  "bin": { "cchud": "bin/cchud.js" },
  "engines": { "node": ">=14" },
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  },
  "optionalDependencies": {
    "@cchud/cli-darwin-arm64": "0.0.0-PLACEHOLDER",
    "@cchud/cli-darwin-x64": "0.0.0-PLACEHOLDER",
    "@cchud/cli-linux-x64": "0.0.0-PLACEHOLDER",
    "@cchud/cli-linux-x64-musl": "0.0.0-PLACEHOLDER",
    "@cchud/cli-win32-x64": "0.0.0-PLACEHOLDER"
  }
}
```

- [ ] **Step 2: `npm/cchud/bin/cchud.js` — JS shim (full file)**

```javascript
#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');

function detectPackage() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'linux' && arch === 'x64') {
    try {
      const report = process.report.getReport();
      const glibc = report.header.glibcVersionRuntime;
      return glibc ? '@cchud/cli-linux-x64' : '@cchud/cli-linux-x64-musl';
    } catch {
      return '@cchud/cli-linux-x64';
    }
  }
  if (platform === 'darwin' && arch === 'arm64') return '@cchud/cli-darwin-arm64';
  if (platform === 'darwin' && arch === 'x64')   return '@cchud/cli-darwin-x64';
  if (platform === 'win32'  && arch === 'x64')   return '@cchud/cli-win32-x64';

  process.stderr.write(
    `cchud: unsupported platform ${platform}-${arch}.\n` +
    `Supported: darwin-arm64, darwin-x64, linux-x64 (gnu+musl), win32-x64.\n` +
    `See https://github.com/IGoRFonin/cchud/issues to request support.\n`
  );
  process.exit(1);
}

function main() {
  const pkg = process.env.CCHUD_NPM_PACKAGE || detectPackage();
  let binPath;
  try {
    const ext = process.platform === 'win32' ? '.exe' : '';
    binPath = require.resolve(`${pkg}/bin/cchud${ext}`);
  } catch (e) {
    process.stderr.write(
      `cchud: native binary package "${pkg}" not installed.\n` +
      `Try: npm install --include=optional ${pkg}@<version>\n` +
      `Or reinstall: npx --yes cchud@<version> install\n`
    );
    process.exit(1);
  }

  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: true,
  });
  if (result.error) {
    process.stderr.write(`cchud: failed to spawn ${binPath}: ${result.error.message}\n`);
    process.exit(1);
  }
  process.exit(result.status === null ? 1 : result.status);
}

main();
```

- [ ] **Step 3: 5 platform package.json**

`npm/cli-darwin-arm64/package.json`:

```json
{
  "name": "@cchud/cli-darwin-arm64",
  "version": "0.0.0-PLACEHOLDER",
  "description": "cchud native binary for darwin-arm64 (do not install directly)",
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "os": ["darwin"],
  "cpu": ["arm64"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

`npm/cli-darwin-x64/package.json`:

```json
{
  "name": "@cchud/cli-darwin-x64",
  "version": "0.0.0-PLACEHOLDER",
  "description": "cchud native binary for darwin-x64 (do not install directly)",
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "os": ["darwin"],
  "cpu": ["x64"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

`npm/cli-linux-x64/package.json`:

```json
{
  "name": "@cchud/cli-linux-x64",
  "version": "0.0.0-PLACEHOLDER",
  "description": "cchud native binary for linux-x64 (glibc) (do not install directly)",
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "os": ["linux"],
  "cpu": ["x64"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

`npm/cli-linux-x64-musl/package.json`:

```json
{
  "name": "@cchud/cli-linux-x64-musl",
  "version": "0.0.0-PLACEHOLDER",
  "description": "cchud native binary for linux-x64 (musl) (do not install directly)",
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "os": ["linux"],
  "cpu": ["x64"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

`npm/cli-win32-x64/package.json`:

```json
{
  "name": "@cchud/cli-win32-x64",
  "version": "0.0.0-PLACEHOLDER",
  "description": "cchud native binary for win32-x64 (do not install directly)",
  "license": "MIT",
  "homepage": "https://github.com/IGoRFonin/cchud",
  "repository": {
    "type": "git",
    "url": "https://github.com/IGoRFonin/cchud.git"
  },
  "files": ["bin/"],
  "os": ["win32"],
  "cpu": ["x64"],
  "publishConfig": {
    "access": "public",
    "provenance": true
  }
}
```

- [ ] **Step 4: `npm/README.md`**

```markdown
# npm/ — cchud npm package layout

This directory contains the **6 packages** that make up cchud's npm
distribution (Approach 2 / `optionalDependencies` per platform — pioneered
by Bun, swc, esbuild, rolldown, Biome).

```
npm/
├── cchud/                    # main package (JS shim, ~5 KB)
│   ├── package.json          # 0.0.0-PLACEHOLDER (real version substituted by build script)
│   └── bin/cchud.js          # spawns matching @cchud/cli-<platform>
└── cli-<platform>/           # native binary tarball
    ├── package.json          # os:[…] cpu:[…]
    └── bin/cchud[.exe]       # populated from GH artifacts during release
```

## Why 6 packages?

When users run `npx --yes cchud@1.0.0 install`, npm:

1. Fetches `cchud@1.0.0` (always).
2. Walks `optionalDependencies`, filtering by `os` + `cpu`.
3. Installs ONLY the matching `@cchud/cli-<platform>` (one package, ~5 MB).

Other platforms' packages are **skipped, not downloaded**. No `postinstall`
script means: no network call at install time, no SHA verification in JS,
works with `--ignore-scripts`, npm cache works offline.

## Local dev

`bin/cchud[.exe]` files are NOT committed (see `.gitignore`). Populated by
`scripts/build-npm-packages.sh` during `release.yml` from per-platform
GitHub artifacts.

For testing the JS shim locally:

```bash
# Drop a fixture binary that prints argv:
mkdir -p npm/cli-darwin-arm64/bin
echo -e '#!/bin/sh\necho "fixture: $@"' > npm/cli-darwin-arm64/bin/cchud
chmod +x npm/cli-darwin-arm64/bin/cchud
node npm/cchud/bin/cchud.js --version  # spawns fixture
```

## Publish flow

`release.yml::npm-publish` (Phase 9 Task 11):

1. `./scripts/build-npm-packages.sh <version>` — substitutes PLACEHOLDER → version, copies binaries from artifacts.
2. `cd npm/cli-<platform> && npm publish --provenance --tag <next|latest> --access public` × 5.
3. `cd npm/cchud && npm publish --provenance --tag <next|latest> --access public`.

The order matters: platform packages **before** main, otherwise `cchud`'s
`optionalDependencies` resolution would 404.
```

- [ ] **Step 5: Update `.gitignore`**

Прочитать существующий `.gitignore` и добавить в конец:

```gitignore
# Phase 9 — npm package binaries (populated by release.yml).
npm/*/bin/cchud
npm/*/bin/cchud.exe
npm/*/node_modules/
npm/*/package-lock.json
```

- [ ] **Step 6: Smoke shim локально с fixture binary**

```bash
mkdir -p npm/cli-darwin-arm64/bin
cat > npm/cli-darwin-arm64/bin/cchud <<'EOF'
#!/bin/sh
echo "fixture: $*"
exit 0
EOF
chmod +x npm/cli-darwin-arm64/bin/cchud

# JS shim ожидает require.resolve(<pkg>/bin/cchud) — для local smoke
# нужен симлинк в node_modules. Создадим временный.
mkdir -p npm/cchud/node_modules/@cchud
ln -sf "$(pwd)/npm/cli-darwin-arm64" npm/cchud/node_modules/@cchud/cli-darwin-arm64

# Test (нужен Node ≥14):
node npm/cchud/bin/cchud.js --version
```

Expected на macOS arm64: `fixture: --version`.

⚠ После теста очистить:

```bash
rm -rf npm/cchud/node_modules npm/cli-darwin-arm64/bin
```

(Финальные binaries будут populated CI'ем в T11.)

- [ ] **Step 7: Verify package.json валидны (jq parse)**

```bash
for f in npm/cchud/package.json npm/cli-*/package.json; do
  jq empty "$f" && echo "OK: $f"
done
```

Expected: 6× `OK:` строк.

- [ ] **Step 8: Commit**

```bash
git add npm/ .gitignore
git commit -m "$(cat <<'EOF'
feat(phase-9): T7 npm packages — 6-package layout (cchud + 5× @cchud/cli-*)

Approach 2 / optionalDependencies pattern (Bun/swc/esbuild/rolldown/Biome).
- npm/cchud/{package.json, bin/cchud.js} — JS shim ~50 LOC, no deps
- npm/cli-{darwin-arm64, darwin-x64, linux-x64, linux-x64-musl, win32-x64}/
- glibc/musl detection в shim через process.report.getReport()
- CCHUD_NPM_PACKAGE env override для отладки
- gitignore исключает binary tarballs (populated в release.yml)
EOF
)"
```
