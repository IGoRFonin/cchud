# Task 8 — Install integration tests

**Files:**
- Create: `tests/install.rs`

## Goal

Покрыть `cchud install` пятью integration-тестами: (1) creates fresh, (2) refuses чужой statusLine, (3) `--force` overwrites, (4) preserves unrelated keys (AC-008), (5) repeat install silently updates path. Каждый тест работает в изолированном `tempfile::TempDir` с `HOME` override; запускаются последовательно через `serial_test::serial` (между ними нельзя гонять параллельно — все мутируют `HOME` env).

## Inputs

- Tasks 1–7 закрыты: `cchud install` команда работает, smoke-тесты Step 5 в Task 7 руками подтвердили все ветки.
- `serial_test = "3"` и `tempfile = "3"` в `[dev-dependencies]` (Task 1).
- `assert_cmd = "2"` в `[dev-dependencies]` (Phase 1).

---

- [ ] **Step 1: Создать `tests/install.rs` со всеми 5 тестами**

```rust
//! Integration tests for `cchud install`.
//!
//! Each test runs in a fresh `tempfile::TempDir` with `HOME` overridden
//! to that directory. `serial_test::serial` ensures they don't race on
//! the shared `HOME` env var.
//!
//! Covers spec decision 3 (detect + --force) and PRD AC-008 (preserve
//! unrelated keys).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn settings_path(home: &TempDir) -> PathBuf {
    home.path().join(".claude/settings.json")
}

fn write_settings(home: &TempDir, content: &str) {
    let path = settings_path(home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
}

fn read_settings(home: &TempDir) -> serde_json::Value {
    let s = fs::read_to_string(settings_path(home)).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn cchud_install(home: &TempDir, extra_args: &[&str]) -> std::process::Output {
    Command::cargo_bin("cchud")
        .unwrap()
        .env("HOME", home.path())
        // On Windows, dirs::home_dir() uses USERPROFILE
        .env("USERPROFILE", home.path())
        .arg("install")
        .args(extra_args)
        .output()
        .unwrap()
}

#[test]
#[serial]
fn install_creates_settings_when_absent() {
    let home = TempDir::new().unwrap();
    // Не создаём ~/.claude — пусть install сам создаст
    let out = cchud_install(&home, &[]);
    assert!(
        out.status.success(),
        "install must succeed when no settings.json exists; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let value = read_settings(&home);
    let cmd = value
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .expect("statusLine.command must be set");
    assert!(
        cmd.contains("cchud"),
        "statusLine.command should reference cchud binary, got {cmd}"
    );
    assert_eq!(
        value.get("statusLine").and_then(|s| s.get("type")).and_then(|t| t.as_str()),
        Some("command"),
        "statusLine.type must be 'command'"
    );
}

#[test]
#[serial]
fn install_refuses_existing_non_cchud_statusline() {
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{"statusLine":{"type":"command","command":"/usr/bin/somethingelse"}}"#,
    );
    let original = fs::read_to_string(settings_path(&home)).unwrap();

    let out = cchud_install(&home, &[]);
    assert!(
        !out.status.success(),
        "install must FAIL when foreign statusLine exists without --force"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("statusLine already set to: /usr/bin/somethingelse"),
        "stderr must mention existing path; got {stderr:?}"
    );
    assert!(
        stderr.contains("--force"),
        "stderr must hint --force; got {stderr:?}"
    );

    // File must NOT have been touched
    let after = fs::read_to_string(settings_path(&home)).unwrap();
    assert_eq!(
        original, after,
        "settings.json must be untouched on refuse"
    );
}

#[test]
#[serial]
fn install_force_overwrites_existing() {
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{"statusLine":{"type":"command","command":"/usr/bin/somethingelse"}}"#,
    );
    let out = cchud_install(&home, &["--force"]);
    assert!(
        out.status.success(),
        "install --force must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value = read_settings(&home);
    let cmd = value
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap();
    assert!(
        cmd.contains("cchud"),
        "after --force, command must reference cchud, got {cmd}"
    );
    assert!(
        !cmd.contains("somethingelse"),
        "old command must be replaced, got {cmd}"
    );
}

#[test]
#[serial]
fn install_preserves_unrelated_keys() {
    // AC-008: install does not lose mcpServers, theme, customApiKeyResponses,
    // or any other top-level keys.
    let home = TempDir::new().unwrap();
    write_settings(
        &home,
        r#"{
            "theme": "dark",
            "mcpServers": {"example": {"command": "/usr/bin/foo"}},
            "customApiKeyResponses": {"approved": ["sk-test"]}
        }"#,
    );
    let out = cchud_install(&home, &[]);
    assert!(out.status.success(), "install must succeed without statusLine");
    let value = read_settings(&home);

    assert_eq!(
        value.get("theme").and_then(|t| t.as_str()),
        Some("dark"),
        "theme must be preserved"
    );
    assert!(
        value.get("mcpServers").is_some(),
        "mcpServers must be preserved"
    );
    assert_eq!(
        value
            .get("mcpServers")
            .and_then(|m| m.get("example"))
            .and_then(|e| e.get("command"))
            .and_then(|c| c.as_str()),
        Some("/usr/bin/foo"),
        "nested mcpServers content must be preserved"
    );
    assert!(
        value.get("customApiKeyResponses").is_some(),
        "customApiKeyResponses must be preserved"
    );
    assert!(
        value.get("statusLine").is_some(),
        "statusLine must be added"
    );
}

#[test]
#[serial]
fn install_repeat_silently_updates_path() {
    // First install — fresh
    let home = TempDir::new().unwrap();
    let out1 = cchud_install(&home, &[]);
    assert!(out1.status.success());
    let cmd1 = read_settings(&home)
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap()
        .to_string();
    assert!(cmd1.contains("cchud"));

    // Second install — without --force, should still succeed (cmd contains "cchud")
    let out2 = cchud_install(&home, &[]);
    assert!(
        out2.status.success(),
        "repeat install on cchud-managed statusLine must succeed without --force; stderr={}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let cmd2 = read_settings(&home)
        .get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(|c| c.as_str())
        .unwrap()
        .to_string();
    assert!(cmd2.contains("cchud"));
    // Path may be identical (same binary) or different (re-canonicalized); both OK.
}
```

