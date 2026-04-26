# Task 1 — Bootstrap (Cargo + toolchain + .gitignore)

**Files:**
- Create: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/main.rs` (auto-generated hello-world, replaced in Task 2), `.gitignore`, `.git/`
- Modify: `plan/phase-1-init.md` (R3 — fix регистр имени)

## Goal

Создать Cargo-проект на MSRV 1.85, объявить полный набор зависимостей (даже не используемых в Phase 1 — чтобы протестировать резолв на MSRV), настроить `.gitignore`, починить регистр имени в исходном intent-доке.

## Inputs

- Текущая директория `/Users/igor/mp/startup/cchud` содержит `plan/`, `docs/`, `benches/`, `ccstatusline-research.md`. Нет `src/`, `Cargo.toml`, `.git/`.
- Локальный `rustc --version` ≥ 1.85.
- Соединение с интернетом (cargo build тянет crates).

---

- [ ] **Step 1: Запустить `cargo init`**

```bash
cargo init --name cchud --bin
```

Expected output: `Creating binary (application) package` + сообщение что git initialized. Создаёт: `Cargo.toml` (с дефолтным content), `src/main.rs` (hello-world), `.gitignore` (содержит `/target` + `Cargo.lock` для bin... wait, `Cargo.lock` для bin НЕ в .gitignore). Запускает `git init`.

- [ ] **Step 2: Проверить что репозиторий создан**

```bash
git status
```

Expected: `On branch main` (или `master` — зависит от глобального git config). Видно untracked: `.gitignore`, `Cargo.toml`, `src/main.rs`. Existing files (`plan/`, `docs/`, `benches/`, `ccstatusline-research.md`) тоже untracked.

- [ ] **Step 3: Заменить сгенерированный `Cargo.toml` нашим**

Перезаписать `Cargo.toml` следующим содержимым:

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

- [ ] **Step 4: Создать `rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.85"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

Note: rust-toolchain.toml читается только rustup. Локально (если rustc через brew) файл инертен — никакого вреда. CI использует `dtolnay/rust-toolchain@stable` (см. Task 3) — тоже не читает этот файл, использует stable. Файл — документация intent + работает для разработчиков с rustup.

- [ ] **Step 5: Расширить `.gitignore` (O1)**

`cargo init` создал минимальный `.gitignore` (`/target`). Дополнить — добавить эти строки в конец файла:

```
# Bench artifacts
bench-results.md

# Editor / OS
*.swp
*.swo
.DS_Store
.idea/
.vscode/

# Local env
.env
.env.local
```

- [ ] **Step 6: Сгенерировать `Cargo.lock` и первый build**

```bash
cargo build --release
```

Expected: тянет crates (serde, serde_json, lexopt, dirs, anstyle, anstream, nu-ansi-term, unicode-width, unicode-segmentation, terminal_size, supports-color + transitive). Билдит hello-world `main.rs` в release. Создаётся `Cargo.lock` и `target/`. Итоговый бинарь: `target/release/cchud`. Build time ~30–90 сек на холодную.

Если падает — проверить что rustc поддерживает edition 2024 (`rustc --version` ≥ 1.85). Если deps не резолвятся на MSRV — fallback: поднять `rust-version` в Cargo.toml до 1.87 и обновить `rust-toolchain.toml` channel="1.87", записать в DECISIONS.

- [ ] **Step 7: Проверить standard gate (build с `--locked` + clippy + fmt)**

