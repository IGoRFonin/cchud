# Task 3 — Trivial cluster (Version, ClaudeSessionId, TerminalWidth, OutputStyle, VimMode, SessionName)

**Files:**
- Create: `src/widgets/trivial.rs` (5 widgets: `Version`, `ClaudeSessionId`, `TerminalWidth`, `OutputStyle`, `VimMode`)
- Create: `src/widgets/session.rs` (1 widget in T3: `SessionName`; T5 добавит `SessionClock`, `SessionCost`)
- Modify: `src/widgets/mod.rs` (`pub mod trivial; pub mod session;`; 6 match-arms заменяют `Stub` на реальные impl)

## Goal

Шесть простых "однополевых" виджетов — каждый читает 1 поле payload и кратко его форматирует.

| Widget | Source | Render | Edge |
|---|---|---|---|
| `Version` | `payload.version` | `Some(v.clone())` | None если поле отсутствует |
| `ClaudeSessionId` | `payload.session_id` | первые 8 символов | session_id всегда есть (required в `StatusPayload`) |
| `TerminalWidth` | `terminal_size::terminal_size()` | `Some(format!("{}", w.0))` | None если no TTY (e.g. `cargo test`) |
| `OutputStyle` | `payload.output_style?.name` | `Some(name.clone())` | None если field отсутствует или пустой |
| `VimMode` | `payload.vim?.mode` | `Some(mode.clone())` | None если vim не включён или mode пуст |
| `SessionName` | `payload.transcript_path` | basename без `.jsonl` | None если `transcript_path: None` |

`SessionName` — кладём в `widgets/session.rs` потому что T5 добавит `SessionClock` и `SessionCost` в тот же файл (один кластер). Остальные 5 — в `widgets/trivial.rs`.

`ClaudeSessionId` — берёт первые 8 ASCII-символов через `chars().take(8).collect()`. Не используем `&s[..8]` — `session_id` приходит как UUID (ASCII), но защита от non-ASCII через `chars()` копеечная.

## Inputs

- T1, T2 закрыты.
- `terminal_size` 0.4 уже в `Cargo.toml` (Phase 1).
- `tests/snapshots.rs` НЕ включает `TerminalWidth` (детерминизм CI vs local).

---

- [ ] **Step 1: Написать failing-тесты для `trivial.rs` (5 виджетов)**

Create `/Users/igor/mp/startup/cchud/src/widgets/trivial.rs`:

