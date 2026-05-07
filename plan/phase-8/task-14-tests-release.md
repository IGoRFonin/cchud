# Task 14 — Snapshots / import / no-TTY tests + bench + release 0.9.0

**Цель:** Финал Phase 8. ≥6 ratatui TestBackend snapshot тестов, ≥6 import scenarios, configure-no-TTY smoke, hyperfine cold-start bench, version bump 0.5.0 → 0.9.0, CHANGELOG, manual battle-test, tag.

**Files:**
- Create: `tests/tui_snapshots.rs`
- Create: `tests/import_tests.rs`
- Create: `tests/configure_no_tty.rs`
- Create: `tests/configs/import-full-cc.json` (fixture)
- Create: `tests/configs/import-with-section.json` (fixture)
- Create: `tests/configs/import-malformed.json` (fixture)
- Create: `tests/configs/import-unknown-widgets.json` (fixture)
- Create: `tests/configs/import-empty-section.json` (fixture)
- Create: `benches/phase-8.md`
- Create: `plan/phase-8/manual-test-log.md`
- Modify: `Cargo.toml:3` — `version = "0.9.0"`
- Modify: `CHANGELOG.md` — добавить запись 0.9.0
- Modify: `README.md` — TUI секция

---

- [ ] **Step 1: `tests/tui_snapshots.rs` — ≥6 ratatui snapshot тестов**

```rust
//! Phase 8 TUI snapshot suite — `ratatui::backend::TestBackend(80, 24)`.

#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use cchud::tui::app::{App, EditField, Mode, Pane, SettingsField, ColorField};
use cchud::tui::sample;
use cchud::tui::ui;
use cchud::types::config::Settings;

fn fresh_app() -> App {
    let (p, f) = sample::payload();
    let json = r#"{"lines":[{"widgets":[{"type":"model"}]}]}"#;
    let s: Settings = serde_json::from_str(json).unwrap();
    App::new(s, p, f)
}

fn render_to_buffer(app: &App) -> String {
    let backend = TestBackend::new(80, 24);
    let mut term = Terminal::new(backend).unwrap();
    term.draw(|f| ui::draw(f, app)).unwrap();
    format!("{:?}", term.backend().buffer())
}

#[test]
fn initial_state() {
    let app = fresh_app();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn palette_filtered_by_git() {
    let mut app = fresh_app();
    app.focus = Pane::Palette;
    app.palette_filter = "git".into();
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn settings_with_color_picker_open() {
    let mut app = fresh_app();
    app.focus = Pane::Settings;
    app.settings_field_cursor = 0; // Color row
    app.editing_field = Some(EditField::ColorHex {
        field: ColorField::Foreground,
        buffer: "#aabbcc".into(),
        cursor: 7,
    });
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn themes_overlay_open() {
    let mut app = fresh_app();
    app.mode = Mode::ThemesOverlay;
    app.theme_field_cursor = 1; // Dracula
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn help_overlay_open() {
    let mut app = fresh_app();
    app.mode = Mode::HelpOverlay;
    insta::assert_snapshot!(render_to_buffer(&app));
}

#[test]
fn confirm_quit_modal_when_dirty() {
    use cchud::types::config::Line;
    let mut app = fresh_app();
    app.editable.lines.push(Line::default());
    app.mode = Mode::ConfirmQuit;
    insta::assert_snapshot!(render_to_buffer(&app));
}
```

- [ ] **Step 2: Создать import test fixtures**

`tests/configs/import-full-cc.json`:
```json
{
  "ccstatusline": {
    "version": 1,
    "lines": [{"widgets": [{"type": "model"}, {"type": "git-branch"}]}],
    "theme": {"kind": "powerline", "theme_name": "dracula"}
  },
  "statusLine": {"type": "command", "command": "/usr/local/bin/old"}
}
```

`tests/configs/import-with-section.json`:
```json
{
  "ccstatusline": {
    "lines": [{"widgets": [{"type": "session-cost"}, {"type": "context-percentage"}]}]
  }
}
```

`tests/configs/import-malformed.json`:
```
{not valid json
```

`tests/configs/import-unknown-widgets.json`:
```json
{
  "lines": [{"widgets": [{"type": "model"}, {"type": "ccstatusline-fake-1"}, {"type": "ccstatusline-fake-2"}]}]
}
```

`tests/configs/import-empty-section.json`:
```json
{
  "ccstatusline": {"lines": [{"widgets": [{"type": "ccstatusline-fake-1"}]}]}
}
```

