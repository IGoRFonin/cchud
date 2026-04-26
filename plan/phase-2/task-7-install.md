# Task 7 — `cchud install` command + main args dispatch

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/install.rs`
- Modify: `src/main.rs` (полноценный args dispatch: `--version`, `--help`, `install`)

## Goal

Реализовать `cchud install` — пишет в `~/.claude/settings.json` запуск самого себя через `statusLine.command`. Обработка трёх кейсов: (1) нет существующего statusLine → пишет, (2) есть чужой statusLine → exit 1 с сообщением и подсказкой `--force`, (3) уже cchud → молча обновляет путь. Atomic write через tmp-файл + rename. Сохраняет все остальные ключи в settings.json (используем `serde_json::Value`, не `Settings`). Тесты — отдельной задачей T8.

## Inputs

- Tasks 1–6 закрыты: walking skeleton + config layer работают.
- `serde_json` в `[dependencies]`, поддерживает `Value`/`json!`/`to_string_pretty`.
- `dirs` в `[dependencies]` — для `home_dir()`.

---

- [ ] **Step 1: Создать `src/commands/mod.rs`**

```rust
//! Subcommands: cchud install (Task 7), cchud configure/import/doctor (later phases).

pub mod install;
```

- [ ] **Step 2: Создать `src/commands/install.rs`**

```rust
//! `cchud install` — wires cchud into ~/.claude/settings.json statusLine.command.
//!
//! Behavior (matches spec decision 3):
//! - No existing statusLine → write our path, exit 0
//! - Existing statusLine NOT containing "cchud" + no --force → exit 1
//! - Existing statusLine NOT containing "cchud" + --force → overwrite
//! - Existing statusLine ALREADY containing "cchud" → silently update path
//!
//! Preserves all other keys in settings.json (mcpServers, theme, etc.) by
//! operating on `serde_json::Value` instead of typed `Settings`.
//!
//! Atomic write: writes to settings.json.tmp, then renames. Race-safe.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    let force = args.iter().any(|a| a == "--force");
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: cannot determine executable path: {e}");
            return ExitCode::from(1);
        }
    };
    let path = settings_path();

    let mut root = read_or_empty(&path);

    if let Some(cmd) = root
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(serde_json::Value::as_str)
    {
        if !cmd.contains("cchud") && !force {
            eprintln!("cchud: statusLine already set to: {cmd}");
            eprintln!("       use --force to overwrite, or remove it manually first.");
            return ExitCode::from(1);
        }
    }

    root["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "padding": 0,
    });

    if let Err(e) = write_atomic(&path, &root) {
        eprintln!("cchud: cannot write {}: {e}", path.display());
        return ExitCode::from(1);
    }
    println!("cchud installed: {}", exe.display());
    ExitCode::SUCCESS
}

fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/settings.json")
}

fn read_or_empty(path: &Path) -> serde_json::Value {
    let Ok(s) = std::fs::read_to_string(path) else {
        return serde_json::json!({});
    };
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({}))
}

