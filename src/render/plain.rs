//! Plain (single-line) renderer.
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
    pub fn compose_inner(
        &self,
        segments: &[Segment],
        _state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> Vec<super::StyledSegment> {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return compose_minimalist(segments);
        }

        let visible: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.text.is_empty() && !s.align_marker)
            .collect();

        let mut out = Vec::with_capacity(visible.len() * 2);
        for (i, seg) in visible.iter().enumerate() {
            if i > 0 {
                out.push(super::StyledSegment::plain(self.separator.clone()));
            }
            out.push(super::StyledSegment {
                text: seg.text.clone(),
                style: seg.style,
                hyperlink: seg.hyperlink.clone(),
            });
        }
        out
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let composed = self.compose_inner(segments, state, theme);
        emit_plain(&composed, self.level, self.hyperlinks)
    }
}

fn emit_plain(
    composed: &[super::StyledSegment],
    level: super::ColorLevel,
    hyperlinks: bool,
) -> String {
    composed
        .iter()
        .map(|s| {
            let painted = s.style.render(&s.text, level);
            match &s.hyperlink {
                Some(url) => link(&painted, url, hyperlinks),
                None => painted,
            }
        })
        .collect::<String>()
}

pub(super) fn compose_minimalist(segments: &[Segment]) -> Vec<super::StyledSegment> {
    let visible: Vec<&Segment> = segments
        .iter()
        .filter(|s| !s.text.is_empty() && !s.align_marker)
        .collect();
    if visible.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(visible.len() * 2);
    for (i, seg) in visible.iter().enumerate() {
        if i > 0 {
            out.push(super::StyledSegment::plain(" | "));
        }
        out.push(super::StyledSegment::plain(strip_emoji_prefix(&seg.text)));
    }
    out
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
        assert_eq!(
            p(ColorLevel::None, false).render_line(&[], &mut state, &t),
            ""
        );
    }

    #[test]
    fn filters_empty_segments() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::plain("a"), Segment::plain(""), Segment::plain("b")];
        assert_eq!(
            p(ColorLevel::None, false).render_line(&segs, &mut state, &t),
            "a | b"
        );
    }

    #[test]
    fn level_none_strips_color() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::styled(
            "x",
            Style::none().fg(Color::Rgb(255, 0, 0)),
        )];
        assert_eq!(
            p(ColorLevel::None, false).render_line(&segs, &mut state, &t),
            "x"
        );
    }

    #[test]
    fn truecolor_emits_ansi() {
        let mut state = RenderState::default();
        let t = theme();
        let segs = [Segment::styled(
            "x",
            Style::none().fg(Color::Rgb(255, 0, 0)),
        )];
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
        assert_eq!(
            p(ColorLevel::None, false).render_line(&[s], &mut state, &t),
            "anchor"
        );
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
