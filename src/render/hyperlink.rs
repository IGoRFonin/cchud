//! OSC 8 hyperlink wrapper — Phase 4 Task 6.
//!
//! Compact: terminal detection via `supports-hyperlinks 3`, formatting is
//! a 2-line `format!` (no extra crate).

#![deny(clippy::unwrap_used, clippy::expect_used)]

/// Wrap `text` with OSC 8 hyperlink to `url` if `supported`. Otherwise return
/// `text` unchanged. Uses ST = `\x1b\\` (BEL `\x07` is also valid; we follow
/// upstream ccstatusline which uses ESC-backslash for broadest compat).
#[must_use]
pub fn link(text: &str, url: &str, supported: bool) -> String {
    if supported {
        format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
    } else {
        text.to_string()
    }
}

/// Detect OSC 8 support on stdout. Honors `NO_HYPERLINKS=1` (standard).
#[must_use]
pub fn supports_hyperlinks_detect() -> bool {
    supports_hyperlinks::supports_hyperlinks()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_wraps_with_osc8_when_supported() {
        let out = link("anchor", "https://x.com", true);
        assert_eq!(out, "\x1b]8;;https://x.com\x1b\\anchor\x1b]8;;\x1b\\");
    }

    #[test]
    fn link_returns_text_when_unsupported() {
        assert_eq!(link("anchor", "https://x.com", false), "anchor");
    }

    #[test]
    fn link_passes_through_url_with_query_string() {
        let url = "https://example.com/path?a=1&b=2";
        let out = link("L", url, true);
        assert!(out.contains(url));
        assert!(out.starts_with("\x1b]8;;"));
        assert!(out.ends_with("\x1b]8;;\x1b\\"));
    }
}
