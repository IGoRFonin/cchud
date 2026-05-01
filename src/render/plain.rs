//! Plain (single-line) renderer — Phase 4 Task 8 / Phase 7 Task 11.
//!
//! Joins segments with `separator`. If `level != None`, applies each
//! segment's `Style`. If `hyperlinks == true`, wraps segments with
//! `Segment.hyperlink == Some(_)` in OSC 8.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::hyperlink::link;
use super::{ColorLevel, RenderState, Segment};
use crate::types::config::ThemeConfig;

#[derive(Debug)]
pub struct Plain {
    pub separator: String,
    pub level: ColorLevel,
    pub hyperlinks: bool,
}

impl Plain {
    #[must_use]
    pub fn render_line(
        &self,
        segments: &[Segment],
        _state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return render_minimalist(segments);
        }

        let mut parts: Vec<String> = Vec::with_capacity(segments.len());
        for seg in segments {
            if seg.text.is_empty() || seg.align_marker {
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

pub(super) fn render_minimalist(segments: &[Segment]) -> String {
    segments
        .iter()
        .filter(|s| !s.text.is_empty() && !s.align_marker)
        .map(|s| strip_emoji_prefix(&s.text))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn strip_emoji_prefix(s: &str) -> String {
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        let cp = c as u32;
        if c == ' ' || (0x1F000..=0x1FFFF).contains(&cp) {
            chars.next();
        } else {
            break;
        }
    }
    chars.collect()
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

    fn theme() -> ThemeConfig {
        ThemeConfig::default()
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let mut state = RenderState::default();
        let t = theme();
        assert_eq!(p(ColorLevel::None, false).render_line(&[], &mut state, &t), "");
    }

    #[test]
    fn filters_empty_segments() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::plain("a"), Segment::plain(""), Segment::plain("b")];
        assert_eq!(p(ColorLevel::None, false).render_line(&segs, &mut state, &t), "a | b");
    }

    #[test]
    fn level_none_strips_color() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::styled("x", Style::none().fg(Color::Rgb(255, 0, 0)))];
        assert_eq!(p(ColorLevel::None, false).render_line(&segs, &mut state, &t), "x");
    }

    #[test]
    fn truecolor_emits_ansi() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::styled("x", Style::none().fg(Color::Rgb(255, 0, 0)))];
        let out = p(ColorLevel::TrueColor, false).render_line(&segs, &mut state, &t);
        assert!(out.contains("\x1b["), "expected ANSI in {out:?}");
    }

    #[test]
    fn hyperlink_wraps_segment_when_supported() {
        let mut state = RenderState::default();
        let t = theme();
        let mut s = Segment::plain("anchor");
        s.hyperlink = Some("https://x.com".into());
        let out = p(ColorLevel::None, true).render_line(&[s], &mut state, &t);
        assert!(out.starts_with("\x1b]8;;https://x.com"), "got: {out:?}");
        assert!(out.ends_with("\x1b]8;;\x1b\\"));
    }

    #[test]
    fn hyperlink_dropped_when_unsupported() {
        let mut state = RenderState::default();
        let t = theme();
        let mut s = Segment::plain("anchor");
        s.hyperlink = Some("https://x.com".into());
        assert_eq!(p(ColorLevel::None, false).render_line(&[s], &mut state, &t), "anchor");
    }

    #[test]
    fn align_marker_segment_is_skipped() {
        let mut state = RenderState::default();
        let t = theme();
        let mut marker = Segment::plain("");
        marker.align_marker = true;
        let segs = [Segment::plain("a"), marker, Segment::plain("b")];
        assert_eq!(
            p(ColorLevel::None, false).render_line(&segs, &mut state, &t),
            "a | b"
        );
    }
}
