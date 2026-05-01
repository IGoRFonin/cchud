//! Powerline themes — Phase 4 Task 7.
//!
//! 5 hardcoded built-ins as `static BuiltinTheme` (refs only — no `LazyLock`).
//! Runtime `PowerlineTheme` is owned (serde-friendly). Lookup converts
//! `BuiltinTheme` → `PowerlineTheme` on demand (5 small Vec allocs per startup).
//!
//! Color values mirror upstream `ccstatusline/src/utils/themes/*.ts`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Color, Style};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerlineTheme {
    pub name: String,
    pub terminal_bg: Color,
    pub default_fg: Color,
    pub default_bg: Color,
    pub bg_cycle: Vec<Color>,
    pub fg_cycle: Vec<Color>,
    #[serde(default)]
    pub widget_styles: BTreeMap<String, Style>,
}

#[derive(Debug)]
pub struct BuiltinTheme {
    pub name: &'static str,
    pub terminal_bg: Color,
    pub default_fg: Color,
    pub default_bg: Color,
    pub bg_cycle: &'static [Color],
    pub fg_cycle: &'static [Color],
    pub widget_styles: &'static [(&'static str, Style)],
}

impl From<&BuiltinTheme> for PowerlineTheme {
    fn from(b: &BuiltinTheme) -> Self {
        Self {
            name: b.name.to_string(),
            terminal_bg: b.terminal_bg,
            default_fg: b.default_fg,
            default_bg: b.default_bg,
            bg_cycle: b.bg_cycle.to_vec(),
            fg_cycle: b.fg_cycle.to_vec(),
            widget_styles: b
                .widget_styles
                .iter()
                .map(|(k, v)| ((*k).to_string(), *v))
                .collect(),
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

// --- DEFAULT ----------------------------------------------------------
pub static DEFAULT: BuiltinTheme = BuiltinTheme {
    name: "default",
    terminal_bg: rgb(0, 0, 0),
    default_fg: rgb(255, 255, 255),
    default_bg: rgb(54, 54, 54),
    bg_cycle: &[rgb(54, 54, 54), rgb(80, 80, 80), rgb(110, 110, 110)],
    fg_cycle: &[rgb(220, 220, 220); 3],
    widget_styles: &[],
};

// --- DRACULA ----------------------------------------------------------
pub static DRACULA: BuiltinTheme = BuiltinTheme {
    name: "dracula",
    terminal_bg: rgb(40, 42, 54),
    default_fg: rgb(248, 248, 242),
    default_bg: rgb(68, 71, 90),
    bg_cycle: &[
        rgb(98, 114, 164),
        rgb(189, 147, 249),
        rgb(255, 121, 198),
        rgb(80, 250, 123),
    ],
    fg_cycle: &[rgb(40, 42, 54); 4],
    widget_styles: &[],
};

// --- SOLARIZED-DARK ---------------------------------------------------
pub static SOLARIZED_DARK: BuiltinTheme = BuiltinTheme {
    name: "solarized-dark",
    terminal_bg: rgb(0, 43, 54),
    default_fg: rgb(238, 232, 213),
    default_bg: rgb(7, 54, 66),
    bg_cycle: &[
        rgb(38, 139, 210),
        rgb(42, 161, 152),
        rgb(133, 153, 0),
        rgb(181, 137, 0),
    ],
    fg_cycle: &[rgb(0, 43, 54); 4],
    widget_styles: &[],
};

// --- NORD -------------------------------------------------------------
pub static NORD: BuiltinTheme = BuiltinTheme {
    name: "nord",
    terminal_bg: rgb(46, 52, 64),
    default_fg: rgb(216, 222, 233),
    default_bg: rgb(59, 66, 82),
    bg_cycle: &[
        rgb(94, 129, 172),
        rgb(129, 161, 193),
        rgb(143, 188, 187),
        rgb(136, 192, 208),
    ],
    fg_cycle: &[rgb(46, 52, 64); 4],
    widget_styles: &[],
};

// --- GRUVBOX-DARK -----------------------------------------------------
pub static GRUVBOX_DARK: BuiltinTheme = BuiltinTheme {
    name: "gruvbox-dark",
    terminal_bg: rgb(40, 40, 40),
    default_fg: rgb(235, 219, 178),
    default_bg: rgb(60, 56, 54),
    bg_cycle: &[
        rgb(204, 36, 29),
        rgb(152, 151, 26),
        rgb(215, 153, 33),
        rgb(69, 133, 136),
    ],
    fg_cycle: &[rgb(40, 40, 40); 4],
    widget_styles: &[],
};

#[must_use]
pub fn lookup(name: &str) -> Option<&'static BuiltinTheme> {
    match name {
        "default" => Some(&DEFAULT),
        "dracula" => Some(&DRACULA),
        "solarized-dark" => Some(&SOLARIZED_DARK),
        "nord" => Some(&NORD),
        "gruvbox-dark" => Some(&GRUVBOX_DARK),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_all_five() {
        for name in [
            "default",
            "dracula",
            "solarized-dark",
            "nord",
            "gruvbox-dark",
        ] {
            assert!(lookup(name).is_some(), "missing builtin: {name}");
        }
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup("monokai").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn all_themes_have_nonempty_cycles() {
        for theme in [&DEFAULT, &DRACULA, &SOLARIZED_DARK, &NORD, &GRUVBOX_DARK] {
            assert!(!theme.bg_cycle.is_empty(), "{} bg_cycle empty", theme.name);
            assert!(!theme.fg_cycle.is_empty(), "{} fg_cycle empty", theme.name);
            assert_eq!(
                theme.bg_cycle.len(),
                theme.fg_cycle.len(),
                "{} cycle length mismatch",
                theme.name
            );
        }
    }

    #[test]
    fn from_builtin_clones_into_owned() {
        let owned: PowerlineTheme = (&DRACULA).into();
        assert_eq!(owned.name, "dracula");
        assert_eq!(owned.bg_cycle.len(), DRACULA.bg_cycle.len());
        assert_eq!(owned.bg_cycle[0], DRACULA.bg_cycle[0]);
    }

    #[test]
    fn powerline_theme_round_trips_via_serde() {
        let owned: PowerlineTheme = (&NORD).into();
        let json = serde_json::to_string(&owned).expect("serialize");
        let back: PowerlineTheme = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.name, owned.name);
        assert_eq!(back.bg_cycle, owned.bg_cycle);
    }

    #[test]
    fn widget_styles_are_empty_for_phase4() {
        // Per spec decision #10/11 — phase 4 keeps widget_styles available but
        // does not populate it for builtins. Custom themes can override.
        for theme in [&DEFAULT, &DRACULA, &SOLARIZED_DARK, &NORD, &GRUVBOX_DARK] {
            assert!(
                theme.widget_styles.is_empty(),
                "{} should not preset widget styles",
                theme.name
            );
        }
    }
}
