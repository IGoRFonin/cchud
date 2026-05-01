//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 2 walking skeleton. Reads JSON payload from stdin, renders
//! configured widgets joined by Plain renderer, prints to stdout.
//! Config loading lands in Task 6; install command in Task 7.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod cache;
mod commands;
mod config;
mod git;
mod render;
mod types;
mod util;
mod widgets;

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
        Some(other) if other.starts_with("--") => {
            eprintln!("cchud: unknown flag: {other}");
            eprintln!("       run 'cchud --help' for usage");
            ExitCode::from(2)
        }
        _ => render_pipeline(),
    }
}

fn print_help() {
    println!(
        "cchud {} — fast Rust statusline for Claude Code",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("  cchud                  read JSON payload from stdin, render statusline");
    println!("  cchud install          wire cchud into ~/.claude/settings.json");
    println!("  cchud install --force  overwrite existing statusLine");
    println!("  cchud --version        print version");
    println!("  cchud --help           print this help");
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