**Note про `serial_test::serial`:** все 5 тестов имеют атрибут `#[serial]`. Это ставит их в один глобальный mutex group; они выполнятся последовательно даже если cargo запустит пере... wait, `serial_test::serial` работает per-process (один глобальный lock на тесты с этим атрибутом). Поскольку `cargo test` запускает тесты в нескольких threads одного процесса — `#[serial]` гарантирует non-overlap.

**Note про `HOME` + `USERPROFILE`:** `dirs::home_dir()` на Linux/macOS использует `HOME`, на Windows — `USERPROFILE`. Устанавливаем оба для cross-platform CI compatibility.

**Note про `assert_cmd::Command`:** при ошибках `unwrap()` panic'ит — это тестовый код, ОК. `#![allow(clippy::unwrap_used, clippy::expect_used)]` в начале файла снимает lint-замечания для тестов.

- [ ] **Step 2: Запустить новые тесты**

```bash
cargo test --locked --test install
```

Expected output:
```
running 5 tests
test install_creates_settings_when_absent ... ok
test install_force_overwrites_existing ... ok
test install_preserves_unrelated_keys ... ok
test install_refuses_existing_non_cchud_statusline ... ok
test install_repeat_silently_updates_path ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

Тесты идут последовательно (примерно ~1 сек каждый — каждый собирает binary и запускает 1–2 раза).

Если `install_repeat_silently_updates_path` падает — проверить что `cmd.contains("cchud")` логика в `install.rs` смотрит на `command` строку, а не имя бинаря. Если `current_exe()` на CI macOS возвращает path где НЕТ слова `cchud` (например symlink в `/var/folders/.../bin`), assertion упадёт. Решение: проверять `cmd.contains("cchud")` — наш контракт — но если CI raw-runner кладёт binary в путь без слова cchud, тест ослабить до `cmd.contains("/cchud") || cmd.contains("cchud-")`. Или canonicalize binary path в `install.rs` (Phase 9).

Если `install_refuses_existing_non_cchud_statusline` падает на assertion `original == after` — `eprintln!` через `cargo run --release` потенциально пишет в stdout вместо stderr (если stderr не redirected). Проверить что `Command::cargo_bin()` правильно различает.

- [ ] **Step 3: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0. К предыдущим тестам добавляются 5 новых install-тестов.

`cargo clippy` на тест-файле может ругаться на `clippy::unwrap_used` — мы уже добавили `#![allow(...)]` в начало файла. Если ругается на `clippy::missing_panics_doc` для `pub fn` в test'ах — false positive, нет `pub` в тестовых функциях.

