//! Excalidraw-style color palette: 5×3 grid of base colors + 5 shades each.
//!
//! Cursor model used by the color picker:
//!   `0..GRID_LEN`  → grid cell (row = idx/COLS, col = idx%COLS)
//!   `GRID_LEN`     → "Default (none)"
//!   `GRID_LEN + 1` → "Custom hex…"

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub struct PaletteEntry {
    pub name: &'static str,
    /// 5 shade hex strings (light → dark). `shades[4]` is the displayed swatch.
    pub shades: [&'static str; 5],
}

impl PaletteEntry {
    #[must_use]
    pub const fn base(&self) -> &'static str {
        self.shades[4]
    }
}

pub const GRID_COLS: usize = 5;
pub const GRID_ROWS: usize = 3;
pub const GRID_LEN: usize = GRID_COLS * GRID_ROWS;
#[allow(dead_code)]
pub const SHADE_COUNT: usize = 5;

pub const IDX_DEFAULT: usize = GRID_LEN;
pub const IDX_CUSTOM: usize = GRID_LEN + 1;
#[allow(dead_code)]
pub const TOTAL_ENTRIES: usize = GRID_LEN + 2;

/// 15 base colors arranged 5 columns × 3 rows.
/// Row 0 = neutrals, Row 1 = cool, Row 2 = warm.
pub static EXCALIDRAW_PALETTE: &[PaletteEntry] = &[
    // Row 0 — neutrals
    PaletteEntry {
        name: "White",
        shades: ["#dee2e6", "#e9ecef", "#f1f3f5", "#f8f9fa", "#ffffff"],
    },
    PaletteEntry {
        name: "Gray",
        shades: ["#f1f3f5", "#dee2e6", "#ced4da", "#868e96", "#adb5bd"],
    },
    PaletteEntry {
        name: "Slate",
        shades: ["#dee2e6", "#adb5bd", "#868e96", "#343a40", "#495057"],
    },
    PaletteEntry {
        name: "Black",
        shades: ["#868e96", "#495057", "#343a40", "#212529", "#1e1e1e"],
    },
    PaletteEntry {
        name: "Bronze",
        shades: ["#e9d8c4", "#d4b59e", "#a87f63", "#5e4538", "#846358"],
    },
    // Row 1 — cool
    PaletteEntry {
        name: "Teal",
        shades: ["#c3fae8", "#63e6be", "#20c997", "#0ca678", "#099268"],
    },
    PaletteEntry {
        name: "Blue",
        shades: ["#d0ebff", "#74c0fc", "#339af0", "#1c7ed6", "#1971c2"],
    },
    PaletteEntry {
        name: "Violet",
        shades: ["#e5dbff", "#b197fc", "#845ef7", "#7048e8", "#6741d9"],
    },
    PaletteEntry {
        name: "Grape",
        shades: ["#f3d9fa", "#e599f7", "#cc5de8", "#ae3ec9", "#9c36b5"],
    },
    PaletteEntry {
        name: "Pink",
        shades: ["#ffdeeb", "#faa2c1", "#f06595", "#d6336c", "#c2255c"],
    },
    // Row 2 — warm
    PaletteEntry {
        name: "Green",
        shades: ["#d3f9d8", "#8ce99a", "#51cf66", "#37b24d", "#2f9e44"],
    },
    PaletteEntry {
        name: "Cyan",
        shades: ["#c5f6fa", "#66d9e8", "#22b8cf", "#1098ad", "#0c8599"],
    },
    PaletteEntry {
        name: "Yellow",
        shades: ["#fff3bf", "#ffe066", "#fcc419", "#f59f00", "#f08c00"],
    },
    PaletteEntry {
        name: "Orange",
        shades: ["#ffe8cc", "#ffc078", "#ff922b", "#f76707", "#e8590c"],
    },
    PaletteEntry {
        name: "Red",
        shades: ["#ffe3e3", "#ffa8a8", "#ff8787", "#fa5252", "#e03131"],
    },
];

/// Find (entry, shade) indices that produce `hex`.
/// Prefers the base shade match; falls back to any shade index.
#[must_use]
pub fn find_by_hex(hex: &str) -> Option<(usize, usize)> {
    for (i, entry) in EXCALIDRAW_PALETTE.iter().enumerate() {
        if entry.shades[4].eq_ignore_ascii_case(hex) {
            return Some((i, 4));
        }
    }
    for (i, entry) in EXCALIDRAW_PALETTE.iter().enumerate() {
        for (s, shade) in entry.shades.iter().enumerate() {
            if shade.eq_ignore_ascii_case(hex) {
                return Some((i, s));
            }
        }
    }
    None
}

/// Best-effort label for a stored hex, e.g. "Red" or "Red · shade 2".
#[must_use]
pub fn label_for_hex(hex: &str) -> Option<String> {
    let (entry, shade) = find_by_hex(hex)?;
    let name = EXCALIDRAW_PALETTE[entry].name;
    if shade == 4 {
        Some(name.to_string())
    } else {
        Some(format!("{name} · shade {}", shade + 1))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn palette_has_15_entries() {
        assert_eq!(EXCALIDRAW_PALETTE.len(), GRID_LEN);
    }

    #[test]
    fn each_entry_has_5_shades() {
        for entry in EXCALIDRAW_PALETTE {
            assert_eq!(entry.shades.len(), SHADE_COUNT);
        }
    }

    #[test]
    fn total_entries_includes_default_and_custom() {
        assert_eq!(TOTAL_ENTRIES, GRID_LEN + 2);
        assert_eq!(IDX_DEFAULT, GRID_LEN);
        assert_eq!(IDX_CUSTOM, GRID_LEN + 1);
    }

    #[test]
    fn find_by_hex_locates_red_base() {
        let (entry, shade) = find_by_hex("#e03131").unwrap();
        assert_eq!(EXCALIDRAW_PALETTE[entry].name, "Red");
        assert_eq!(shade, 4);
    }

    #[test]
    fn find_by_hex_locates_red_lightest_shade() {
        let (entry, shade) = find_by_hex("#ffe3e3").unwrap();
        assert_eq!(EXCALIDRAW_PALETTE[entry].name, "Red");
        assert_eq!(shade, 0);
    }

    #[test]
    fn label_for_unknown_hex_returns_none() {
        assert!(label_for_hex("#123456").is_none());
    }
}
