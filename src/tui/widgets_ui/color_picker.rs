//! Color picker palette — list of named ANSI colors used by `panels::settings`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub static NAMED_COLORS: &[(&str, &str)] = &[
    ("Default (none)", ""),
    ("Black", "#000000"),
    ("Red", "#cd0000"),
    ("Green", "#00cd00"),
    ("Yellow", "#cdcd00"),
    ("Blue", "#0000ee"),
    ("Magenta", "#cd00cd"),
    ("Cyan", "#00cdcd"),
    ("White", "#e5e5e5"),
    ("Bright Black", "#7f7f7f"),
    ("Bright Red", "#ff0000"),
    ("Bright Green", "#00ff00"),
    ("Bright Yellow", "#ffff00"),
    ("Bright Blue", "#5c5cff"),
    ("Bright Magenta", "#ff00ff"),
    ("Bright Cyan", "#00ffff"),
    ("Bright White", "#ffffff"),
    ("Custom hex…", ""),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_colors_has_18_entries() {
        // 16 ANSI + Default + Custom hex
        assert_eq!(NAMED_COLORS.len(), 18);
    }
}
