//! cchud — Fast Rust statusline for Claude Code CLI.
//!
//! Phase 2 walking skeleton: types defined, render pipeline lands in Task 4.

#![deny(clippy::unwrap_used, clippy::expect_used)]

mod types;

use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let len = input.len();
    println!("cchud (skeleton) | input bytes: {len}");
    Ok(())
}