```rust
//! Trivial single-field widgets — Phase 3 Task 3.
//!
//! Каждый виджет читает одно поле payload (или одно env-значение) и
//! форматирует его минимально. Все impl следуют единой схеме:
//! `payload.foo.as_ref()?.bar.as_deref()?.into()`.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct Version;

impl Widget for Version {
    fn id(&self) -> &'static str {
        "Version"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let v = ctx.payload.version.as_deref()?;
        if v.is_empty() {
            None
        } else {
            Some(v.to_string())
        }
    }
}

pub struct ClaudeSessionId;

impl Widget for ClaudeSessionId {
    fn id(&self) -> &'static str {
        "ClaudeSessionId"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let id = &ctx.payload.session_id;
        if id.is_empty() {
            return None;
        }
        let short: String = id.chars().take(8).collect();
        Some(short)
    }
}

pub struct TerminalWidth;

impl Widget for TerminalWidth {
    fn id(&self) -> &'static str {
        "TerminalWidth"
    }
    fn render(&self, _ctx: &RenderContext<'_>) -> Option<String> {
        let (terminal_size::Width(w), _h) = terminal_size::terminal_size()?;
        Some(format!("{w}"))
    }
}

pub struct OutputStyle;

impl Widget for OutputStyle {
    fn id(&self) -> &'static str {
        "OutputStyle"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let name = ctx.payload.output_style.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
}

pub struct VimMode;

impl Widget for VimMode {
    fn id(&self) -> &'static str {
        "VimMode"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let mode = ctx.payload.vim.as_ref()?.mode.as_deref()?;
        if mode.is_empty() {
            None
        } else {
            Some(mode.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{
        ModelInfo, OutputStyle as PayloadOutputStyle, StatusPayload, VimState, Workspace,
    };

    fn base_payload() -> StatusPayload {
        StatusPayload {
            session_id: "abcd1234-5678-9012-3456-789012345678".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: None,
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    fn ctx_with<'a>(p: &'a StatusPayload, s: &'a crate::types::config::Settings) -> RenderContext<'a> {
        RenderContext::new(p, s)
    }

    #[test]
    fn version_renders_when_present() {
        let mut p = base_payload();
        p.version = Some("2.1.119".into());
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), Some("2.1.119".into()));
    }

    #[test]
    fn version_returns_none_when_missing() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn version_returns_none_when_empty_string() {
        let mut p = base_payload();
        p.version = Some(String::new());
        let s = default_line();
        assert_eq!(Version.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn claude_session_id_takes_first_eight_chars() {
        let p = base_payload();
        let s = default_line();
        // "abcd1234-5678-..." → "abcd1234"
        assert_eq!(
            ClaudeSessionId.render(&ctx_with(&p, &s)),
            Some("abcd1234".into())
        );
    }

    #[test]
    fn claude_session_id_returns_none_for_empty() {
        let mut p = base_payload();
        p.session_id = String::new();
        let s = default_line();
        assert_eq!(ClaudeSessionId.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn claude_session_id_handles_short_id() {
        let mut p = base_payload();
        p.session_id = "abc".into();
        let s = default_line();
        // < 8 символов — берём всё, что есть.
        assert_eq!(ClaudeSessionId.render(&ctx_with(&p, &s)), Some("abc".into()));
    }

    #[test]
    fn terminal_width_returns_none_under_cargo_test() {
        let p = base_payload();
        let s = default_line();
        // cargo test обычно НЕ имеет TTY; ожидаем None.
        // Если CI прогоняет под TTY (редкий случай) — тест пройдёт и для Some(_),
        // потому проверяем "либо None, либо Some(>0)".
        let out = TerminalWidth.render(&ctx_with(&p, &s));
        match out {
            None => {} // expected path under cargo test
            Some(s) => {
                let n: u32 = s.parse().expect("width must be numeric");
                assert!(n > 0, "if Some, width must be > 0");
            }
        }
    }

    #[test]
    fn output_style_renders_name() {
        let mut p = base_payload();
        p.output_style = Some(PayloadOutputStyle {
            name: Some("default".into()),
        });
        let s = default_line();
        assert_eq!(
            OutputStyle.render(&ctx_with(&p, &s)),
            Some("default".into())
        );
    }

    #[test]
    fn output_style_returns_none_without_field() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(OutputStyle.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn output_style_returns_none_with_empty_name() {
        let mut p = base_payload();
        p.output_style = Some(PayloadOutputStyle {
            name: Some(String::new()),
        });
        let s = default_line();
        assert_eq!(OutputStyle.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn vim_mode_renders_when_present() {
        let mut p = base_payload();
        p.vim = Some(VimState {
            mode: Some("INSERT".into()),
        });
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), Some("INSERT".into()));
    }

    #[test]
    fn vim_mode_returns_none_when_disabled() {
        let p = base_payload();
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), None);
    }

    #[test]
    fn vim_mode_returns_none_for_empty_object() {
        let mut p = base_payload();
        p.vim = Some(VimState { mode: None });
        let s = default_line();
        assert_eq!(VimMode.render(&ctx_with(&p, &s)), None);
    }
}
```

- [ ] **Step 2: Написать failing-тесты для `session.rs` (SessionName)**

Create `/Users/igor/mp/startup/cchud/src/widgets/session.rs`:

