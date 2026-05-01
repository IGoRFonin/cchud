//! `TerminalGuard` + event loop — Phase 8 Task 12.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::tui::app::{App, MessageKind, Mode};
use crate::tui::effects::ReducerEffect;
use crate::tui::{reducer, save, ui};

/// RAII: enable raw mode + `EnterAlternateScreen` в `enter()`. `Drop` восстанавливает.
pub struct TerminalGuard;

impl TerminalGuard {
    /// # Errors
    /// Returns `Err` if raw mode or alternate screen cannot be enabled.
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// Запускает TUI loop. Возвращает `Ok(true)` если пользователь сохранил изменения,
/// `Ok(false)` если discard или quit без save.
///
/// # Errors
/// Returns `Err` if the terminal backend fails to initialize or an I/O error occurs.
pub fn run_event_loop(mut app: App) -> io::Result<bool> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else { continue };
        match reducer::handle_key(&mut app, key) {
            ReducerEffect::None | ReducerEffect::RebuildPreview => {}
            ReducerEffect::Quit => return Ok(false),
            ReducerEffect::RequestSaveAndQuit => {
                let saved = perform_save(&mut app);
                return Ok(saved);
            }
            ReducerEffect::RequestDiscardAndQuit => {
                app.discard();
                return Ok(false);
            }
        }
    }
}

fn perform_save(app: &mut App) -> bool {
    let dest = config_destination();
    match save::atomic_save(&app.editable, &dest) {
        Ok(backup) => {
            let msg = backup.map_or_else(
                || format!("Saved · {}", dest.display()),
                |b| format!("Saved · backup at {}", b.display()),
            );
            app.status_message = Some((msg, MessageKind::Info));
            app.mark_saved();
            true
        }
        Err(e) => {
            app.status_message = Some((format!("Save failed: {e}"), MessageKind::Error));
            // User остаётся в TUI — может попробовать ещё раз.
            app.mode = Mode::Edit;
            false
        }
    }
}

fn config_destination() -> PathBuf {
    if let Ok(p) = std::env::var("CCHUD_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/cchud/settings.json")
}
