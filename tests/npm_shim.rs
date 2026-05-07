//! Phase 9 Task 8 — JS shim tests.
//!
//! Spawns Node 14+ to run `npm/cchud/bin/cchud.js`, with fixture binary
//! linked into a tempdir `node_modules`.

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

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&entry.path(), &dst_path);
        } else {
            fs::copy(entry.path(), &dst_path).unwrap();
        }
    }
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
    let fixture = root.join("tests/fixtures/npm_shim/cli-fixture");
    copy_dir_all(&fixture, &nm);

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
        ("macos", "x86_64") => "cli-darwin-x64",
        ("linux", "x86_64") => "cli-linux-x64",
        ("windows", "x86_64") => "cli-win32-x64",
        _ => {
            eprintln!("unsupported test host — skipping");
            return;
        }
    };

    let shim = link_fixture_into(dir.path(), pkg_subdir);

    // Чтобы fixture бинарник был "cchud" (без .sh) — копируем .sh в bin/cchud.
    let fixture_dir = dir
        .path()
        .join("cchud/node_modules/@cchud")
        .join(pkg_subdir)
        .join("bin");
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        stderr.contains("npx --yes cchud"),
        "expected recovery hint; got: {stderr}"
    );
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
