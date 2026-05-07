# Task 7 — Timing cluster: BlockTimer, SessionDuration

**Files:**
- Create: `src/widgets/transcript_timing.rs` (`BlockTimer`, `SessionDuration`)
- Modify: `src/widgets/mod.rs` (`pub mod transcript_timing;` + 2 build_one cases)
- Modify: `src/types/config.rs` (2 enum-варианта)

## Goal

Виджеты на основе `now_ms` и `BillingBlock`/session timestamps.

Контракт T7:
1. **`BlockTimer`** — `⏰ <duration>`; берёт `t.blocks.last()`. None если нет блоков или `now_ms >= active.ends_at_ms`. duration = `ends_at_ms - now_ms`.
2. **`SessionDuration`** — `<duration>`; `last_at - first_at` от `t.session_started_at_ms`/`t.session_last_at_ms`. None если хотя бы одно поле пустое.
3. Оба используют `crate::util::duration::format_duration(ms)` (Phase 3 helper, `MM:SS` / `HH:MM:SS`).
4. Тесты используют фиксированный `ctx.now_ms` через field-init.

## Inputs

- T1–T6 закрыты.
- `RenderContext::transcript()` и `RenderContext.now_ms` доступны.

---

- [ ] **Step 1: Создать `src/widgets/transcript_timing.rs`**

Create `/Users/igor/mp/startup/cchud/src/widgets/transcript_timing.rs`:

```rust
//! Transcript timing cluster — Phase 6 Task 7.
//!
//! - `BlockTimer`: time-to-end текущего 5h billing-блока.
//! - `SessionDuration`: диапазон между первым и последним сообщением.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::util::duration::format_duration;
use crate::widgets::{RenderContext, Widget};

pub struct BlockTimer;
pub struct SessionDuration;

impl Widget for BlockTimer {
    fn id(&self) -> &'static str {
        "BlockTimer"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let active = ctx.transcript()?.blocks.last()?;
        if ctx.now_ms >= active.ends_at_ms {
            return None;
        }
        let remaining_ms = active.ends_at_ms - ctx.now_ms;
        Some(format!("⏰ {}", format_duration(remaining_ms)))
    }
}

impl Widget for SessionDuration {
    fn id(&self) -> &'static str {
        "SessionDuration"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let start = t.session_started_at_ms?;
        let end = t.session_last_at_ms?;
        if end < start {
            return None;
        }
        Some(format_duration(end - start))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::{BillingBlock, TranscriptStats};
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
        now_ms: u64,
    ) -> RenderContext<'_> {
        let mut ctx = RenderContext::new(p, s);
        ctx.now_ms = now_ms;
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn block_timer_none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn block_timer_none_when_no_blocks() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default(), 1_000_000);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn block_timer_renders_remaining_minutes() {
        let p = payload_no_transcript();
        let s = default_line();
        let block = BillingBlock {
            started_at_ms: 0,
            ends_at_ms: 5 * 3600 * 1000, // 5 часов
        };
        let stats = TranscriptStats {
            blocks: vec![block],
            ..TranscriptStats::default()
        };
        // now = 4ч 30мин
        let now_ms = (4 * 3600 + 30 * 60) * 1000;
        let ctx = ctx_with(&p, &s, stats, now_ms);
        // remaining = 30 мин = "30:00"
        assert_eq!(BlockTimer.render(&ctx).as_deref(), Some("⏰ 30:00"));
    }

    #[test]
    fn block_timer_none_when_now_past_end() {
        let p = payload_no_transcript();
        let s = default_line();
        let block = BillingBlock {
            started_at_ms: 0,
            ends_at_ms: 1000,
        };
        let stats = TranscriptStats {
            blocks: vec![block],
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 5000);
        assert!(BlockTimer.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_none_when_no_transcript() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(SessionDuration.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_none_when_bounds_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with(&p, &s, TranscriptStats::default(), 0);
        assert!(SessionDuration.render(&ctx).is_none());
    }

    #[test]
    fn session_duration_formats_minutes_seconds() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            session_started_at_ms: Some(0),
            session_last_at_ms: Some(125_000), // 2:05
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 0);
        assert_eq!(SessionDuration.render(&ctx).as_deref(), Some("02:05"));
    }

    #[test]
    fn session_duration_formats_hours_when_long() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            session_started_at_ms: Some(0),
            session_last_at_ms: Some(3_661_000), // 1ч 01м 01с
            ..TranscriptStats::default()
        };
        let ctx = ctx_with(&p, &s, stats, 0);
        assert_eq!(SessionDuration.render(&ctx).as_deref(), Some("01:01:01"));
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/widgets/transcript_timing.rs`.

