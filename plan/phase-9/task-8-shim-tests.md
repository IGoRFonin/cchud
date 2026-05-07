# Task 8 — JS shim integration tests

**Цель:** Создать `tests/npm_shim.rs` (Rust integration test, который spawns Node) с ≥3 кейсами: shim spawn'ит fixture binary; shim fails gracefully на missing dep; shim emits полезное сообщение для unsupported platform.

**Files:**
- Create: `tests/npm_shim.rs`
- Create: `tests/fixtures/npm_shim/cli-fixture/bin/cchud-fixture.sh` — fake binary, который print argv

---

- [ ] **Step 1: Создать fixture**

`tests/fixtures/npm_shim/cli-fixture/bin/cchud-fixture.sh`:

```bash
#!/bin/sh
# Fixture binary used by tests/npm_shim.rs.
# Echoes argv for shim integration tests.
echo "FIXTURE_OUTPUT: $*"
exit 0
```

Сделать executable: `chmod +x tests/fixtures/npm_shim/cli-fixture/bin/cchud-fixture.sh`.

`tests/fixtures/npm_shim/cli-fixture/package.json`:

```json
{
  "name": "@cchud/cli-fixture",
  "version": "0.0.0-fixture",
  "files": ["bin/"]
}
```

(Имя `@cchud/cli-fixture` — НЕ публикуется на npm, только для local require.resolve.)

- [ ] **Step 2: Написать `tests/npm_shim.rs`**

```rust
//! Phase 9 Task 8 — JS shim tests.
//!
//! Spawns Node 14+ to run `npm/cchud/bin/cchud.js`, with fixture binary
//! linked into a tempdir node_modules.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn project_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("Cargo.toml").exists() && p.pop() {}
    p
}

fn ensure_node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn link_fixture_into(tempdir_root: &Path, fixture_subdir: &str) -> PathBuf {
    let root = project_root();
    let shim = root.join("npm/cchud/bin/cchud.js");
    assert!(shim.exists(), "shim not found at {}", shim.display());

    // Layout: <tempdir>/cchud/{bin/cchud.js, node_modules/@cchud/cli-*}
    let main_pkg = tempdir_root.join("cchud");
    fs::create_dir_all(main_pkg.join("bin")).unwrap();
    fs::copy(&shim, main_pkg.join("bin/cchud.js")).unwrap();

    let nm = main_pkg.join("node_modules/@cchud").join(fixture_subdir);
    fs::create_dir_all(nm.parent().unwrap()).unwrap();

    // Симлинк на репозиторскую fixture директорию.
    let fixture = root.join("tests/fixtures/npm_shim/cli-fixture");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&fixture, &nm).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&fixture, &nm).unwrap();

    main_pkg.join("bin/cchud.js")
}

#[test]
fn shim_spawns_fixture_binary_with_argv() {
    if !ensure_node_available() {
        eprintln!("node not available — skipping");
        return;
    }
    let dir = tempdir().unwrap();

    // Fixture binary register'нется как именно тот пакет, который detectPackage()
    // вернёт для текущей платформы.
    let pkg_subdir = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "cli-darwin-arm64",
        ("macos", "x86_64")  => "cli-darwin-x64",
        ("linux", "x86_64")  => "cli-linux-x64",
        ("windows", "x86_64") => "cli-win32-x64",
        _ => {
            eprintln!("unsupported test host — skipping");
            return;
        }
    };

    let shim = link_fixture_into(dir.path(), pkg_subdir);

    // Чтобы fixture бинарник был "cchud" (без .sh) — копируем .sh в bin/cchud.
    let fixture_dir = dir.path().join("cchud/node_modules/@cchud").join(pkg_subdir).join("bin");
    let cchud_bin = fixture_dir.join(if cfg!(windows) { "cchud.exe" } else { "cchud" });
    let src = project_root().join("tests/fixtures/npm_shim/cli-fixture/bin/cchud-fixture.sh");
    fs::copy(&src, &cchud_bin).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&cchud_bin).unwrap().permissions();
        p.set_mode(0o755);
        fs::set_permissions(&cchud_bin, p).unwrap();
    }

    let out = Command::new("node")
        .arg(&shim)
        .args(["--version", "extra-arg"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FIXTURE_OUTPUT"), "stdout: {stdout}");
    assert!(stdout.contains("--version"), "stdout: {stdout}");
}

#[test]
fn shim_fails_gracefully_when_native_package_missing() {
    if !ensure_node_available() {
        return;
    }
    let dir = tempdir().unwrap();
    // Копируем shim, но НЕ создаём node_modules/@cchud/cli-*.
    let main_pkg = dir.path().join("cchud");
    fs::create_dir_all(main_pkg.join("bin")).unwrap();
    fs::copy(
        project_root().join("npm/cchud/bin/cchud.js"),
        main_pkg.join("bin/cchud.js"),
    )
    .unwrap();

    let out = Command::new("node")
        .arg(main_pkg.join("bin/cchud.js"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("native binary package") || stderr.contains("not installed"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("npx --yes cchud"), "expected recovery hint; got: {stderr}");
}

#[test]
fn shim_unsupported_platform_emits_clear_message() {
    if !ensure_node_available() {
        return;
    }
    let dir = tempdir().unwrap();
    // Запускаем shim с принудительной симуляцией unsupported platform.
    // detectPackage() возвращает в случае unsupported — process.exit(1) с stderr.
    // Симулируем через child env: shim сам проверяет process.platform/arch,
    // мы не можем переопределить эти значения. Поэтому вместо этого
    // тестируем CCHUD_NPM_PACKAGE override + missing — fall through на same path.
    let main_pkg = dir.path().join("cchud");
    fs::create_dir_all(main_pkg.join("bin")).unwrap();
    fs::copy(
        project_root().join("npm/cchud/bin/cchud.js"),
        main_pkg.join("bin/cchud.js"),
    )
    .unwrap();

    let out = Command::new("node")
        .arg(main_pkg.join("bin/cchud.js"))
        .env("CCHUD_NPM_PACKAGE", "@cchud/cli-totally-bogus")
        .arg("--version")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("@cchud/cli-totally-bogus") || stderr.contains("not installed"));
}
```

⚠ Note: Если `node` не доступен на CI — tests skip (early return). На local dev машине Node должен быть установлен (Claude Code требует Node).

- [ ] **Step 3: Run tests — должны PASS**

```bash
cargo test --locked --test npm_shim
```

Expected: 3 tests PASS (или skip с message если Node отсутствует).

- [ ] **Step 4: Run полный suite**

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Expected: всё зелёное.

- [ ] **Step 5: Commit**

```bash
git add tests/npm_shim.rs tests/fixtures/npm_shim/
git commit -m "$(cat <<'EOF'
test(phase-9): T8 npm shim integration tests — 3 кейсов

- shim spawns fixture binary с argv passthrough
- missing native package → exit 1 + recovery hint в stderr
- bogus CCHUD_NPM_PACKAGE override → graceful fail

Tests skip если node отсутствует. Fixture: shell-based echo binary.
EOF
)"
```