```bash
cargo build --release --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все три команды exit 0. `cargo build --release --locked` — no-op после step 6. `cargo clippy` на hello-world `main.rs` не должен ругаться (тривиальный код). `cargo fmt --check` — auto-generated main.rs уже в каноничном формате.

Если clippy ругается на pedantic warnings в hello-world (маловероятно, но возможно `print_with_newline` или подобное) — заменить тело main.rs на:
```rust
fn main() {}
```
Чтобы пройти гейт. Реальный skeleton кладётся в Task 2.

- [ ] **Step 8: Починить регистр имени в `plan/phase-1-init.md` (R3)**

Заменить все вхождения `igorfonin/cchud` → `IGoRFonin/cchud`. 3 строки: 75 (Cargo.toml пример), 247 (`gh repo create`), 260 (exit-criteria). Также строка 247 устарела по содержанию (`--public` → теперь `--private` per Task 5), но это правит Task 5 — здесь только регистр.

Используй Edit tool с `replace_all: true`:
- `old_string`: `igorfonin/cchud`
- `new_string`: `IGoRFonin/cchud`
- `file_path`: `/Users/igor/mp/startup/cchud/plan/phase-1-init.md`

Verify: `grep -c 'IGoRFonin/cchud' plan/phase-1-init.md` → 3, `grep -c 'igorfonin/cchud' plan/phase-1-init.md` → 0.

- [ ] **Step 9: Verification — task-specific gate**

```bash
cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="cchud") | .name'
grep -q 'channel = "1.85"' rust-toolchain.toml && echo "rust-toolchain ok"
grep -q 'bench-results.md' .gitignore && echo "gitignore ok"
grep -q 'IGoRFonin/cchud' Cargo.toml && echo "Cargo.toml repo ok"
```

Expected output:
```
cchud
rust-toolchain ok
gitignore ok
Cargo.toml repo ok
```

Если `jq` не установлен — заменить первой командой на `cargo metadata --format-version 1 --no-deps | grep -o '"name":"cchud"' | head -1`.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml .gitignore src/main.rs plan/phase-1-init.md
git commit -m "chore(phase-1): cargo bootstrap with deps, toolchain, gitignore

- cargo init --name cchud --bin
- declare full dependency set (verifies MSRV 1.85 resolves)
- rust-toolchain.toml pin to 1.85 (rustup-respected, inert otherwise)
- .gitignore: extend with bench-results.md, editor/OS files
- fix repo name case in plan/phase-1-init.md (igorfonin → IGoRFonin)

Task 1/5 of Phase 1.
"
```

Verify: `git log --oneline | head -1` показывает коммит.

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

(`cargo test` no-op — нет тестов до Task 2; должен пройти за 0с.)

## Definition of Done

- [ ] `cargo build --release --locked` зелёный
- [ ] `cargo clippy --locked -- -D warnings` без warning
- [ ] `cargo fmt --check` зелёный
- [ ] `Cargo.lock` создан и закоммичен
- [ ] `Cargo.toml` содержит `repository = "https://github.com/IGoRFonin/cchud"` и MSRV 1.85
- [ ] `.gitignore` содержит `bench-results.md` и `.DS_Store`
- [ ] `plan/phase-1-init.md` не содержит `igorfonin/cchud` (lowercase)
- [ ] Один коммит с префиксом `chore(phase-1): cargo bootstrap`

## Files touched

- `Cargo.toml` (created)
- `Cargo.lock` (auto-generated)
- `rust-toolchain.toml` (created)
- `.gitignore` (created by cargo init, extended)
- `src/main.rs` (auto-generated hello-world, replaced in Task 2)
- `plan/phase-1-init.md` (modified — case fix)
- `.git/` (initialized by cargo init)

## Risks & rollback

- **Cargo init refuses non-empty dir:** маловероятно (cargo init работает в директории с другими файлами, проверяет только отсутствие Cargo.toml). Если упадёт — `cargo init --bin --name cchud --vcs git .` с явным path.
- **MSRV 1.85 не подтягивает deps:** см. Step 6 fallback.
- **Pedantic ругается на hello-world:** см. Step 7 fallback (заменить на `fn main() {}`).
- **Rollback:** `rm -rf .git Cargo.toml Cargo.lock rust-toolchain.toml src/ target/` и снять правки `.gitignore` и `plan/phase-1-init.md` вручную (или через `git checkout` если коммит уже есть).
