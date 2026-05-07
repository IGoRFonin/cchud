# Task 8 — Thinking effort widget

**Files:**
- Create: `src/widgets/transcript_meta.rs` (`ThinkingEffort`)
- Modify: `src/widgets/mod.rs` (`pub mod transcript_meta;` + 1 build_one case)
- Modify: `src/types/config.rs` (1 enum-вариант)

## Goal

Последний виджет Phase 6 — индикатор thinking-уровня последнего assistant-сообщения.

Контракт T8:
1. **`ThinkingEffort`** — `🧠 {level}`; читает `t.last_thinking_effort`. None если поле пустое (старая сессия / отключено).
2. Принимаем любое непустое значение (`low | medium | high | xhigh | max | <future>`); никакой валидации (см. spec § Decision 11).
3. WidgetConfig + kebab-case вариант, build_one case.

## Inputs

- T1–T7 закрыты.

---

- [ ] **Step 1: Создать `src/widgets/transcript_meta.rs`**

Create `/Users/igor/mp/startup/cchud/src/widgets/transcript_meta.rs`:

```rust
//! Transcript meta cluster — Phase 6 Task 8.
//!
//! `ThinkingEffort` отображает уровень thinking из последнего
//! assistant-сообщения. Принимаем любую непустую строку (low/medium/high/
//! xhigh/max — текущие значения CC; future-proof).

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::widgets::{RenderContext, Widget};

pub struct ThinkingEffort;

impl Widget for ThinkingEffort {
    fn id(&self) -> &'static str {
        "ThinkingEffort"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let level = ctx.transcript()?.last_thinking_effort.as_deref()?;
        if level.is_empty() {
            return None;
        }
        Some(format!("🧠 {level}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::TranscriptStats;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};

    fn payload_no_transcript() -> StatusPayload {
        StatusPayload {
            session_id: "test".into(),
            model: ModelInfo {
                id: "m".into(),
                display_name: "M".into(),
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

    fn ctx_with(
        p: &StatusPayload,
        s: &crate::types::config::Settings,
        stats: TranscriptStats,
    ) -> RenderContext<'_> {
        let ctx = RenderContext::new(p, s);
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(ThinkingEffort.render(&ctx).is_none());
    }

    #[test]
    fn none_when_field_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default());
        assert!(ThinkingEffort.render(&ctx).is_none());
    }

    #[test]
    fn renders_high_level() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some("high".into()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert_eq!(ThinkingEffort.render(&ctx).as_deref(), Some("🧠 high"));
    }

    #[test]
    fn renders_unknown_future_level_unchanged() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some("ultra".into()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert_eq!(ThinkingEffort.render(&ctx).as_deref(), Some("🧠 ultra"));
    }

    #[test]
    fn empty_string_returns_none() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_thinking_effort: Some(String::new()),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats);
        assert!(ThinkingEffort.render(&ctx).is_none());
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/widgets/transcript_meta.rs`.

- [ ] **Step 2: Подключить модуль и build_one**

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
  pub mod static_text;
  pub mod transcript_timing;
  pub mod transcript_tokens;
  pub mod trivial;
  ```
- `new_string`:
  ```rust
  pub mod static_text;
  pub mod transcript_meta;
  pub mod transcript_timing;
  pub mod transcript_tokens;
  pub mod trivial;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Затем case:

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
          // Phase 6 — Task 7 (transcript timing cluster):
          WidgetConfig::BlockTimer => Box::new(transcript_timing::BlockTimer),
          WidgetConfig::SessionDuration => Box::new(transcript_timing::SessionDuration),
      }
  }
  ```
- `new_string`:
  ```rust
          // Phase 6 — Task 7 (transcript timing cluster):
          WidgetConfig::BlockTimer => Box::new(transcript_timing::BlockTimer),
          WidgetConfig::SessionDuration => Box::new(transcript_timing::SessionDuration),
          // Phase 6 — Task 8 (transcript meta cluster):
          WidgetConfig::ThinkingEffort => Box::new(transcript_meta::ThinkingEffort),
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 3: Добавить enum-вариант**

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      // Phase 6 — Task 7 (transcript timing cluster):
      BlockTimer,
      SessionDuration,
  }
  ```
- `new_string`:
  ```rust
      // Phase 6 — Task 7 (transcript timing cluster):
      BlockTimer,
      SessionDuration,
      // Phase 6 — Task 8 (transcript meta cluster):
      ThinkingEffort,
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

Тест serde:

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      #[test]
      fn parses_phase6_timing_widgets() {
  ```
- `new_string`:
  ```rust
      #[test]
      fn parses_phase6_thinking_widget() {
          let json = r#"[{ "type": "thinking-effort" }]"#;
          let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
          assert!(matches!(widgets[0], WidgetConfig::ThinkingEffort));
      }

      #[test]
      fn parses_phase6_timing_widgets() {
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

- [ ] **Step 4: Запустить тесты**

```bash
cargo test --locked widgets::transcript_meta 2>&1 | tail -10
cargo test --locked types::config::tests::parses_phase6_thinking_widget 2>&1 | tail -5
```

Expected: 5 + 1 = 6 тестов passed.

- [ ] **Step 5: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 6: Verification — все 8 виджетов Phase 6 на месте**

```bash
grep -cE '(TokensCached|TokensTotal|InputSpeed|OutputSpeed|TotalSpeed|BlockTimer|SessionDuration|ThinkingEffort) =>' src/widgets/mod.rs
```

Expected: `8` — все 8 виджетов wired в `build_one`.

```bash
ls src/widgets/transcript_*.rs
```

Expected:
```
src/widgets/transcript_meta.rs
src/widgets/transcript_timing.rs
src/widgets/transcript_tokens.rs
```

- [ ] **Step 7: Commit**

```bash
git add src/widgets/transcript_meta.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-6): T8 thinking-effort — last assistant thinking level

ThinkingEffort widget reads last_thinking_effort from TranscriptStats
and renders '🧠 {level}'. Accepts any non-empty string (low/medium/high/
xhigh/max are current CC values; future levels render unchanged).

WidgetConfig +1 kebab-case variant; build_one +1 case. All 8 Phase 6
widgets are now wired.

Task 8/10 of Phase 6. T9 adds snapshot suite + hyperfine bench."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/widgets/transcript_meta.rs` создан с `ThinkingEffort`
- [ ] Виджет рендерит `🧠 {level}` для непустой строки; None для отсутствующего/пустого поля; future-уровни пропускаются 1:1
- [ ] `WidgetConfig::ThinkingEffort` добавлен; serde тест зелёный
- [ ] `build_one` 1 case
- [ ] `cargo test widgets::transcript_meta` зелёный (5 тестов)
- [ ] Все 8 Phase 6 виджетов wired через `build_one`
- [ ] Один commit `feat(phase-6): T8 thinking-effort ...`

## Files touched

- `src/widgets/transcript_meta.rs` (created)
- `src/widgets/mod.rs` (modified — `pub mod transcript_meta;` + 1 build_one case)
- `src/types/config.rs` (modified — 1 variant + serde test)

## Risks & rollback

- **CC может ввести новое значение `effort` (например `extra-high`)**: принимаем любое без валидации (spec § Decision 11). Виджет рендерит как есть.
- **Emoji 🧠 рендерится плохо в старых терминалах**: 2-cell rendering ОК на iTerm2 / Windows Terminal / VSCode. На plain xterm может быть `?`. README: "transcript widgets best in modern terminals".
- **Rollback**: `git revert HEAD` снимает виджет; T1–T7 продолжают работать.
