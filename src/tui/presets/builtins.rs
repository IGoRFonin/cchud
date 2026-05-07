//! Built-in presets — 5 entries embedded as JSON.
#![deny(clippy::unwrap_used)]

use std::sync::LazyLock;

use super::PresetData;

const MINIMAL: &str = include_str!("./builtins/minimal.json");
const DEV: &str = include_str!("./builtins/dev.json");
const GIT_HEAVY: &str = include_str!("./builtins/git-heavy.json");
const POWERLINE_DRACULA: &str = include_str!("./builtins/powerline-dracula.json");
const POWERLINE_GRUVBOX: &str = include_str!("./builtins/powerline-gruvbox.json");

#[allow(clippy::expect_used)]
fn parse(json: &str) -> PresetData {
    // expect — оправдано: built-in JSON статический и проходит CI-тест
    // builtins_all_parse_without_panic.
    serde_json::from_str(json).expect("built-in preset JSON must parse — bug if it doesn't")
}

pub static ALL: LazyLock<[(&'static str, PresetData); 5]> = LazyLock::new(|| {
    [
        ("minimal", parse(MINIMAL)),
        ("dev", parse(DEV)),
        ("git-heavy", parse(GIT_HEAVY)),
        ("powerline-dracula", parse(POWERLINE_DRACULA)),
        ("powerline-gruvbox", parse(POWERLINE_GRUVBOX)),
    ]
});