- [ ] **Step 2: Подключить модуль и build_one**

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
  pub mod static_text;
  pub mod transcript_tokens;
  pub mod trivial;
  ```
- `new_string`:
  ```rust
  pub mod static_text;
  pub mod transcript_timing;
  pub mod transcript_tokens;
  pub mod trivial;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Затем добавить case'ы:

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
          // Phase 6 — Task 6 (transcript tokens cluster):
          WidgetConfig::TokensCached => Box::new(transcript_tokens::TokensCached),
          WidgetConfig::TokensTotal => Box::new(transcript_tokens::TokensTotal),
          WidgetConfig::InputSpeed => Box::new(transcript_tokens::InputSpeed),
          WidgetConfig::OutputSpeed => Box::new(transcript_tokens::OutputSpeed),
          WidgetConfig::TotalSpeed => Box::new(transcript_tokens::TotalSpeed),
      }
  }
  ```
- `new_string`:
  ```rust
          // Phase 6 — Task 6 (transcript tokens cluster):
          WidgetConfig::TokensCached => Box::new(transcript_tokens::TokensCached),
          WidgetConfig::TokensTotal => Box::new(transcript_tokens::TokensTotal),
          WidgetConfig::InputSpeed => Box::new(transcript_tokens::InputSpeed),
          WidgetConfig::OutputSpeed => Box::new(transcript_tokens::OutputSpeed),
          WidgetConfig::TotalSpeed => Box::new(transcript_tokens::TotalSpeed),
          // Phase 6 — Task 7 (transcript timing cluster):
          WidgetConfig::BlockTimer => Box::new(transcript_timing::BlockTimer),
          WidgetConfig::SessionDuration => Box::new(transcript_timing::SessionDuration),
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 3: Добавить enum-варианты**

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      // Phase 6 — Task 6 (transcript tokens cluster):
      TokensCached,
      TokensTotal,
      InputSpeed,
      OutputSpeed,
      TotalSpeed,
  }
  ```
- `new_string`:
  ```rust
      // Phase 6 — Task 6 (transcript tokens cluster):
      TokensCached,
      TokensTotal,
      InputSpeed,
      OutputSpeed,
      TotalSpeed,
      // Phase 6 — Task 7 (transcript timing cluster):
      BlockTimer,
      SessionDuration,
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

Тест serde:

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      #[test]
      fn parses_phase6_token_widgets() {
  ```
- `new_string`:
  ```rust
      #[test]
      fn parses_phase6_timing_widgets() {
          let json = r#"[
              { "type": "block-timer" },
              { "type": "session-duration" }
          ]"#;
          let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
          assert_eq!(widgets.len(), 2);
          assert!(matches!(widgets[0], WidgetConfig::BlockTimer));
          assert!(matches!(widgets[1], WidgetConfig::SessionDuration));
      }

      #[test]
      fn parses_phase6_token_widgets() {
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

- [ ] **Step 4: Запустить тесты**

```bash
cargo test --locked widgets::transcript_timing 2>&1 | tail -15
cargo test --locked types::config::tests::parses_phase6_timing_widgets 2>&1 | tail -5
```

Expected: 8 + 1 = 9 тестов passed.

- [ ] **Step 5: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 6: Verification**

```bash
grep -c 'pub struct BlockTimer' src/widgets/transcript_timing.rs
grep -c 'pub struct SessionDuration' src/widgets/transcript_timing.rs
grep -c 'BlockTimer =>' src/widgets/mod.rs
grep -c 'SessionDuration,' src/types/config.rs
```

Expected:
```
1
1
1
1+
```

- [ ] **Step 7: Commit**

```bash
git add src/widgets/transcript_timing.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-6): T7 timing cluster — BlockTimer + SessionDuration

Adds 2 transcript timing widgets:
- BlockTimer:       '⏰ <fmt_duration>' time remaining in current 5h
                    billing block (now_ms vs blocks.last().ends_at_ms).
                    None when no blocks or block already expired.
- SessionDuration:  format_duration(last_at - first_at). None when bounds
                    incomplete (CC sessions без timestamps в каждом entry).

Both use util::duration::format_duration (Phase 3) for MM:SS / HH:MM:SS
output. Tests inject deterministic now_ms via field-init.

WidgetConfig +2 kebab-case variants; build_one +2 cases.

Task 7/10 of Phase 6. T8 adds ThinkingEffort."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/widgets/transcript_timing.rs` создан с двумя виджетами
- [ ] `BlockTimer` рендерит `⏰ MM:SS` / `⏰ HH:MM:SS`; None при `now >= ends_at`
- [ ] `SessionDuration` рендерит формат duration; None при отсутствии bounds; защита `end < start → None`
- [ ] `WidgetConfig` 2 новых kebab-case варианта; serde тест зелёный
- [ ] `build_one` 2 case'а
- [ ] `cargo test widgets::transcript_timing` зелёный (8 тестов)
- [ ] Один commit `feat(phase-6): T7 timing cluster ...`

## Files touched

- `src/widgets/transcript_timing.rs` (created)
- `src/widgets/mod.rs` (modified — `pub mod transcript_timing;` + 2 build_one cases)
- `src/types/config.rs` (modified — 2 variants + serde test)

## Risks & rollback

- **`format_duration(ms)` от Phase 3 ожидает `u64`**: `ends_at - now` — `u64`, `last - first` — `u64`. Защита от underflow: явно проверяем порядок (`now >= ends_at` или `end < start`) и возвращаем None.
- **`ctx.now_ms` устаревает**: каждый `RenderContext::new()` инжектит свежее `unix_now_ms()`. На один рендер `now_ms` фиксирован — это и есть желаемый snapshot.
- **`SessionDuration` пересекается с `SessionClock`**: `SessionClock` (Phase 3) показывает `cost.total_duration_ms` (wall-time CLI), `SessionDuration` — диапазон transcript timestamp'ов. См. spec § Decision 14.
- **Rollback**: `git revert HEAD` снимает 2 виджета; T1–T6 остаются работать.
