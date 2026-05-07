# Task 3 — `cchud install` self-relocation integration

**Цель:** Расширить `pub fn run` в `src/commands/install.rs`: после parsing `--force` / `--no-relocate` flags выполнять self-relocation через `canonical_target_path` + `relocate_to`, затем wire `~/.claude/settings.json` (теперь с `final_exe`, не `current_exe`), затем PATH check. Идемпотентен (повторный run no-op).

**Files:**
- Modify: `src/commands/install.rs::run` — расширить алгоритм.
- Modify: `tests/install_relocate.rs` — добавить ≥3 кейсов высокого уровня (`run` с `CCHUD_SETTINGS` env override на tempdir).

---

- [ ] **Step 1: Написать failing tests (TDD) — добавить в конец `tests/install_relocate.rs`**

```rust
// ---------- High-level integration tests for `commands::install::run` ----------
//
// Используем CCHUD_SETTINGS env var (existing escape hatch в settings_path())
// чтобы redirect ~/.claude/settings.json в tempdir.
// Используем --no-relocate чтобы skip копирование (избежать write в $HOME/.local/bin).

#[test]
fn run_no_relocate_writes_settings_and_returns_zero() {
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");

    // Установить env var — temporary, restored by Drop.
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    let args = vec!["--no-relocate".to_string()];
    let exit = cchud::commands::install::run(&args);
    assert_eq!(format!("{exit:?}"), format!("{:?}", std::process::ExitCode::SUCCESS));

    let body = fs::read_to_string(&settings_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let cmd = v.get("statusLine").unwrap().get("command").unwrap().as_str().unwrap();
    assert!(cmd.contains("cchud") || cmd.contains("test_runner"), "cmd: {cmd}");
}

#[test]
fn run_idempotent_second_call_is_no_op_logically() {
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    let args = vec!["--no-relocate".to_string()];
    cchud::commands::install::run(&args);
    let first = fs::read_to_string(&settings_path).unwrap();
    cchud::commands::install::run(&args);
    let second = fs::read_to_string(&settings_path).unwrap();
    assert_eq!(first, second, "second run should produce identical settings.json");
}

#[test]
fn run_force_overwrites_non_cchud_status_line() {
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    fs::write(
        &settings_path,
        r#"{"statusLine":{"type":"command","command":"ccstatusline"}}"#,
    )
    .unwrap();

    // Без --force должен fail.
    let exit_no_force = cchud::commands::install::run(&["--no-relocate".to_string()]);
    assert_eq!(format!("{exit_no_force:?}"), format!("{:?}", std::process::ExitCode::from(1)));

    // С --force должен пройти.
    let exit_force = cchud::commands::install::run(
        &["--force".to_string(), "--no-relocate".to_string()],
    );
    assert_eq!(format!("{exit_force:?}"), format!("{:?}", std::process::ExitCode::SUCCESS));

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let cmd = v.get("statusLine").unwrap().get("command").unwrap().as_str().unwrap();
    assert!(!cmd.contains("ccstatusline"));
}

// Tiny RAII helper для env vars в tests.
struct EnvVarGuard {
    key: &'static str,
    prev: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set<P: AsRef<Path>>(key: &'static str, val: P) -> Self {
        let prev = std::env::var_os(key);
        std::env::set_var(key, val.as_ref());
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}
```

⚠ **Note:** `std::env::set_var` is unsafe in newer Rust editions. Если crate использует Rust 2024 + `unsafe-set-var` lint — обернуть `unsafe { … }`. Иначе оставить как есть.

- [ ] **Step 2: Run tests — confirm они FAIL**

```bash
cargo test --locked --test install_relocate
```

Expected: новые tests fail — `run` ещё не имеет `--no-relocate` обработки.

- [ ] **Step 3: Refactor `pub fn run` в `src/commands/install.rs`**

Заменить тело `run` на:

