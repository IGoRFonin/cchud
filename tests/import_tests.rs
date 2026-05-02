#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cchud_with_dest(dest: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("cchud").unwrap();
    cmd.env("CCHUD_CONFIG", dest);
    cmd
}

#[test]
fn imports_full_ccstatusline_config() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-full-cc.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("widgets across"));
    assert!(dest.exists());
}

#[test]
fn import_missing_source_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "/nonexistent/__nope.json"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn import_malformed_json_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "tests/configs/import-malformed.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not valid JSON"));
}

#[test]
fn import_unknown_widgets_warns_but_succeeds() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args([
            "import",
            "--from",
            "tests/configs/import-unknown-widgets.json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("ccstatusline-fake-1"))
        .stderr(predicate::str::contains("ccstatusline-fake-2"));
}

#[test]
fn import_empty_section_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args([
            "import",
            "--from",
            "tests/configs/import-empty-section.json",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing imported"));
}

#[test]
fn import_existing_dest_without_force_exits_1() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    std::fs::write(&dest, "{}").unwrap();

    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-full-cc.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn import_ccstatusline_nested_section_succeeds() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-with-section.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("widgets across"));
    assert!(dest.exists());
}

#[test]
fn import_then_configure_without_tty_exits_2() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    cchud_with_dest(&dest)
        .args([
            "import",
            "--from",
            "tests/configs/import-full-cc.json",
            "--then-configure",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn import_existing_dest_with_force_overwrites_and_backs_up() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    std::fs::write(&dest, "{}").unwrap();

    cchud_with_dest(&dest)
        .args([
            "import",
            "--from",
            "tests/configs/import-full-cc.json",
            "--force",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("backup at"));

    // Backup file должен начинаться с `<dest>.bak.`
    let has_bak = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .any(|e| e.file_name().to_string_lossy().contains(".bak."));
    assert!(has_bak, "expected at least one .bak.* file");
}
