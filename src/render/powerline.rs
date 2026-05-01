//! Powerline (filled-arrow) renderer — Phase 4 Task 9 / Phase 7 Task 11.
//!
//! Default separator U+E0B0 (right-pointing filled triangle). Each segment
//! emits: `style(prev_bg→bg, sep)` + `style(seg, fg, bg)`. After the last
//! segment a final separator transitions to `theme.terminal_bg`.
//!
//! Segments without an explicit fg/bg pull from the theme cycle by index.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::hyperlink::link;
use super::themes::PowerlineTheme;
use super::{Color, ColorLevel, RenderState, Segment, Style};
use crate::types::config::ThemeConfig;

pub const DEFAULT_SEPARATOR_LEFT: char = '\u{e0b0}';
pub const DEFAULT_SEPARATOR_RIGHT: char = '\u{e0b2}';

#[derive(Debug)]
#[allow(dead_code)]
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
    pub const fn new(theme: PowerlineTheme, level: ColorLevel, hyperlinks: bool) -> Self {
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
    pub fn render_line(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let term_width = crate::util::terminal_width();
        let force_minimalist = theme.minimalist_mode
            || (theme.compact_threshold > 0 && term_width < theme.compact_threshold as usize);
        if force_minimalist {
            return super::plain::render_minimalist(segments);
        }

        if theme.auto_align {
            if let Some(idx) = segments.iter().position(|s| s.align_marker) {
                let left = self.render_inner(&segments[..idx], state, theme);
                let right = self.render_inner(&segments[idx + 1..], state, theme);
                return pad_to_width(&left, &right, term_width);
            }
        }

        self.render_inner(segments, state, theme)
    }

    fn render_inner(
        &self,
        segments: &[Segment],
        state: &mut RenderState,
        theme: &ThemeConfig,
    ) -> String {
        let visible: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.text.is_empty() && !s.align_marker)
            .collect();
        if visible.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        let mut prev_bg = self.theme.terminal_bg;

        for seg in &visible {
            let idx = state.global_theme_index;
            let bg = seg.style.bg.unwrap_or_else(|| self.cycle_bg(idx));
            let fg = seg.style.fg.unwrap_or_else(|| self.cycle_fg(idx));

            let sep_style = if theme.inherit_separator_colors {
                Style::none().fg(prev_bg).bg(prev_bg)
            } else {
                Style::none().fg(prev_bg).bg(bg)
            };
            out.push_str(&sep_style.render(&self.separator_left.to_string(), self.level));

            let body_text = format!(" {} ", seg.text);
            let body_style = Style {
                fg: Some(fg),
                bg: Some(bg),
                ..seg.style
            };
            let styled_body = body_style.render(&body_text, self.level);
            let body_with_link = match &seg.hyperlink {
                Some(url) => link(&styled_body, url, self.hyperlinks),
                None => styled_body,
            };
            out.push_str(&body_with_link);

            prev_bg = bg;
            state.global_theme_index = state.global_theme_index.saturating_add(1);
        }

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

fn pad_to_width(left: &str, right: &str, width: usize) -> String {
    let lw = crate::util::ansi::visible_width(left);
    let rw = crate::util::ansi::visible_width(right);
    let pad = width.saturating_sub(lw + rw);
    format!("{left}{}{right}", " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::themes::{DRACULA, PowerlineTheme};

    fn theme() -> PowerlineTheme {
        (&DRACULA).into()
    }

    fn default_theme_config() -> ThemeConfig {
        ThemeConfig::default()
    }

    #[test]
    fn empty_input_yields_empty_string() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        assert_eq!(p.render_line(&[], &mut state, &t), "");
    }

    #[test]
    fn empty_segments_filtered() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [Segment::plain(""), Segment::plain("")];
        assert_eq!(p.render_line(&segs, &mut state, &t), "");
    }

    #[test]
    fn single_segment_emits_two_separators() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [Segment::plain("hi")];
        let out = p.render_line(&segs, &mut state, &t);
        let sep_count = out.matches(DEFAULT_SEPARATOR_LEFT).count();
        assert_eq!(sep_count, 2, "expected 2 separators in {out:?}");
        assert!(out.contains("hi"));
    }

    #[test]
    fn three_segments_emit_four_separators() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs = [
            Segment::plain("a"),
            Segment::plain("b"),
            Segment::plain("c"),
        ];
        let out = p.render_line(&segs, &mut state, &t);
        assert_eq!(out.matches(DEFAULT_SEPARATOR_LEFT).count(), 4);
    }

    #[test]
    fn cycles_wrap_around_when_more_segments_than_cycle_len() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        // Dracula has 4 colors; emit 6 segments, last two reuse first two cycle slots.
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let segs: Vec<Segment> = (0..6).map(|i| Segment::plain(format!("s{i}"))).collect();
        let out = p.render_line(&segs, &mut state, &t);
        for i in 0..6 {
            assert!(out.contains(&format!("s{i}")), "missing s{i} in {out:?}");
        }
    }

    #[test]
    fn segment_explicit_bg_overrides_cycle() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::TrueColor, false);
        let style = Style::none().bg(Color::Rgb(123, 45, 67));
        let segs = [Segment::styled("x", style)];
        let out = p.render_line(&segs, &mut state, &t);
        assert!(
            out.contains("48;2;123;45;67"),
            "expected forced bg in {out:?}"
        );
    }

    #[test]
    fn level_none_emits_no_ansi() {
        let mut state = RenderState::default();
        let t = default_theme_config();
        let p = Powerline::new(theme(), ColorLevel::None, false);
        let segs = [Segment::plain("hi")];
        let out = p.render_line(&segs, &mut state, &t);
        assert!(!out.contains('\x1b'), "expected no ANSI: {out:?}");
        assert!(out.contains("hi"));
        assert_eq!(out.matches(DEFAULT_SEPARATOR_LEFT).count(), 2);
    }
}