```rust
//! Session-cluster widgets — Phase 3 Task 3 (SessionName) + Task 5
//! (SessionClock, SessionCost).
//!
//! Объединены в один файл, потому что все три читают `payload.session_id`,
//! `payload.transcript_path` или `payload.cost` — общий контекст сессии.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use crate::widgets::{RenderContext, Widget};

pub struct SessionName;

impl Widget for SessionName {
    fn id(&self) -> &'static str {
        "SessionName"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let path = ctx.payload.transcript_path.as_deref()?;
        let stem = Path::new(path).file_stem()?.to_str()?;
        if stem.is_empty() {
            None
        } else {
            Some(stem.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_with_transcript(path: Option<&str>) -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            workspace: Workspace {
                current_dir: "/tmp".into(),
                project_dir: None,
                added_dirs: None,
            },
            transcript_path: path.map(String::from),
            cwd: None,
            version: None,
            fast_mode: None,
            exceeds_200k_tokens: None,
            output_style: None,
            cost: None,
            context_window: None,
            worktree: None,
            vim: None,
            rate_limits: None,
            effort: None,
            thinking: None,
        }
    }

    #[test]
    fn session_name_strips_dot_jsonl() {
        let p = payload_with_transcript(Some(
            "/Users/igor/.claude/projects/-tmp/abc-123-def-456.jsonl",
        ));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(
            SessionName.render(&ctx),
            Some("abc-123-def-456".into())
        );
    }

    #[test]
    fn session_name_returns_none_when_path_absent() {
        let p = payload_with_transcript(None);
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert_eq!(SessionName.render(&ctx), None);
    }

    #[test]
    fn session_name_handles_path_without_extension() {
        let p = payload_with_transcript(Some("/tmp/abc"));
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        // file_stem на пути без extension возвращает basename
        assert_eq!(SessionName.render(&ctx), Some("abc".into()));
    }
}
```

- [ ] **Step 3: Запустить — должны упасть на компиляции**

```bash
cargo test --locked --lib widgets::trivial 2>&1 | head -10
cargo test --locked --lib widgets::session 2>&1 | head -10
```

Expected: оба `error[E0583]: file not found for module ...` (модули не подключены).

- [ ] **Step 4: Подключить модули + заменить 6 stub'ов в `build_one`**

Edit `src/widgets/mod.rs`:

Edit 1 (mod declarations):
- `old_string`: `pub mod model;\npub mod static_text;`
- `new_string`: `pub mod model;\npub mod session;\npub mod static_text;\npub mod trivial;`
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Edit 2 (заменить 6 Stub-arm на реальные impl):
- `old_string`:
  ```rust
          WidgetConfig::Version => Box::new(Stub("Version")),
          WidgetConfig::ClaudeSessionId => Box::new(Stub("ClaudeSessionId")),
          WidgetConfig::TerminalWidth => Box::new(Stub("TerminalWidth")),
          WidgetConfig::OutputStyle => Box::new(Stub("OutputStyle")),
          WidgetConfig::VimMode => Box::new(Stub("VimMode")),
          WidgetConfig::SessionName => Box::new(Stub("SessionName")),
  ```
- `new_string`:
  ```rust
          // Phase 3 — Task 3 (trivial cluster):
          WidgetConfig::Version => Box::new(trivial::Version),
          WidgetConfig::ClaudeSessionId => Box::new(trivial::ClaudeSessionId),
          WidgetConfig::TerminalWidth => Box::new(trivial::TerminalWidth),
          WidgetConfig::OutputStyle => Box::new(trivial::OutputStyle),
          WidgetConfig::VimMode => Box::new(trivial::VimMode),
          WidgetConfig::SessionName => Box::new(session::SessionName),
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 5: Запустить тесты — должны быть зелёные**

```bash
cargo test --locked --lib widgets::trivial widgets::session
```

Expected:
- `widgets::trivial::tests::*` — 13 тестов passed
- `widgets::session::tests::*` — 3 теста passed

Если `version_renders_when_present` падает — проверить, что в `base_payload()` поле `version` именно `Option<String>`, не что-то ещё.

Если `output_style_renders_name` падает на компиляции из-за импорта `OutputStyle as PayloadOutputStyle` — это псевдоним, чтобы избежать конфликта с widget'ом `OutputStyle` в этом же файле; убедиться что импорт `crate::types::payload::OutputStyle as PayloadOutputStyle` валиден (типы существуют после T1).

- [ ] **Step 6: Smoke-тест с реальным семплом**

```bash
cat benches/samples/payload-cchud-sonnet-xlarge.json | cargo run --release 2>/dev/null
```

Expected: `Sonnet 4.6` (default-line, рендер не сломан).

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Тестов суммарно ≥52.

Возможные clippy:
- `pedantic::module_name_repetitions` на `OutputStyle` widget'е (имя совпадает с модулем `output_style` нет — модуль `trivial`). ОК.
- `pedantic::ref_option` на `&self` — игнорируем.

- [ ] **Step 8: Verification — task-specific gate**

```bash
grep -c 'pub struct Version' src/widgets/trivial.rs
grep -c 'pub struct ClaudeSessionId' src/widgets/trivial.rs
grep -c 'pub struct TerminalWidth' src/widgets/trivial.rs
grep -c 'pub struct OutputStyle' src/widgets/trivial.rs
grep -c 'pub struct VimMode' src/widgets/trivial.rs
grep -c 'pub struct SessionName' src/widgets/session.rs
grep -c 'trivial::Version' src/widgets/mod.rs
grep -c 'session::SessionName' src/widgets/mod.rs
```

Expected: каждая команда → `1`.

- [ ] **Step 9: Commit**

```bash
git add src/widgets/trivial.rs src/widgets/session.rs src/widgets/mod.rs
git commit -m "feat(phase-3): T3 trivial cluster + SessionName