```rust
pub fn run(args: &[String]) -> ExitCode {
    let force = args.iter().any(|a| a == "--force");
    let no_relocate = args.iter().any(|a| a == "--no-relocate");

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: cannot determine executable path: {e}");
            return ExitCode::from(1);
        }
    };

    // Step 1: Self-relocate (если не --no-relocate).
    let final_exe = if no_relocate {
        current_exe.clone()
    } else {
        let target = match canonical_target_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("cchud install: cannot resolve target path: {e}");
                return ExitCode::from(1);
            }
        };
        if same_file(&current_exe, &target).unwrap_or(false) {
            // Already at target — no-op.
        } else if let Err(e) = relocate_to(&current_exe, &target) {
            eprintln!("cchud install: relocation failed: {e}");
            return ExitCode::from(1);
        } else {
            println!("cchud: binary installed to {}", target.display());
        }
        target
    };

    // Step 2: Wire ~/.claude/settings.json.
    if let Err(e) = write_settings_with_exe(&final_exe, force) {
        eprintln!("cchud install: cannot write settings.json: {e}");
        return ExitCode::from(1);
    }
    println!("cchud: wired into Claude Code");

    // Step 3: PATH check (warning не валит exit).
    check_path_or_warn(&final_exe);

    ExitCode::SUCCESS
}

fn write_settings_with_exe(exe: &Path, force: bool) -> std::io::Result<()> {
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
            return Err(std::io::Error::other("statusLine occupied"));
        }
    }

    root["statusLine"] = serde_json::json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "padding": 0,
    });

    write_atomic(&path, &root)
}
```

(Старая копия логики settings — теперь живёт только в `write_settings_with_exe`. Backup section добавляется в Task 4. `write_atomic` остаётся как есть.)

- [ ] **Step 4: Update `--help` text в main.rs**

Прочитать `src/main.rs::print_help` и добавить строку для `--no-relocate`:

```rust
println!("  cchud install               wire cchud + self-relocate to ~/.local/bin/");
println!("  cchud install --force       overwrite existing statusLine");
println!("  cchud install --no-relocate skip self-copy (dev-only)");
```

(Заменить существующие строки `install` / `install --force`.)

- [ ] **Step 5: Run tests**

```bash
cargo test --locked --test install_relocate
```

Expected: ВСЕ tests PASS (12 helpers + 3 high-level).

- [ ] **Step 6: Manual smoke**

```bash
cargo build --release --locked
mkdir -p /tmp/cchud-test && cd /tmp/cchud-test
CCHUD_SETTINGS=/tmp/cchud-test/settings.json /Users/igor/mp/startup/cchud/target/release/cchud install --no-relocate
cat /tmp/cchud-test/settings.json
```

Expected: `statusLine.command` указывает на `target/release/cchud` (т.к. `--no-relocate`).

```bash
rm /tmp/cchud-test/settings.json
CCHUD_SETTINGS=/tmp/cchud-test/settings.json /Users/igor/mp/startup/cchud/target/release/cchud install
ls -la ~/.local/bin/cchud
cat /tmp/cchud-test/settings.json
```

Expected: binary скопирован в `~/.local/bin/cchud` (mode 0755), settings.json указывает туда.

⚠ После manual smoke очистить: `rm ~/.local/bin/cchud /tmp/cchud-test/settings.json` (если эта машина не должна держать тестовый install).

- [ ] **Step 7: Run полный test suite**

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Expected: всё зелёное.

- [ ] **Step 8: Commit**

```bash
git add src/commands/install.rs src/main.rs tests/install_relocate.rs
git commit -m "$(cat <<'EOF'
feat(phase-9): T3 install self-relocation — copy to ~/.local/bin/cchud + PATH check

cchud install теперь:
1. Копирует current_exe в ~/.local/bin/cchud (Unix) или %LOCALAPPDATA%\cchud\cchud.exe (Win)
2. Wires final_exe (а не current_exe) в ~/.claude/settings.json
3. Печатает PATH warning с shell-specific hint (zsh/bash/fish)

Идемпотентен. --no-relocate skip копирование (для разработки).
3 integration tests + smoke (CCHUD_SETTINGS override).
EOF
)"
```
