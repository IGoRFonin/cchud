//! Powerline (filled-arrow) renderer — Phase 4 Task 9.
//!
//! Default separator U+E0B0 (right-pointing filled triangle). Each segment
//! emits: `style(prev_bg→bg, sep)` + `style(seg, fg, bg)`. After the last
//! segment a final separator transitions to `theme.terminal_bg`.
//!
//! Segments without an explicit fg/bg pull from the theme cycle by index.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::hyperlink::link;
use super::themes::PowerlineTheme;
use super::{Color, ColorLevel, Segment, Style};

pub const DEFAULT_SEPARATOR_LEFT: char = '\u{e0b0}';
pub const DEFAULT_SEPARATOR_RIGHT: char = '\u{e0b2}';

#[derive(Debug)]
pub struct Powerline {
    pub theme: PowerlineTheme,
    pub separator_left: char,
    pub separator_right: char,
    pub start_cap: Option<char>,
    pub end_cap: Option<char>,
    pub level: ColorLevel,
    pub hyperlinks: bool,
}

impl Powerline {
    #[must_use]
    pub fn new(theme: PowerlineTheme, level: ColorLevel, hyperlinks: bool) -> Self {
        Self {
            theme,
            separator_left: DEFAULT_SEPARATOR_LEFT,
            separator_right: DEFAULT_SEPARATOR_RIGHT,
            start_cap: None,
            end_cap: None,
            level,
            hyperlinks,
        }
    }

    #[must_use]
    pub fn render(&self, segments: &[Segment]) -> String {
        let visible: Vec<&Segment> = segments.iter().filter(|s| !s.text.is_empty()).collect();
        if visible.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        let mut prev_bg = self.theme.terminal_bg;

        for (i, seg) in visible.iter().enumerate() {
            let bg = seg.style.bg.unwrap_or_else(|| self.cycle_bg(i));
            let fg = seg.style.fg.unwrap_or_else(|| self.cycle_fg(i));

            // Transition separator: fg=prev_bg, bg=this_bg.
            let sep_style = Style::none().fg(prev_bg).bg(bg);
            out.push_str(&sep_style.render(&self.separator_left.to_string(), self.level));

            // Pad text with one space on each side (upstream parity).
            let body_text = format!(" {} ", seg.text);
            let body_style = Style {
                fg: Some(fg),
                bg: Some(bg),
                bold: seg.style.bold,
                italic: seg.style.italic,
                dim: seg.style.dim,
                underline: seg.style.underline,
            };
            let styled_body = body_style.render(&body_text, self.level);
            let body_with_link = match &seg.hyperlink {
                Some(url) => link(&styled_body, url, self.hyperlinks),
                None => styled_body,
            };
            out.push_str(&body_with_link);

            prev_bg = bg;
        }

        // Final transition to terminal_bg.
        let final_sep = Style::none().fg(prev_bg).bg(self.theme.terminal_bg);
        out.push_str(&final_sep.render(&self.separator_left.to_string(), self.level));

        out
    }

    fn cycle_bg(&self, idx: usize) -> Color {
        let cycle = &self.theme.bg_cycle;
        if cycle.is_empty() {
            self.theme.default_bg
        } else {
            cycle[idx % cycle.len()]
        }
    }

    fn cycle_fg(&self, idx: usize) -> Color {
        let cycle = &self.theme.fg_cycle;
        if cycle.is_empty() {
            self.theme.default_fg
        } else {
            cycle[idx % cycle.len()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::themes::{DRACULA, PowerlineTheme};

    fn theme() -> PowerlineTheme {
        (&DRACULA).into()
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        assert_eq!(p.render(&[]), "");
    }

    #[test]
    fn empty_segments_filtered() {
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [Segment::plain(""), Segment::plain("")];
        assert_eq!(p.render(&segs), "");
    }

    #[test]
    fn single_segment_emits_two_separators() {
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [Segment::plain("hi")];
        let out = p.render(&segs);
        // Two separators: opening transition + closing transition.
        let sep_count = out.matches(DEFAULT_SEPARATOR_LEFT).count();
        assert_eq!(sep_count, 2, "expected 2 separators in {out:?}");
        assert!(out.contains("hi"));
    }

    #[test]
    fn three_segments_emit_four_separators() {
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [
            Segment::plain("a"),
            Segment::plain("b"),
            Segment::plain("c"),
        ];
        let out = p.render(&segs);
        assert_eq!(out.matches(DEFAULT_SEPARATOR_LEFT).count(), 4);
    }

    #[test]
    fn cycles_wrap_around_when_more_segments_than_cycle_len() {
        // Dracula has 4 colors; emit 6 segments, last two reuse first two cycle slots.
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs: Vec<Segment> = (0..6).map(|i| Segment::plain(format!("s{i}"))).collect();
        let out = p.render(&segs);
        for i in 0..6 {
            assert!(out.contains(&format!("s{i}")), "missing s{i} in {out:?}");
        }
    }

    #[test]
    fn segment_explicit_bg_overrides_cycle() {
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let style = Style::none().bg(Color::Rgb(123, 45, 67));
        let segs = [Segment::styled("x", style)];
        let out = p.render(&segs);
        assert!(out.contains("48;2;123;45;67"), "expected forced bg in {out:?}");
    }

    #[test]
    fn level_none_emits_no_ansi() {
        let p = Powerline::new(theme(), ColorLevel::None, false);
        let segs = [Segment::plain("hi")];
        let out = p.render(&segs);
        assert!(!out.contains('\x1b'), "expected no ANSI: {out:?}");
        // Текст и separators всё равно присутствуют.
        assert!(out.contains("hi"));
        assert_eq!(out.matches(DEFAULT_SEPARATOR_LEFT).count(), 2);
    }
}
