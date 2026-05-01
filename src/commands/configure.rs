//! `cchud configure` — TUI entry-point. Phase 8 Task 13.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io::IsTerminal;
use std::process::ExitCode;

#[must_use]
pub fn run(_args: &[String]) -> ExitCode {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("cchud configure: must be run in a terminal");
        return ExitCode::from(2);
    }

    let settings = crate::config::load();
    let (sample, tempfile) = crate::tui::sample::payload();

    match crate::tui::run_configure(settings, sample, tempfile) {
        Ok(saved) => {
            if saved {
                eprintln!("cchud configure: settings saved");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("cchud configure: {e}");
            ExitCode::from(1)
        }
    }
}
