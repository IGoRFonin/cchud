# Task 4 — settings.json backup `<path>.bak.<unix-ms>`

**Цель:** Перед write `~/.claude/settings.json` копировать существующий файл в `<path>.bak.<unix-ms>` (best-effort, не fatal). Reuse паттерна из `src/tui/save.rs::backup_path` (Phase 8). Тест проверяет, что backup создаётся и содержит prev content.

**Files:**
- Modify: `src/commands/install.rs::write_settings_with_exe` — добавить backup перед mutation.
- Modify: `tests/install_relocate.rs` — добавить ≥2 кейсов backup.

---

- [ ] **Step 1: Failing tests первыми**

В конец `tests/install_relocate.rs` добавить:

```rust
#[test]
fn run_creates_bak_file_when_settings_already_exists() {
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    fs::write(
        &settings_path,
        r#"{"statusLine":{"type":"command","command":"cchud-old"}}"#,
    )
    .unwrap();

    let exit = cchud::commands::install::run(&["--no-relocate".to_string()]);
    assert_eq!(format!("{exit:?}"), format!("{:?}", std::process::ExitCode::SUCCESS));

    // Должен существовать хотя бы один файл с prefix settings.json.bak.
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("settings.json.bak."))
        .collect();
    assert!(!entries.is_empty(), "expected at least one backup, got: {entries:?}");

    let bak_name = entries.first().unwrap();
    let bak_body = fs::read_to_string(dir.path().join(bak_name)).unwrap();
    assert!(bak_body.contains("cchud-old"), "backup should contain prev content; got: {bak_body}");
}

#[test]
fn run_no_backup_when_settings_did_not_exist() {
    let dir = tempdir().unwrap();
    let settings_path = dir.path().join("settings.json");
    let _guard = EnvVarGuard::set("CCHUD_SETTINGS", &settings_path);

    cchud::commands::install::run(&["--no-relocate".to_string()]);

    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(".bak."))
        .collect();
    assert!(entries.is_empty(), "no backup expected if file did not exist");
}
```

- [ ] **Step 2: Run tests — confirm они FAIL**

```bash
cargo test --locked --test install_relocate run_creates_bak_file_when_settings_already_exists
```

Expected: FAIL (backup ещё не создаётся).

- [ ] **Step 3: Реализовать backup в `write_settings_with_exe`**

Найти функцию `write_settings_with_exe` (созданную в T3) и добавить backup-логику ПОСЛЕ строки `let mut root = read_or_empty(&path);`:

```rust
fn write_settings_with_exe(exe: &Path, force: bool) -> std::io::Result<()> {
    let path = settings_path();
    let mut root = read_or_empty(&path);

    // Phase 9 Task 4: backup перед write (best-effort).
    if path.exists() {
        let unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let mut bak = path.as_os_str().to_owned();
        bak.push(format!(".bak.{unix_ms}"));
        let bak_path = std::path::PathBuf::from(bak);
        let _ = std::fs::copy(&path, &bak_path); // best-effort
    }

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

(Использует тот же pattern, что и `tui::save::backup_path` — `as_os_str().to_owned()` + `.push(format!(".bak.{ts}"))` гарантирует, что мы не заменяем расширение, а добавляем суффикс.)

- [ ] **Step 4: Run tests — confirm они PASS**

```bash
cargo test --locked --test install_relocate
```

Expected: ВСЕ tests PASS, включая два новых backup-теста.

- [ ] **Step 5: Polish — ensure backup pattern matches Phase 8**

Проверить, что test от Phase 8 `tui::save::backup_path_uses_unix_ms_suffix` всё ещё PASS:

```bash
cargo test --locked --test install_relocate
cargo test --locked tui::save
```

Expected: оба зелёные.

- [ ] **Step 6: Run полный test suite**

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Expected: всё зелёное.

- [ ] **Step 7: Commit**

```bash
git add src/commands/install.rs tests/install_relocate.rs
git commit -m "$(cat <<'EOF'
feat(phase-9): T4 settings.json backup — <path>.bak.<unix-ms>

Перед write ~/.claude/settings.json создаётся backup (best-effort).
Reuse паттерна из Phase 8 tui::save (ts из SystemTime::UNIX_EPOCH ms).
2 integration tests: backup created with prev content, no backup when file absent.
EOF
)"
```