- [ ] **Step 4: Verification — task-specific gate**

```bash
test -f tests/install.rs && echo "tests/install.rs ok"
grep -c '#\[serial\]' tests/install.rs
grep -c '#\[test\]' tests/install.rs
cargo test --locked --test install 2>&1 | grep -E "test result: ok\. 5 passed"
```

Expected output:
```
tests/install.rs ok
5
5
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in ...
```

- [ ] **Step 5: Commit**

```bash
git add tests/install.rs
git commit -m "test(phase-2): 5 integration tests for cchud install

Cover spec decision 3 + PRD AC-008:
- install_creates_settings_when_absent — fresh ~/ no settings.json
- install_refuses_existing_non_cchud_statusline — exit 1, file untouched
- install_force_overwrites_existing — --force replaces
- install_preserves_unrelated_keys — mcpServers/theme/customApiKeyResponses
  stay intact (AC-008)
- install_repeat_silently_updates_path — cchud command, no --force needed

All tests use tempfile::TempDir + HOME (and USERPROFILE for Windows)
override; serial_test::serial keeps them sequential despite
parallel cargo test runner.

Task 8/10 of Phase 2.
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

- [ ] `tests/install.rs` существует, содержит 5 `#[test]` функций, каждая с `#[serial]`
- [ ] `cargo test --locked --test install` зелёный (5 passed)
- [ ] AC-008 покрыт `install_preserves_unrelated_keys` (theme + mcpServers + customApiKeyResponses)
- [ ] Все тесты используют `HOME` + `USERPROFILE` env override
- [ ] `cargo clippy --locked -- -D warnings` зелёный
- [ ] Один коммит `test(phase-2): 5 integration tests for cchud install`

## Files touched

- `tests/install.rs` (created)

## Risks & rollback

- **CI Windows: `Command::cargo_bin` exec не находит binary**: `assert_cmd` поддерживает Windows; убедиться что `cchud.exe` собирается. Phase 1 task-3 CI matrix включает Windows — должно работать.
- **`HOME` env override на Windows игнорируется**: `dirs::home_dir()` на Windows использует `USERPROFILE`, не `HOME`. Мы устанавливаем оба — должно покрывать все случаи.
- **`#[serial]` mutex не перехватывает `cargo test --jobs N`**: `serial_test` использует `RwLock` per-process. Все 5 тестов в одном процессе → корректно. Если `cargo test --test install --test snapshots` запускает в раздельных процессах (по тест-крейтам) — тогда install и snapshots не блокируют друг друга, но это нормально (snapshots не использует HOME).
- **`current_exe()` на macOS CI runner returns путь без "cchud"**: если CI кладёт binary в `/private/var/folders/.../target/release/cchud` — `cmd.contains("cchud")` ОК, путь содержит filename. Если кладёт в путь типа `/runner/x` без слова cchud — fallback в Step 2.
- **Пятый тест `install_repeat_silently_updates_path` глючит на CI** (parallel temp dir collisions): `tempfile::TempDir` гарантирует unique dir per test через PID+counter — collision исключена.
- **Rollback**: `git revert HEAD` — только тестовый файл, продакшен-код не меняется.