fn write_atomic(path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let pretty = serde_json::to_string_pretty(value)?;
    std::fs::write(&tmp, pretty)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

**Note про `unwrap_used`/`expect_used`:** в spec lints для `commands/install.rs` оставлены `warn` (не `deny`) — этот файл оперирует на FS и имеет legitimate `expect`/`unwrap_or_else`. Не пишем `#![deny(...)]` в начало этого файла.

**Note про atomic rename:** `std::fs::rename` атомарен на одном filesystem (Unix POSIX, NTFS). Если `~/.claude/` и `/tmp` на разных томах — fail. Решение: `tmp = path.with_extension("json.tmp")` гарантирует тот же родительский каталог, что и `path` → одноместный rename.

**Note про `read_or_empty`:** глотает ошибку парсинга и возвращает `{}`. Это ОК для install (хуже не сделаем — пишем заново). config::load в Task 6 наоборот пишет warning. Несовместимость намеренная: install — это recovery action, мы знаем что собрались переписать.

- [ ] **Step 3: Подключить `mod commands;` в `src/main.rs`**

Добавить в список модулей:
```rust
mod commands;
mod config;
mod render;
mod types;
mod widgets;
```

- [ ] **Step 4: Полноценный args dispatch в `src/main.rs::main`**

Заменить текущий `main`:

```rust
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("install") => commands::install::run(&args[1..]),
        Some(other) if other.starts_with("--") => {
            eprintln!("cchud: unknown flag: {other}");
            eprintln!("       run 'cchud --help' for usage");
            ExitCode::from(2)
        }
        _ => render_pipeline(),
    }
}

fn print_help() {
    println!("cchud {} — fast Rust statusline for Claude Code", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("  cchud                  read JSON payload from stdin, render statusline");
    println!("  cchud install          wire cchud into ~/.claude/settings.json");
    println!("  cchud install --force  overwrite existing statusLine");
    println!("  cchud --version        print version");
    println!("  cchud --help           print this help");
}
```

**Note про `--unknown-flag` обработку:** возвращаем exit 2 (стандарт CLI: 1 = ошибка из вызванной операции, 2 = ошибка вызова). Нет `panic`, есть guidance.

- [ ] **Step 5: Smoke test — все 4 ветки CLI**

```bash
cargo build --release --locked

# Ветка --version
./target/release/cchud --version
# Expected: "0.0.1" (или текущая версия из Cargo.toml)

# Ветка --help
./target/release/cchud --help
# Expected: USAGE блок

# Ветка --unknown
./target/release/cchud --foobar
echo "exit: $?"
# Expected: stderr "cchud: unknown flag: --foobar" + exit 2

# Ветка install (не вызывая в реальный HOME!) — используем temp HOME
HOME=/tmp/cchud-install-test mkdir -p /tmp/cchud-install-test/.claude
HOME=/tmp/cchud-install-test ./target/release/cchud install
echo "exit: $?"
cat /tmp/cchud-install-test/.claude/settings.json
# Expected: stdout "cchud installed: /...path..." + exit 0
# settings.json содержит {"statusLine": {"type": "command", "command": "/...cchud", "padding": 0}}

# Ветка install с существующим чужим statusLine → exit 1
echo '{"statusLine":{"type":"command","command":"/usr/bin/somethingelse"},"theme":"dark"}' > /tmp/cchud-install-test/.claude/settings.json
HOME=/tmp/cchud-install-test ./target/release/cchud install
echo "exit: $?"
# Expected: stderr "cchud: statusLine already set to: /usr/bin/somethingelse" + exit 1

# Ветка install --force → overwrite
HOME=/tmp/cchud-install-test ./target/release/cchud install --force
echo "exit: $?"
cat /tmp/cchud-install-test/.claude/settings.json
# Expected: exit 0, settings.json содержит наш cchud путь, "theme":"dark" сохранён

# Ветка install по уже cchud-установке (без --force) → silent update
HOME=/tmp/cchud-install-test ./target/release/cchud install
echo "exit: $?"
# Expected: exit 0 (cchud в command содержится → молча обновляем путь)

# Cleanup
rm -rf /tmp/cchud-install-test
```

- [ ] **Step 6: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. Никаких новых тестов (они в Task 8); существующие 9+ тестов зелёные.

- [ ] **Step 7: Verification — task-specific gate**

```bash
test -f src/commands/mod.rs && echo "commands/mod.rs ok"
test -f src/commands/install.rs && echo "commands/install.rs ok"
grep -q 'pub fn run' src/commands/install.rs && echo "install::run ok"
grep -q 'fn write_atomic' src/commands/install.rs && echo "atomic write ok"
grep -q '"--version"' src/main.rs && echo "version dispatch ok"
grep -q '"install"' src/main.rs && echo "install dispatch ok"
grep -q 'fn print_help' src/main.rs && echo "help fn ok"
./target/release/cchud --version
```

Expected: все строки `ok` + версия из `Cargo.toml` (например `0.0.1`).

- [ ] **Step 8: Commit**

```bash
git add src/commands/mod.rs src/commands/install.rs src/main.rs
git commit -m "feat(phase-2): cchud install + args dispatch

install command writes ~/.claude/settings.json statusLine.command:
- no existing statusLine → write, exit 0
- existing non-cchud statusLine → exit 1 + diagnostic + --force hint
- existing non-cchud + --force → overwrite
- existing cchud (repeat install) → silently update path

Operates on serde_json::Value (not typed Settings) to preserve all
unrelated keys (mcpServers, theme, etc.). Atomic write via tmp file
+ rename — race-safe on same filesystem.

main.rs gains full args dispatch:
  --version   print Cargo.toml version
  --help|-h   USAGE block
  install     subcommand with positional args (--force flag)
  unknown --  exit 2 with diagnostic
  default     render pipeline (stdin → widgets → stdout)

Tests in Task 8.

Task 7/10 of Phase 2.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/commands/mod.rs` существует, экспортирует `pub mod install;`
- [ ] `src/commands/install.rs` содержит `pub fn run(args: &[String]) -> ExitCode`
- [ ] Atomic write через `with_extension("json.tmp")` + `rename`
- [ ] Все error-ветки печатают `cchud:` префикс на stderr и возвращают `ExitCode::from(1)` или `ExitCode::from(2)`
- [ ] `src/main.rs` обрабатывает: `--version`, `--help`/`-h`, `install`, unknown flags, default render
- [ ] Smoke test (Step 5) проходит все 6 веток (version, help, unknown, install fresh, install refuses, install --force, install repeat)
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] Один коммит `feat(phase-2): cchud install + args dispatch`

## Files touched

- `src/commands/mod.rs` (created)
- `src/commands/install.rs` (created)
- `src/main.rs` (modified — args dispatch, print_help, mod commands)

## Risks & rollback

- **`current_exe()` возвращает не absolute path**: документация говорит obtain canonical path. Если на каких-то ОС возвращает relative — `current_exe()?.canonicalize()?` дал бы абсолют, но canonicalize резолвит symlinks (что нежелательно для homebrew installs). Оставляем `current_exe()` как есть; canonical Symlinks — Phase 9.
- **`std::fs::rename` cross-fs error (EXDEV)**: гарантировано не происходит — tmp создаётся в том же directory, что target.
- **macOS quarantine xattr добавляет задержку первого запуска**: README уже содержит инструкцию `xattr -d com.apple.quarantine` (Phase 1 docs).
- **`~/.claude/` directory не существует**: `write_atomic` вызывает `create_dir_all(parent)` — создаст рекурсивно. Если permission denied — exit 1 с diagnostic.
- **`--force` после имени команды vs до**: `args.iter().any(|a| a == "--force")` ищет где угодно в `args[1..]`. Это правильно — пользователь может ввести `cchud install --force` или `cchud install --force somethingelse`. Phase 9 формализует.
- **Smoke test temp HOME оставлен**: cleanup в самом конце. При прерывании Ctrl-C — выполни `rm -rf /tmp/cchud-install-test` вручную.
- **Rollback**: `git revert HEAD` — изолировано в одном коммите.
