# Фаза 1 — Инициализация репозитория

**Длительность:** 1–2 часа
**Входные условия:** Фаза 0 (payload-семплы, baseline)
**Релиз:** — (но push первого коммита в public repo)

## Цель

Скелет проекта: `cargo new`, базовый Cargo.toml, CI на 3 платформах, snapshot-тесты на payload'ах из Фазы 0. Бинарь печатает заглушку.

## Шаги

### 1.1. Cargo init

```bash
cd /Users/igor/mp/startup/cchud
cargo init --name cchud --bin
```

### 1.2. Структура

```
cchud/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .github/
│   └── workflows/
│       ├── ci.yml          # build/test/clippy/fmt × 3 платформы
│       ├── bench.yml       # hyperfine на каждый PR
│       └── release.yml     # тэг → cross-build → GH Releases (фаза 9)
├── benches/
│   ├── samples/            # из фазы 0
│   ├── baseline.md         # из фазы 0
│   └── run.sh              # hyperfine скрипт
├── src/
│   ├── main.rs             # точка входа
│   ├── lib.rs
│   ├── types/
│   │   └── mod.rs          # пустой пока
│   ├── widgets/
│   │   ├── mod.rs          # Widget trait + registry
│   │   └── stub.rs         # заглушка
│   ├── render/
│   │   └── mod.rs
│   ├── config/
│   │   └── mod.rs
│   └── tui/
│       └── mod.rs          # feature-gated
├── tests/
│   └── snapshots.rs        # insta-тесты
├── docs/
│   ├── prd-cchud.md
│   ├── upstream-map.md
│   ├── widgets.md
│   └── DECISIONS.md
├── plan/                   # этот документ
├── LICENSE                 # MIT
├── README.md
├── ATTRIBUTION.md          # упоминание ccstatusline upstream
└── CHANGELOG.md
```

### 1.3. Cargo.toml

```toml
[package]
name = "cchud"
version = "0.0.1"
edition = "2024"
rust-version = "1.85"
authors = ["Igor Fonin <menotoa1@gmail.com>"]
license = "MIT"
description = "Fast Rust statusline for Claude Code CLI"
repository = "https://github.com/IGoRFonin/cchud"
keywords = ["claude", "claude-code", "statusline", "cli", "terminal"]
categories = ["command-line-utilities"]
readme = "README.md"

[dependencies]
# Hot-path
serde = { version = "1", features = ["derive"] }
serde_json = "1"
lexopt = "0.3"
dirs = "6"

# Terminal
anstyle = "1"
anstream = "0.6"
nu-ansi-term = "0.50"
unicode-width = "0.2"
unicode-segmentation = "1"
terminal_size = "0.4"
supports-color = "3"

# Lazy для виджетов (добавятся в фазах 3-7)
# sonic-rs, gix, ureq, bincode, ratatui, crossterm — позже

[dev-dependencies]
insta = { version = "1", features = ["json"] }
assert_cmd = "2"
predicates = "3"

[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"
opt-level = 3

[profile.bench]
inherits = "release"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
```

### 1.4. rust-toolchain.toml

```toml
[toolchain]
channel = "1.85"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

### 1.5. CI: `.github/workflows/ci.yml`

Matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`.

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --locked
      - run: cargo test --locked
      - run: cargo clippy --locked -- -D warnings
      - run: cargo fmt --check
```

### 1.6. Bench workflow: `.github/workflows/bench.yml`

```yaml
name: Bench
on: pull_request
jobs:
  hyperfine:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --locked
      - run: brew install hyperfine
      - run: bash benches/run.sh > bench-results.md
      - uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const body = fs.readFileSync('bench-results.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## hyperfine\n\n' + body
            });
```

### 1.7. Bench-скрипт: `benches/run.sh`

```bash
#!/bin/bash
set -e
SAMPLES=$(ls benches/samples/*.json | head -3)
for SAMPLE in $SAMPLES; do
  echo "### $(basename $SAMPLE)"
  hyperfine --warmup 20 --runs 200 \
    "cat $SAMPLE | ./target/release/cchud" \
    --export-markdown -
done
```

### 1.8. Skeleton main.rs

```rust
use std::io::Read;

fn main() -> std::io::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    println!("cchud (skeleton) | input bytes: {}", input.len());
    Ok(())
}
```

### 1.9. Snapshot-тесты на skeleton

`tests/snapshots.rs`:
```rust
use assert_cmd::Command;
use std::fs;

#[test]
fn renders_skeleton_for_each_sample() {
    let samples = fs::read_dir("benches/samples").unwrap();
    for entry in samples {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let payload = fs::read_to_string(&path).unwrap();
            let output = Command::cargo_bin("cchud").unwrap()
                .write_stdin(payload)
                .output().unwrap();
            assert!(output.status.success());
            assert!(!output.stdout.is_empty());
        }
    }
}
```

В Фазе 2 заменим на `insta::assert_snapshot!`.

### 1.10. README + LICENSE + ATTRIBUTION

- `LICENSE` — стандартный MIT с твоим именем и годом 2026
- `README.md` — placeholder с целью, статусом "WIP", ссылкой на upstream ccstatusline и PRD
- `ATTRIBUTION.md` — явная благодарность Matthew Breedlove (@sirmalloc) и ccstatusline

### 1.11. Первый коммит и push

```bash
git init
git add .
git commit -m "chore: initial commit (skeleton + CI + PRD + plan)"
gh repo create IGoRFonin/cchud --public --source=. --push
```

Проверить, что CI зелёный на всех 3 платформах.

## Exit Criteria

- [ ] `cargo build --release` собирается на macOS/Linux/Windows
- [ ] `cargo test` проходит
- [ ] `cargo clippy -- -D warnings` без warning'ов
- [ ] `cargo fmt --check` проходит
- [ ] `./target/release/cchud < benches/samples/payload-001.json` печатает skeleton-строку
- [ ] CI зелёный в GitHub Actions
- [ ] Repo `IGoRFonin/cchud` опубликован на GitHub
- [ ] `cchud@0.0.1` зарезервирован на crates.io? **НЕТ** — пока не публикуем, чтобы не залочить имя на пустышку

## Связи

- **Фаза 0** даёт payload-семплы для тестов
- **Фаза 2** заменит skeleton на pipeline и реальные виджеты
- **Фаза 9** расширит release.yml для cross-build на 5 платформ

## Риски

- Cargo dependency resolution на старом MSRV — pin к 1.85, проверить.
- `nu-ansi-term` 0.50 совместим с anstyle 1.x (anstyle-nu-ansi-term есть как мост).
- Windows CI с проблемами длинных путей — `git config --system core.longpaths true`, документировать в README troubleshooting.
