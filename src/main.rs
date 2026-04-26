//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 1 skeleton: read JSON payload from stdin, print byte count.
//! Real widget pipeline lands in Phase 2.

use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let len = input.len();
    println!("cchud (skeleton) | input bytes: {len}");
    Ok(())
}