- [ ] **Step 3: `tests/import_tests.rs` — ≥6 scenarios**

```rust
#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn cchud_with_dest(dest: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("cchud").unwrap();
    cmd.env("CCHUD_CONFIG", dest);
    cmd
}

#[test]
fn imports_full_ccstatusline_config() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-full-cc.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("widgets across"));
    assert!(dest.exists());
}

#[test]
fn import_missing_source_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "/nonexistent/__nope.json"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn import_malformed_json_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "tests/configs/import-malformed.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not valid JSON"));
}

#[test]
fn import_unknown_widgets_warns_but_succeeds() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "tests/configs/import-unknown-widgets.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ccstatusline-fake-1"))
        .stderr(predicate::str::contains("ccstatusline-fake-2"));
}

#[test]
fn import_empty_section_exits_1() {
    let dir = tempdir().unwrap();
    cchud_with_dest(&dir.path().join("settings.json"))
        .args(["import", "--from", "tests/configs/import-empty-section.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nothing imported"));
}

#[test]
fn import_existing_dest_without_force_exits_1() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    std::fs::write(&dest, "{}").unwrap();

    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-full-cc.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn import_existing_dest_with_force_overwrites_and_backs_up() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("settings.json");
    std::fs::write(&dest, "{}").unwrap();

    cchud_with_dest(&dest)
        .args(["import", "--from", "tests/configs/import-full-cc.json", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backup at"));

    // Backup file должен начинаться с `<dest>.bak.`
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains(".bak."))
        .collect();
    assert!(!entries.is_empty(), "expected at least one .bak.* file");
}
```

- [ ] **Step 4: `tests/configure_no_tty.rs`**

```rust
#![cfg(all(test, feature = "tui"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn configure_without_tty_exits_2_with_message() {
    Command::cargo_bin("cchud")
        .unwrap()
        .arg("configure")
        .write_stdin("")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("must be run in a terminal"));
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test --features tui --locked
cargo test --locked --no-default-features
```

Expected: оба PASS. ≥220 tests суммарно. Snapshot тесты в `tests/tui_snapshots.rs` потребуют первый прогон + `cargo insta accept` (zero-baseline → принимает baseline).

```bash
cargo insta accept
```

(Только если это первый прогон — после baseline `cargo test` зелёный без `accept`.)

- [ ] **Step 6: `benches/phase-8.md` + hyperfine cold-start**

```bash
mkdir -p benches/results
hyperfine --warmup 3 --runs 50 \
  'printf "" | ./target/release/cchud configure' \
  --export-json benches/results/phase-8.json
```

Note: команда попадает в no-TTY guard и измеряет только cold-start (~ parse settings + sample::payload + tempfile creation). Цель — < 50 ms p95.

`benches/phase-8.md`:
```markdown
# Phase 8 — bench results

## cchud configure cold-start (no-TTY guard)

| Run                                       | Mean (ms) | p95 (ms) |
|-------------------------------------------|-----------|----------|
| `printf '' \| cchud configure`            | <fill from benches/results/phase-8.json> | <fill> |

NFR: < 50 ms p95 — config load + sample::payload + tempfile + IsTerminal check.

Запуск:
```
hyperfine --warmup 3 --runs 50 'printf "" | ./target/release/cchud configure'
```
```

(Заполнить таблицу после прогона.)

- [ ] **Step 7: Manual battle-test (5+ мин)**

В реальной CC-сессии запустить `cchud configure` и пройти сценарии:
- Add 5 widgets из палитры (filter `git`, `session`, `context`).
- Reorder Alt+↑↓.
- Edit color через picker (Default → custom hex).
- Open Themes (`t`), переключить Dracula.
- Help overlay (`?`).
- Save (Ctrl+S), проверить `~/.config/cchud/settings.json`, backup создан.
- Quit с dirty — проверить modal `[s/d/c]`.
- Re-open, проверить что settings загружены.
- `cchud import --from /path/to/ccstatusline-config.json --then-configure` → smoke.

Записать наблюдения в `plan/phase-8/manual-test-log.md`:
```markdown
# Phase 8 Manual Test Log — 2026-05-XX

## Sessions
- [date] · 5+ мин · macOS 14 · iTerm2

## Configs tested
- (заполнить)

## Observations
- (заполнить)

## Issues found
- (заполнить или None)
```

