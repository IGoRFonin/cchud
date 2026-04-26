//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 2 walking skeleton. Reads JSON payload from stdin, renders
//! configured widgets joined by Plain renderer, prints to stdout.
//! Config loading lands in Task 6; install command in Task 7.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod config;
mod render;
mod types;
mod widgets;

use std::process::ExitCode;

use crate::render::{Plain, Renderer};
use crate::types::payload::StatusPayload;
use crate::widgets::{RenderContext, build_widgets};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => render_pipeline(),
    }
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
    let widgets = build_widgets(&settings);
    let segments: Vec<String> = widgets.iter().filter_map(|w| w.render(&ctx)).collect();
    let renderer = Plain {
        separator: " | ".into(),
    };
    println!("{}", renderer.render(&segments));
    ExitCode::SUCCESS
}
