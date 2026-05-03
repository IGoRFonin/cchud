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