- [ ] **Step 8: Bump version 0.5.0 → 0.9.0**

В `Cargo.toml:3`:
```toml
version = "0.9.0"
```

Run:
```bash
cargo build --release --locked
./target/release/cchud --version
# Expected: 0.9.0
```

- [ ] **Step 9: Update CHANGELOG.md**

Добавить в начало (после `## [Unreleased]`):

```markdown
## [0.9.0] — 2026-05-XX

**Phase 8: TUI Configurator + Import.** Interactive TUI на ratatui+crossterm + CLI миграция с ccstatusline.

### Added

- `cchud configure` — interactive TUI configurator (4 panels: Lines / Palette / Settings / Preview).
- `cchud import [--from <path>] [--then-configure] [--force]` — best-effort миграция с ccstatusline. Авто-детект секции `ccstatusline` в `~/.claude/settings.json`.
- 60-widget palette с filter (`/`) + 11 категорий (Model/Static/Trivial/Session/Context/Worktree/Git/Transcript/Usage/Env).
- Per-widget overrides через TUI: color picker (16 ANSI named + Custom hex), background_color, tri-state bold.
- Themes overlay (`t`): 5 builtins + 9 globals.
- Help overlay (`?`): cheatsheet keybindings.
- Confirm-quit modal `[s/d/c]` для dirty changes.
- Atomic save с backup `<path>.bak.<unix-ts-ms>`.
- Feature flag `tui` (default = ["tui"]); `cargo install cchud --no-default-features` собирает минимальный бинарь.

### Changed

- **Refactor `Renderer::compose_line` extract** — pure-функция возвращает `Vec<StyledSegment>`. `Renderer::render_line = compose_line + emit_ansi` (общий ANSI emitter). TUI live preview переиспользует `compose_line + style_map::to_span`.
- `Renderer::for_preview(&Settings)` (cfg tui) — конструктор forces TrueColor + hyperlinks=false.
- `PartialEq, Eq` derived на Settings/Line/WidgetItem/WidgetConfig/ThemeConfig/PowerlineTheme/всех Params (`App.dirty()` через `==`).

### Performance

- TUI cold-start: < 50 ms p95 (config load + sample payload + tempfile creation).
- Hot path рендера (`cchud` без аргументов) — без изменений: cchud-8w < 5 ms p95 / cchud-60w < 12 ms p95.
- Phase 4 + Phase 7 + git + transcript snapshots — byte-identical после refactor (no regression).

### Dependencies

- Added (optional, feature `tui`): `ratatui = "=0.30.0"` (crossterm backend), `crossterm = "=0.29.0"`, `tempfile = "3"` (поднято из dev-dependencies).
```

- [ ] **Step 10: Update README.md** — добавить секцию TUI и `cchud configure` / `cchud import`.

- [ ] **Step 11: Final verify**

```bash
cargo build --release --locked
cargo build --release --locked --no-default-features
cargo test --locked
cargo test --locked --no-default-features
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo fmt --check
du -h target/release/cchud
```

Expected:
- All builds PASS.
- All tests PASS (≥220 суммарно).
- Default binary < 10 MB.
- No-default-features binary < 8.5 MB.
- Phase 4 + Phase 7 snapshots byte-identical.

- [ ] **Step 12: Commit + tag + push**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md \
        tests/tui_snapshots.rs tests/import_tests.rs tests/configure_no_tty.rs \
        tests/configs/import-*.json tests/snapshots/ \
        benches/phase-8.md benches/results/phase-8.json \
        plan/phase-8/manual-test-log.md plan/README.md
git commit -m "$(cat <<'EOF'
release: 0.9.0 — Phase 8 (TUI configurator + import + compose_line refactor)

- cchud configure: 4-panel ratatui TUI (Lines/Palette/Settings/Preview)
- cchud import: best-effort ccstatusline migration (--from / --then-configure / --force)
- Renderer::compose_line refactor: pure Vec<StyledSegment>; Phase 4 + Phase 7 snapshots byte-identical
- Feature flag tui (default); --no-default-features build < 8.5 MB
- ≥6 ratatui snapshot tests, ≥7 import scenarios, configure-no-TTY smoke
- Manual battle-test logged in plan/phase-8/manual-test-log.md

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
git tag v0.9.0
git push origin main
git push origin v0.9.0
```

Expected: CI matrix зелёный (macos / ubuntu / windows). GitHub Release создаётся автоматически (если настроен release-on-tag workflow).
