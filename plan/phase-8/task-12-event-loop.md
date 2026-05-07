# Task 12 — `tui::event` (TerminalGuard + run_event_loop) + `tui::run_configure`

**Цель:** RAII guard для raw-mode + alternate screen, event loop с poll(100ms), интеграция reducer + save. Это последний кусок TUI до commands::configure (T13).

**Files:**
- Modify: `src/tui/event.rs` — наполнить (был skeleton после T1)
- Modify: `src/tui/mod.rs` — обновить `run_configure` body (заменить заглушку из T1)

---

- [ ] **Step 1: `TerminalGuard` + `run_event_loop`**

В `src/tui/event.rs`:

```rust
//! TerminalGuard + event loop — Phase 8 Task 12.

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

/// RAII: enable raw mode + EnterAlternateScreen в `enter()`. Drop восстанавливает.
pub struct TerminalGuard;

impl TerminalGuard {
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
pub fn run_event_loop(mut app: App) -> io::Result<bool> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let key = match event::read()? {
            Event::Key(k) => k,
            _ => continue,
        };
        match reducer::handle_key(&mut app, key) {
            ReducerEffect::None | ReducerEffect::RebuildPreview => continue,
            ReducerEffect::Quit => return Ok(false),
            ReducerEffect::RequestSaveAndQuit => {
                let saved = perform_save(&mut app)?;
                return Ok(saved);
            }
            ReducerEffect::RequestDiscardAndQuit => {
                app.discard();
                return Ok(false);
            }
        }
    }
}

fn perform_save(app: &mut App) -> io::Result<bool> {
    let dest = config_destination();
    match save::atomic_save(&app.editable, &dest) {
        Ok(backup) => {
            let msg = match backup {
                Some(b) => format!("Saved · backup at {}", b.display()),
                None => format!("Saved · {}", dest.display()),
            };
            app.status_message = Some((msg, MessageKind::Info));
            app.mark_saved();
            Ok(true)
        }
        Err(e) => {
            app.status_message = Some((
                format!("Save failed: {e}"),
                MessageKind::Error,
            ));
            // User остаётся в TUI — может попробовать ещё раз.
            app.mode = Mode::Edit;
            Ok(false)
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
```

- [ ] **Step 2: Обновить `tui::mod::run_configure`**

В `src/tui/mod.rs` заменить body на:

```rust
pub fn run_configure(
    settings: crate::types::config::Settings,
    sample: crate::types::payload::StatusPayload,
    transcript: Option<tempfile::NamedTempFile>,
) -> std::io::Result<bool> {
    let app = app::App::new(settings, sample, transcript);
    event::run_event_loop(app)
}
```

(Уже было такое — но в T1 я задал заглушку body. Если в T1 уже именно этот код, оставить.)

- [ ] **Step 3: Build**

```bash
cargo build --features tui --locked
cargo build --locked --no-default-features
```

Expected: оба PASS. TUI теперь компилируется end-to-end.

- [ ] **Step 4: Lints**

```bash
cargo clippy --features tui --locked --all-targets -- -D warnings
```

Expected: PASS. Если `clippy::missing_errors_doc` — позволь `#[allow]` или добавить `# Errors` блок к `run_event_loop`.

- [ ] **Step 5: Smoke test (обернуть в integration test позже в T14)**

Без TTY guard невозможно запустить TUI; пока просто verify — биты компилируются и `cargo test` зелёный.

```bash
cargo test --features tui --locked
```

Expected: PASS на всех existing tests.

- [ ] **Step 6: Commit**

```bash
git add src/tui/event.rs src/tui/mod.rs
git commit -m "$(cat <<'EOF'
feat(phase-8): T12 event loop — TerminalGuard + run_event_loop + run_configure

- TerminalGuard: RAII enable_raw_mode + EnterAlternateScreen; Drop restores
- run_event_loop: terminal.draw + poll(100ms) + reducer::handle_key + effect dispatch
- ReducerEffect dispatch: None/RebuildPreview→continue, Quit→Ok(false),
  RequestSaveAndQuit→atomic_save+mark_saved, RequestDiscardAndQuit→discard
- perform_save: writes to ~/.config/cchud/settings.json (or $CCHUD_CONFIG),
  status_message Info on success / Error on IO failure (stays in TUI)
- run_configure now wires App::new + event loop end-to-end

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
