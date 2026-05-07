//! TUI configurator.
//!
//! Все модули под `#[cfg(feature = "tui")]`. Hot path рендера (`cchud` без аргументов)
//! не зависит от этого модуля. См. `docs/superpowers/specs/2026-05-01-phase-8-tui-design.md`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod app;
pub mod effects;
pub mod event;
pub mod overlays;
pub mod panels;
pub mod reducer;
pub mod sample;
pub mod save;
pub mod style_map;
pub mod ui;
pub mod widget_meta;
pub mod widgets_ui;

#[allow(unused_imports)]
pub use event::run_event_loop;

/// Public entry — вызывается из `commands::configure::run`.
/// Возвращает `Ok(true)` если user сохранил, `Ok(false)` если discard или quit без save.
///
/// # Errors
/// Returns `Err` if the terminal backend fails to initialize or an I/O error occurs.
pub fn run_configure(
    settings: crate::types::config::Settings,
    sample: crate::types::payload::StatusPayload,
    transcript: Option<tempfile::NamedTempFile>,
) -> std::io::Result<bool> {
    let app = app::App::new(settings, sample, transcript);
    event::run_event_loop(app)
}
