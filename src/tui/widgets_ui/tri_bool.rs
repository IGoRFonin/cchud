//! Tri-state bool. None / Some(true) / Some(false).

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub const fn glyph(value: Option<bool>) -> &'static str {
    match value {
        None => "[ ]",
        Some(true) => "[✓]",
        Some(false) => "[✗]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_for_three_states() {
        assert_eq!(glyph(None), "[ ]");
        assert_eq!(glyph(Some(true)), "[✓]");
        assert_eq!(glyph(Some(false)), "[✗]");
    }
}