Six single-field widgets:
- Version reads payload.version
- ClaudeSessionId takes first 8 chars of session_id (UUID-style)
- TerminalWidth via terminal_size::terminal_size() — None under no-TTY
- OutputStyle reads payload.output_style.name
- VimMode reads payload.vim.mode (None when vim disabled)
- SessionName takes file_stem of payload.transcript_path (.jsonl stripped)

SessionName lives in widgets/session.rs because T5 will add SessionClock
and SessionCost — single cluster, single file.

Empty/missing string fields → None (Plain renderer drops them).

Task 3/9 of Phase 3.
"
```

## Verification (стандартный гейт)

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/widgets/trivial.rs` создан, содержит 5 виджетов с `Widget` impl
- [ ] `src/widgets/session.rs` создан, содержит `SessionName`
- [ ] `Version`/`OutputStyle`/`VimMode` граcefully возвращают None на отсутствие/пустоту поля
- [ ] `ClaudeSessionId` берёт первые 8 chars (а не bytes — защита от non-ASCII)
- [ ] `TerminalWidth` использует `terminal_size::terminal_size()`, возвращает None под no-TTY
- [ ] `SessionName` использует `Path::file_stem()` — кросс-платформенно убирает `.jsonl`
- [ ] `widgets::mod` подключает оба новых модуля и инстанциирует 6 виджетов
- [ ] `cargo test --locked --lib widgets::trivial` — 13 тестов зелёные
- [ ] `cargo test --locked --lib widgets::session` — 3 теста зелёные
- [ ] Phase 2 default-line snapshot'ы зелёные (регрессии нет)
- [ ] Один commit `feat(phase-3): T3 trivial cluster + SessionName`

## Files touched

- `src/widgets/trivial.rs` (created)
- `src/widgets/session.rs` (created)
- `src/widgets/mod.rs` (modified)

## Risks & rollback

- **`terminal_size()` всё-таки вернул `Some(_)` в `cargo test` (CI-runner с `--nocapture` или TTY)**: тест `terminal_width_returns_none_under_cargo_test` проверяет либо None, либо `Some(width > 0)` — пропустит оба варианта.
- **`Path::file_stem()` ведёт себя по-разному на Unix vs Windows для пути `/tmp/abc.jsonl`**: на обеих платформах вернёт `abc`. Backslashes (Windows-only) — irrelevant, мы тестируем UNIX path.
- **`session_id` не UUID, а что-то экзотическое**: `chars().take(8)` корректно работает для любого строкового slice.
- **`p.version = Some(String::new())` не должен рендерить пустую строку**: explicit `if v.is_empty()` check в реализации `Version::render` — есть.
- **Clippy `pedantic::option_if_let_else` на `as_deref()?`**: канонический паттерн, игнорируем если жалоба.
- **Rollback**: `git revert HEAD` — снимает оба файла + 6 stub'ов восстанавливаются.
