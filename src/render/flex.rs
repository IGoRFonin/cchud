//! Phase 7 Task 11 — `flex_mode` width truncation.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::types::config::FlexMode;
use crate::util::ansi::visible_width;

/// Returns truncate budget. `None` = no truncation.
#[must_use]
#[allow(dead_code)]
pub const fn flex_budget(mode: FlexMode, term_width: usize) -> Option<usize> {
    match mode {
        FlexMode::Disabled => None,
        FlexMode::Full => Some(term_width),
        FlexMode::FullMinus20 => term_width.checked_sub(20),
        FlexMode::FullMinus40 => term_width.checked_sub(40),
    }
}

/// Truncate `rendered` to `budget` visible columns. Adds ellipsis `…` if truncated.
#[must_use]
#[allow(dead_code)]
pub fn truncate_to_budget(rendered: &str, budget: Option<usize>) -> String {
    let Some(budget) = budget else {
        return rendered.to_string();
    };
    let visible = visible_width(rendered);
    if visible <= budget {
        return rendered.to_string();
    }
    let target = budget.saturating_sub(1);
    let truncated = truncate_visible(rendered, target);
    format!("{truncated}…")
}

/// Truncate to `target` visible columns, preserving ANSI escape sequences as-is.
fn truncate_visible(s: &str, target: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;

    let mut out = String::new();
    let mut visible = 0_usize;
    let mut in_escape = false;

    for g in s.graphemes(true) {
        if g == "\u{1b}" {
            in_escape = true;
            out.push_str(g);
            continue;
        }
        if in_escape {
            out.push_str(g);
            // ANSI CSI ends on a letter in 0x40..=0x7e; OSC ends on BEL or ST.
            if g
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == 'm' || c == '\\')
            {
                in_escape = false;
            }
            continue;
        }
        let w = UnicodeWidthStr::width(g);
        if visible + w > target {
            break;
        }
        out.push_str(g);
        visible += w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_full_returns_term_width() {
        assert_eq!(flex_budget(FlexMode::Full, 80), Some(80));
    }

    #[test]
    fn budget_full_minus_40_subtracts() {
        assert_eq!(flex_budget(FlexMode::FullMinus40, 100), Some(60));
    }

    #[test]
    fn budget_disabled_returns_none() {
        assert_eq!(flex_budget(FlexMode::Disabled, 100), None);
    }

    #[test]
    fn budget_saturating_sub_for_narrow_term() {
        assert_eq!(flex_budget(FlexMode::FullMinus40, 30), None);
    }

    #[test]
    fn truncate_short_returns_unchanged() {
        assert_eq!(truncate_to_budget("hello", Some(80)), "hello");
    }

    #[test]
    fn truncate_long_with_ellipsis() {
        let out = truncate_to_budget("hello world", Some(7));
        assert_eq!(out, "hello …");
    }

    #[test]
    fn truncate_none_budget_returns_unchanged() {
        assert_eq!(truncate_to_budget("any string", None), "any string");
    }
}
