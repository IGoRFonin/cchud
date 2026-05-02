#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn configure_without_tty_exits_2_with_message() {
    Command::cargo_bin("cchud")
        .unwrap()
        .arg("configure")
        .write_stdin("")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("must be run in a terminal"));
}
