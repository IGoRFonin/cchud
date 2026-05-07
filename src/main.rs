//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Reads JSON payload from stdin, renders configured widgets joined by the
//! Plain or Powerline renderer, prints to stdout.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod cache;
mod commands;
mod config;
mod git;
mod render;
mod types;
mod util;
mod widgets;

#[cfg(feature = "tui")]
mod tui;

use std::io::IsTerminal;
use std::process::ExitCode;

use crate::render::{Renderer, Segment};
use crate::types::payload::StatusPayload;
use crate::widgets::{RenderContext, build_widgets};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("install") => commands::install::run(&args[1..]),
        Some("doctor") => commands::doctor::run(&args[1..]),
        Some("configure") => configure_command(&args[1..]),
        Some("import") => import_command(&args[1..]),
        Some(other) if other.starts_with("--") => {
            eprintln!("cchud: unknown flag: {other}");
            eprintln!("       run 'cchud --help' for usage");
            ExitCode::from(2)
        }
        _ => default_action(),
    }
}

/// `cchud` без аргументов:
/// - stdin — TTY → интерактивный режим: idempotent install + TUI configurator.
/// - stdin — pipe → render statusline (Claude Code путь).
fn default_action() -> ExitCode {
    if std::io::stdin().is_terminal() {
        // Best-effort install: stderr-warnings on failure (occupied statusLine, etc.)
        // не блокируют запуск TUI — пользователь увидит сообщение и решит сам.
        let _ = commands::install::run(&[]);
        configure_command(&[])
    } else {
        render_pipeline()
    }
}

fn print_help() {
    println!(
        "cchud {} — fast Rust statusline for Claude Code",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("  cchud                       interactive: install + open TUI (when stdin is a tty)");
    println!("  cchud                       render statusline from stdin JSON (when piped)");
    println!("  cchud install               wire cchud + self-relocate to ~/.local/bin/");
    println!("  cchud install --force       overwrite existing statusLine");
    println!("  cchud install --no-relocate skip self-copy (dev-only)");
    println!("  cchud doctor                run 9-check environment report (stdout; exit 0/1/2)");
    println!("  cchud doctor --json         emit machine-readable JSON to stdout (exit 0/1/2)");
    println!("  cchud configure             open the interactive TUI configurator");
    println!("  cchud import [args]         migrate ccstatusline config; see --help");
    println!("  cchud --version             print version");
    println!("  cchud --help                print this help");
}

#[cfg(feature = "tui")]
fn configure_command(args: &[String]) -> ExitCode {
    commands::configure::run(args)
}

#[cfg(not(feature = "tui"))]
fn configure_command(_args: &[String]) -> ExitCode {
    eprintln!("cchud configure: requires the 'tui' feature; rebuild with default features");
    ExitCode::from(2)
}

#[cfg(feature = "tui")]
fn import_command(args: &[String]) -> ExitCode {
    commands::import::run(args)
}

#[cfg(not(feature = "tui"))]
fn import_command(_args: &[String]) -> ExitCode {
    eprintln!("cchud import: requires the 'tui' feature; rebuild with default features");
    ExitCode::from(2)
}

fn render_pipeline() -> ExitCode {
    let payload: StatusPayload = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cchud: invalid payload: {e}");
            return ExitCode::SUCCESS; // AC-007: graceful, ничего в stdout
        }
    };
    let settings = config::load();
    let ctx = RenderContext::new(&payload, &settings);
    let lines = build_widgets(&settings);
    let renderer = Renderer::from_settings(&settings);
    let mut state = crate::render::RenderState::default();
    let term_width = crate::util::terminal_width();
    let budget = crate::render::flex::flex_budget(settings.theme.flex_mode, term_width);

    let mut output = String::new();
    for (i, line_widgets) in lines.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        let segments: Vec<Segment> = line_widgets
            .iter()
            .filter_map(|(w, ovr)| {
                let text = w.render(&ctx)?;
                let is_align = w.id() == "align-right";
                let style = crate::render::apply_widget_style(
                    w.default_style(),
                    None,
                    ovr,
                    &settings.theme,
                );
                Some(Segment {
                    text,
                    style,
                    hyperlink: w.hyperlink(&ctx),
                    align_marker: is_align,
                })
            })
            .collect();
        let line_str = renderer.render_line(&segments, &mut state, &settings.theme);
        let truncated = crate::render::flex::truncate_to_budget(&line_str, budget);
        output.push_str(&truncated);
        if !settings.theme.continue_theme_across_lines {
            state.reset();
        }
    }
    println!("{output}");
    ExitCode::SUCCESS
}
