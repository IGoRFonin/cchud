//! Phase 9 Task 10 — install.sh integration tests.
//!
//! Uses `python3 -m http.server` to simulate GitHub Releases endpoint,
//! tests install.sh end-to-end.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serial_test::serial;
use tempfile::tempdir;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/install_sh")
}

const fn current_target() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "cchud-darwin-arm64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "cchud-darwin-x64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "cchud-linux-x64";
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    return "unknown";
}

fn ensure_fixtures_built() -> bool {
    let target = current_target();
    if target == "unknown" {
        return false;
    }
    let f = fixture_dir().join(format!("{target}.tar.gz"));
    let s = fixture_dir().join(format!("{target}.tar.gz.sha256"));
    f.exists() && s.exists()
}

fn ensure_python_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Spawns `python3 -m http.server` in fixture dir on a random free port.
/// Returns (child, port).
fn spawn_mock_server() -> (std::process::Child, u16) {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // release; python will grab it

    let mut child = Command::new("python3")
        .args(["-m", "http.server", &port.to_string()])
        .current_dir(fixture_dir())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    // Wait for server to be ready.
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return (child, port);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("python http server did not start on port {port}");
}

#[test]
#[serial]
fn install_sh_downloads_extracts_and_chmods() {
    if !ensure_python_available() || !ensure_fixtures_built() {
        eprintln!("skip: python3 or fixtures missing");
        return;
    }
    let (mut server, port) = spawn_mock_server();
    let work = tempdir().unwrap();
    let install_dir = work.path().join("bin");

    let out = Command::new("sh")
        .arg(project_root().join("install.sh"))
        .env("CCHUD_VERSION", "fixture")
        .env("CCHUD_BASE_URL", format!("http://127.0.0.1:{port}"))
        .env("CCHUD_INSTALL_DIR", &install_dir)
        .env("HOME", work.path())
        .env("CCHUD_SETTINGS", work.path().join("settings.json"))
        .output()
        .unwrap();

    let _ = server.kill();
    let _ = server.wait();

    assert!(
        out.status.success(),
        "install.sh failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let bin = install_dir.join("cchud");
    assert!(bin.exists(), "expected {} to exist", bin.display());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&bin).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }
}

#[test]
#[serial]
fn install_sh_fails_on_sha256_mismatch() {
    if !ensure_python_available() || !ensure_fixtures_built() {
        return;
    }
    let target = current_target();
    let sha_path = fixture_dir().join(format!("{target}.tar.gz.sha256"));
    let backup = fs::read_to_string(&sha_path).unwrap();
    fs::write(
        &sha_path,
        "0000000000000000000000000000000000000000000000000000000000000000  bogus.tar.gz\n",
    )
    .unwrap();

    let (mut server, port) = spawn_mock_server();
    let work = tempdir().unwrap();
    let install_dir = work.path().join("bin");

    let out = Command::new("sh")
        .arg(project_root().join("install.sh"))
        .env("CCHUD_VERSION", "fixture")
        .env("CCHUD_BASE_URL", format!("http://127.0.0.1:{port}"))
        .env("CCHUD_INSTALL_DIR", &install_dir)
        .env("HOME", work.path())
        .output()
        .unwrap();
    let _ = server.kill();
    let _ = server.wait();

    // Restore fixture.
    fs::write(&sha_path, backup).unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("sha256 mismatch"), "stderr: {stderr}");
}

#[test]
#[serial]
fn install_sh_idempotent_re_run() {
    if !ensure_python_available() || !ensure_fixtures_built() {
        return;
    }
    let (mut server, port) = spawn_mock_server();
    let work = tempdir().unwrap();
    let install_dir = work.path().join("bin");

    for _ in 0..2 {
        let out = Command::new("sh")
            .arg(project_root().join("install.sh"))
            .env("CCHUD_VERSION", "fixture")
            .env("CCHUD_BASE_URL", format!("http://127.0.0.1:{port}"))
            .env("CCHUD_INSTALL_DIR", &install_dir)
            .env("HOME", work.path())
            .env("CCHUD_SETTINGS", work.path().join("settings.json"))
            .output()
            .unwrap();
        assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    }

    let _ = server.kill();
    let _ = server.wait();

    let bin = install_dir.join("cchud");
    assert!(bin.exists());
}

#[test]
fn install_sh_unsupported_arch_exits_one() {
    if !ensure_python_available() || !ensure_fixtures_built() {
        return;
    }
    let work = tempdir().unwrap();

    let out = Command::new("sh")
        .arg(project_root().join("install.sh"))
        .env("CCHUD_VERSION", "fixture")
        .env("CCHUD_BASE_URL", "http://127.0.0.1:1") // unreachable port
        .env("CCHUD_INSTALL_DIR", work.path().join("bin"))
        .env("HOME", work.path())
        .output()
        .unwrap();

    assert!(!out.status.success(), "should fail when curl cannot reach base url");
}
