//! Phase 7: parse hex/ANSI-256 colors from per-widget overrides + theme globals.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use super::Color;

/// Парсит `#rrggbb` или ANSI-256 имя (`bright_red`, `cyan`, ...). None при невалидном вводе.
#[must_use]
pub fn parse_color(s: Option<&str>) -> Option<Color> {
    let s = s?.trim();
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    parse_ansi_named(s)
}

fn parse_hex(hex: &str) -> Option<Color> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn parse_ansi_named(name: &str) -> Option<Color> {
    let n = match name.to_ascii_lowercase().as_str() {
        "black" => 0,
        "red" => 1,
        "green" => 2,
        "yellow" => 3,
        "blue" => 4,
        "magenta" => 5,
        "cyan" => 6,
        "white" => 7,
        "bright_black" | "gray" | "grey" => 8,
        "bright_red" => 9,
        "bright_green" => 10,
        "bright_yellow" => 11,
        "bright_blue" => 12,
        "bright_magenta" => 13,
        "bright_cyan" => 14,
        "bright_white" => 15,
        _ => return None,
    };
    Some(Color::Ansi256(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parses() {
        assert_eq!(parse_color(Some("#fafafa")), Some(Color::Rgb(0xfa, 0xfa, 0xfa)));
        assert_eq!(parse_color(Some("#000000")), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn ansi_named_parses() {
        assert_eq!(parse_color(Some("red")), Some(Color::Ansi256(1)));
        assert_eq!(parse_color(Some("bright_red")), Some(Color::Ansi256(9)));
        assert_eq!(parse_color(Some("CYAN")), Some(Color::Ansi256(6)));
    }

    #[test]
    fn invalid_returns_none() {
        assert!(parse_color(Some("monokai")).is_none());
        assert!(parse_color(Some("#zz0000")).is_none());
        assert!(parse_color(Some("#fff")).is_none()); // short hex not supported
        assert!(parse_color(None).is_none());
    }
}
