//! Plain (single-line) renderer — Phase 4 Task 8.
//!
//! Joins segments with `separator`. If `level != None`, applies each
//! segment's `Style`. If `hyperlinks == true`, wraps segments with
//! `Segment.hyperlink == Some(_)` in OSC 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::hyperlink::link;
use super::{ColorLevel, Segment};

#[derive(Debug)]
pub struct Plain {
    pub separator: String,
    pub level: ColorLevel,
    pub hyperlinks: bool,
}

impl Plain {
    #[must_use]
    pub fn render(&self, segments: &[Segment]) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(segments.len());
        for seg in segments {
            if seg.text.is_empty() {
                continue;
            }
            let styled = seg.style.render(&seg.text, self.level);
            let with_link = match &seg.hyperlink {
                Some(url) => link(&styled, url, self.hyperlinks),
                None => styled,
            };
            parts.push(with_link);
        }
        parts.join(&self.separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Color, Style};

    fn p(level: ColorLevel, hyperlinks: bool) -> Plain {
        Plain {
            separator: " | ".into(),
            level,
            hyperlinks,
        }
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let r = p(ColorLevel::None, false);
        assert_eq!(r.render(&[]), "");
    }

    #[test]
    fn filters_empty_segments() {
        let r = p(ColorLevel::None, false);
        let segs = [Segment::plain("a"), Segment::plain(""), Segment::plain("b")];
        assert_eq!(r.render(&segs), "a | b");
    }

    #[test]
    fn level_none_strips_color() {
        let r = p(ColorLevel::None, false);
        let segs = [Segment::styled(
            "x",
            Style::none().fg(Color::Rgb(255, 0, 0)),
        )];
        assert_eq!(r.render(&segs), "x");
    }

    #[test]
    fn truecolor_emits_ansi() {
        let r = p(ColorLevel::TrueColor, false);
        let segs = [Segment::styled(
            "x",
            Style::none().fg(Color::Rgb(255, 0, 0)),
        )];
        let out = r.render(&segs);
        assert!(out.contains("\x1b["), "expected ANSI in {out:?}");
    }

    #[test]
    fn hyperlink_wraps_segment_when_supported() {
        let r = p(ColorLevel::None, true);
        let mut s = Segment::plain("anchor");
        s.hyperlink = Some("https://x.com".into());
        let out = r.render(&[s]);
        assert!(out.starts_with("\x1b]8;;https://x.com"), "got: {out:?}");
        assert!(out.ends_with("\x1b]8;;\x1b\\"));
    }

    #[test]
    fn hyperlink_dropped_when_unsupported() {
        let r = p(ColorLevel::None, false);
        let mut s = Segment::plain("anchor");
        s.hyperlink = Some("https://x.com".into());
        assert_eq!(r.render(&[s]), "anchor");
    }
}
