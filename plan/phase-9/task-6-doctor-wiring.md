# Task 6 — `cchud doctor` subcommand в main.rs + help text

**Цель:** Wire `cchud doctor` в `src/main.rs::main()` через новую match arm. Обновить help text. Убедиться, что `cchud doctor` запускается даже без `tui` feature (doctor — pure check, не TUI).

**Files:**
- Modify: `src/main.rs::main` — добавить `Some("doctor") => commands::doctor::run(&args[1..])`.
- Modify: `src/main.rs::print_help` — добавить строку `cchud doctor`.

---

- [ ] **Step 1: Найти match block в `main`**

В `src/main.rs::main` сейчас есть:

```rust
match args.first().map(String::as_str) {
    Some("--version") => { … },
    Some("--help" | "-h") => { … },
    Some("install") => commands::install::run(&args[1..]),
    Some("configure") => configure_command(&args[1..]),
    Some("import") => import_command(&args[1..]),
    Some(other) if other.starts_with("--") => { … },
    _ => render_pipeline(),
}
```

Добавить branch ПЕРЕД `Some(other)`:

```rust
Some("doctor") => commands::doctor::run(&args[1..]),
```

(`doctor` НЕ под `#[cfg(feature = "tui")]` — он должен работать в no-default-features build.)

- [ ] **Step 2: Update `print_help`**

Прочитать `print_help` и заменить блок USAGE на:

```rust
fn print_help() {
    println!(
        "cchud {} — fast Rust statusline for Claude Code",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("  cchud                       read JSON payload from stdin, render statusline");
    println!("  cchud install               wire cchud + self-relocate to ~/.local/bin/");
    println!("  cchud install --force       overwrite existing statusLine");
    println!("  cchud install --no-relocate skip self-copy (dev-only)");
    println!("  cchud doctor                run 9-check environment report");
    println!("  cchud doctor --json         emit machine-readable JSON report");
    println!("  cchud configure             open the interactive TUI configurator");
    println!("  cchud import [args]         migrate ccstatusline config; see --help");
    println!("  cchud --version             print version");
    println!("  cchud --help                print this help");
}
```

- [ ] **Step 3: Smoke build (default features)**

```bash
cargo build --locked
./target/debug/cchud doctor
echo "exit=$?"
./target/debug/cchud doctor --json
echo "exit=$?"
./target/debug/cchud --help | grep doctor
```

Expected:
- Human report — 9 checks с marks.
- JSON output — корректный JSON с `summary`/`checks`/`exit_code`.
- Help содержит `cchud doctor`.

- [ ] **Step 4: Smoke build (no-default-features)**

```bash
cargo build --locked --no-default-features
./target/debug/cchud doctor
echo "exit=$?"
```

Expected: `cchud doctor` работает БЕЗ `tui` feature. Exit 0/1/2.

- [ ] **Step 5: Run полный suite**

```bash
cargo test --locked
cargo test --locked --no-default-features
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features -- -D warnings
cargo fmt --check
```

Expected: всё зелёное.

- [ ] **Step 6: Verify binary size**

```bash
cargo build --release --locked
ls -la target/release/cchud
```

Expected: < 10 MB.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "$(cat <<'EOF'
feat(phase-9): T6 cchud doctor wiring — main.rs subcommand + help text

Doctor работает БЕЗ tui feature (no-default-features build OK).
help text дополнен: doctor / doctor --json / install --no-relocate.
EOF
)"
```
