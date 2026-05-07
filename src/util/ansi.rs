//! ANSI strip + visible width helpers.
//!
//! Used by `widgets::TerminalWidth` (subtract content width) and
//! `render::powerline::Powerline` (truncate by terminal width).
//!
//! `strip` runs an `anstyle-parse` state machine, so it handles malformed
//! and mixed CSI/OSC sequences without panic.

#![deny(clippy::unwrap_used, clippy::expect_used)]

/// Strip CSI + OSC ANSI sequences, return printable text.
#[must_use]
#[allow(dead_code)]
pub fn strip(input: &str) -> String {
    use anstyle_parse::{Parser, Perform};

    struct Sink(String);
    impl Perform for Sink {
        fn print(&mut self, c: char) {
            self.0.push(c);
        }
        fn execute(&mut self, byte: u8) {
            // Preserve newline, tab, carriage return — drop other C0/C1.
            if matches!(byte, b'\n' | b'\t' | b'\r') {
                self.0.push(byte as char);
            }
        }
        // Default impls swallow CSI / OSC / DCS / ESC.
    }

    let mut parser = Parser::<anstyle_parse::DefaultCharAccumulator>::new();
    let mut sink = Sink(String::with_capacity(input.len()));
    for &b in input.as_bytes() {
        parser.advance(&mut sink, b);
    }
    sink.0
}

/// Visible width = unicode-width of stripped string.
#[must_use]
#[allow(dead_code)]
pub fn visible_width(input: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(strip(input).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_csi_color() {
        assert_eq!(strip("\x1b[31mhello\x1b[0m"), "hello");
    }

    #[test]
    fn strip_removes_osc8_hyperlink() {
        let s = "\x1b]8;;https://x.com\x1b\\link\x1b]8;;\x1b\\";
        assert_eq!(strip(s), "link");
    }

    #[test]
    fn strip_handles_mixed_csi_and_osc() {
        let s = "\x1b[1mbold \x1b]8;;u\x1b\\anchor\x1b]8;;\x1b\\\x1b[0m";
        assert_eq!(strip(s), "bold anchor");
    }

    #[test]
    fn strip_passes_through_plain_text() {
        assert_eq!(strip("just words"), "just words");
    }

    #[test]
    fn strip_does_not_panic_on_truncated_escape() {
        let _ = strip("\x1b[3"); // unterminated CSI — must not panic
    }

    #[test]
    fn visible_width_counts_ascii() {
        assert_eq!(visible_width("hello"), 5);
    }

    #[test]
    fn visible_width_counts_emoji_as_two() {
        // U+1F600 GRINNING FACE — width 2 by unicode-width.
        assert_eq!(visible_width("\u{1f600}"), 2);
    }

    #[test]
    fn visible_width_counts_cjk_as_two() {
        assert_eq!(visible_width("中文"), 4);
    }

    #[test]
    fn visible_width_strips_ansi_first() {
        assert_eq!(visible_width("\x1b[31mhi\x1b[0m"), 2);
    }
}
