# Task 6 — Tokens cluster: 5 widgets + format_tokens helper

**Files:**
- Create: `src/util/format_tokens.rs` (k/M formatter)
- Modify: `src/util/mod.rs` (`pub mod format_tokens;`)
- Create: `src/widgets/transcript_tokens.rs` (`TokensCached`, `TokensTotal`, `InputSpeed`, `OutputSpeed`, `TotalSpeed`)
- Modify: `src/widgets/mod.rs` (`pub mod transcript_tokens;` + 5 case'ов в `build_one`)
- Modify: `src/types/config.rs` (5 enum-вариантов)

## Goal

Первый кластер виджетов поверх `cache::TranscriptStats`.

Контракт T6:
1. `format_tokens(n)` — `n < 1000 → "{n}"`, `1000 ≤ n < 1_000_000 → "{n/1000:.1}k"` без хвостовых нулей, `n ≥ 1_000_000 → "{n/1_000_000:.1}M"` без хвостовых нулей.
2. **`TokensCached`** — `cT: <fmt>`; sum `cache_read + cache_creation`; None если sum == 0.
3. **`TokensTotal`** — `totT: <fmt>`; sum `input + output + cached`; None если sum == 0.
4. **`InputSpeed`** — `↓<n> t/s`; None если `last_assistant` пустой; speed = `tokens_in / max(duration_ms / 1000, 1)`.
5. **`OutputSpeed`** — `↑<n> t/s`; ditto.
6. **`TotalSpeed`** — `⇅<n> t/s`; `tokens_in + tokens_out`.
7. WidgetConfig получает 5 новых kebab-case вариантов; `build_one` 5 case'ов; `tests/snapshots_widgets.json` (если есть) обновляется в T9.

## Inputs

- T1–T5 закрыты.
- `RenderContext::transcript()` доступен.

---

- [ ] **Step 1: Создать `src/util/format_tokens.rs`**

Create `/Users/igor/mp/startup/cchud/src/util/format_tokens.rs`:

```rust
//! Format token counts as compact strings — Phase 6 Task 6.
//!
//! `n < 1000`            → `"{n}"`
//! `1000 ≤ n < 1_000_000` → `"{n/1000:.1}k"` без хвостового `.0`
//! `n ≥ 1_000_000`        → `"{n/1_000_000:.1}M"` без хвостового `.0`
//!
//! Примеры: 0 → "0", 999 → "999", 1000 → "1k", 1500 → "1.5k",
//!          999_500 → "999.5k", 1_000_000 → "1M", 1_234_567 → "1.2M".

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn format_tokens(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    if n < 1_000_000 {
        return compact(n as f64 / 1_000.0, "k");
    }
    compact(n as f64 / 1_000_000.0, "M")
}

fn compact(value: f64, suffix: &str) -> String {
    let s = format!("{value:.1}");
    let trimmed = s.strip_suffix(".0").unwrap_or(&s);
    format!("{trimmed}{suffix}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn under_thousand() {
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn round_thousand_no_trailing_zero() {
        assert_eq!(format_tokens(1_000), "1k");
    }

    #[test]
    fn fractional_thousand() {
        assert_eq!(format_tokens(1_500), "1.5k");
    }

    #[test]
    fn near_million() {
        assert_eq!(format_tokens(999_500), "999.5k");
    }

    #[test]
    fn round_million_no_trailing_zero() {
        assert_eq!(format_tokens(1_000_000), "1M");
    }

    #[test]
    fn fractional_million() {
        assert_eq!(format_tokens(1_234_567), "1.2M");
    }

    #[test]
    fn large_million() {
        assert_eq!(format_tokens(50_500_000), "50.5M");
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/util/format_tokens.rs`.

- [ ] **Step 2: Зарегистрировать в `src/util/mod.rs`**

Edit `src/util/mod.rs`:

- `old_string`:
  ```rust
  pub mod ansi;
  pub mod ascii_bar;
  pub mod duration;
  pub mod model_context_size;
  pub mod now;
  ```
- `new_string`:
  ```rust
  pub mod ansi;
  pub mod ascii_bar;
  pub mod duration;
  pub mod format_tokens;
  pub mod model_context_size;
  pub mod now;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/util/mod.rs`

- [ ] **Step 3: Добавить 5 enum-вариантов в `WidgetConfig`**

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      // Phase 5 — Task 7 (PR):
      GitPr,
  }
  ```
- `new_string`:
  ```rust
      // Phase 5 — Task 7 (PR):
      GitPr,

      // Phase 6 — Task 6 (transcript tokens cluster):
      TokensCached,
      TokensTotal,
      InputSpeed,
      OutputSpeed,
      TotalSpeed,
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

Затем добавить тест на serde:

Edit `src/types/config.rs`:

- `old_string`:
  ```rust
      #[test]
      fn parses_phase5_remote_widgets() {
  ```
- `new_string`:
  ```rust
      #[test]
      fn parses_phase6_token_widgets() {
          let json = r#"[
              { "type": "tokens-cached" },
              { "type": "tokens-total" },
              { "type": "input-speed" },
              { "type": "output-speed" },
              { "type": "total-speed" }
          ]"#;
          let widgets: Vec<WidgetConfig> = serde_json::from_str(json).unwrap();
          assert_eq!(widgets.len(), 5);
          assert!(matches!(widgets[0], WidgetConfig::TokensCached));
          assert!(matches!(widgets[4], WidgetConfig::TotalSpeed));
      }

      #[test]
      fn parses_phase5_remote_widgets() {
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/types/config.rs`

- [ ] **Step 4: Создать `src/widgets/transcript_tokens.rs`**

Create `/Users/igor/mp/startup/cchud/src/widgets/transcript_tokens.rs`:

```rust
//! Transcript tokens cluster — Phase 6 Task 6.
//!
//! Тонкие getter'ы над `RenderContext::transcript()`. Все возвращают None
//! если транскрипта нет / нужное поле пустое.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use crate::cache::{MessageStats, TranscriptStats};
use crate::util::format_tokens::format_tokens;
use crate::widgets::{RenderContext, Widget};

pub struct TokensCached;
pub struct TokensTotal;
pub struct InputSpeed;
pub struct OutputSpeed;
pub struct TotalSpeed;

impl Widget for TokensCached {
    fn id(&self) -> &'static str {
        "TokensCached"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let cached = t
            .tokens_cache_read_total
            .saturating_add(t.tokens_cache_creation_total);
        if cached == 0 {
            return None;
        }
        Some(format!("cT: {}", format_tokens(cached)))
    }
}

impl Widget for TokensTotal {
    fn id(&self) -> &'static str {
        "TokensTotal"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let t = ctx.transcript()?;
        let total = total_tokens(t);
        if total == 0 {
            return None;
        }
        Some(format!("totT: {}", format_tokens(total)))
    }
}

impl Widget for InputSpeed {
    fn id(&self) -> &'static str {
        "InputSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        Some(format!("↓{} t/s", speed(last.tokens_in, &last)))
    }
}

impl Widget for OutputSpeed {
    fn id(&self) -> &'static str {
        "OutputSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        Some(format!("↑{} t/s", speed(last.tokens_out, &last)))
    }
}

impl Widget for TotalSpeed {
    fn id(&self) -> &'static str {
        "TotalSpeed"
    }
    fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
        let last = ctx.transcript()?.last_assistant?;
        let total = last.tokens_in.saturating_add(last.tokens_out);
        Some(format!("⇅{} t/s", speed(total, &last)))
    }
}

fn total_tokens(t: &TranscriptStats) -> u64 {
    t.tokens_in_total
        .saturating_add(t.tokens_out_total)
        .saturating_add(t.tokens_cache_read_total)
        .saturating_add(t.tokens_cache_creation_total)
}

/// `tokens / max(duration_sec, 1)` → integer t/s.
fn speed(tokens: u64, msg: &MessageStats) -> u64 {
    let duration_ms = msg.completed_at_ms.saturating_sub(msg.started_at_ms);
    let duration_sec = (duration_ms / 1000).max(1);
    tokens / duration_sec
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::cache::TranscriptStats;
    use crate::config::default_line;
    use crate::types::payload::{ModelInfo, StatusPayload, Workspace};
    use crate::widgets::RenderContext;

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

    /// Помещает фиксированный TranscriptStats в OnceCell контекста через
    /// `#[cfg(test)]` helper из T5.
    fn ctx_with_stats<'a>(
        p: &'a StatusPayload,
        s: &'a crate::types::config::Settings,
        stats: TranscriptStats,
    ) -> RenderContext<'a> {
        let ctx = RenderContext::new(p, s);
        ctx.set_transcript_for_tests(Some(stats));
        ctx
    }

    #[test]
    fn tokens_cached_none_when_transcript_missing() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = RenderContext::new(&p, &s);
        assert!(TokensCached.render(&ctx).is_none());
    }

    #[test]
    fn tokens_cached_none_when_zero() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats::default();
        let ctx = ctx_with_stats(&p, &s, stats);
        assert!(TokensCached.render(&ctx).is_none());
    }

    #[test]
    fn tokens_cached_sums_read_and_creation() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            tokens_cache_read_total: 700,
            tokens_cache_creation_total: 300,
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(TokensCached.render(&ctx).as_deref(), Some("cT: 1k"));
    }

    #[test]
    fn tokens_total_none_when_zero() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats::default();
        let ctx = ctx_with_stats(&p, &s, stats);
        assert!(TokensTotal.render(&ctx).is_none());
    }

    #[test]
    fn tokens_total_sums_all_four_fields() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            tokens_in_total: 1000,
            tokens_out_total: 500,
            tokens_cache_read_total: 2000,
            tokens_cache_creation_total: 1000,
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        // 1000 + 500 + 2000 + 1000 = 4500 → "4.5k"
        assert_eq!(TokensTotal.render(&ctx).as_deref(), Some("totT: 4.5k"));
    }

    #[test]
    fn input_speed_none_when_no_last_assistant() {
        let p = payload_no_transcript();
        let s = default_line();
        let ctx = ctx_with_stats(&p, &s, TranscriptStats::default());
        assert!(InputSpeed.render(&ctx).is_none());
    }

    #[test]
    fn input_speed_uses_max_one_second_floor() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 1_000,
                completed_at_ms: 1_100, // 100 ms — duration < 1s, floor = 1s
                tokens_in: 1500,
                tokens_out: 500,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(InputSpeed.render(&ctx).as_deref(), Some("↓1500 t/s"));
    }

    #[test]
    fn output_speed_uses_real_duration_when_more_than_1s() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 0,
                completed_at_ms: 5_000, // 5 sec
                tokens_in: 0,
                tokens_out: 250,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        assert_eq!(OutputSpeed.render(&ctx).as_deref(), Some("↑50 t/s"));
    }

    #[test]
    fn total_speed_sums_in_and_out() {
        let p = payload_no_transcript();
        let s = default_line();
        let stats = TranscriptStats {
            last_assistant: Some(MessageStats {
                started_at_ms: 0,
                completed_at_ms: 2_000, // 2 sec
                tokens_in: 100,
                tokens_out: 100,
            }),
            ..TranscriptStats::default()
        };
        let ctx = ctx_with_stats(&p, &s, stats);
        // (100 + 100) / 2 = 100
        assert_eq!(TotalSpeed.render(&ctx).as_deref(), Some("⇅100 t/s"));
    }
}
```

Path: `/Users/igor/mp/startup/cchud/src/widgets/transcript_tokens.rs`.

> **Why test-only setter:** `RenderContext.transcript: OnceCell<...>` приватное (см. T5). Тестовый ход `set_transcript_for_tests` живёт под `#[cfg(test)]` в T5 — production API не загрязнён, никакого `unsafe`. Snapshot-тесты в T9 будут использовать реальный файл через `transcript_path`.

- [ ] **Step 5: Подключить модуль и build_one в `src/widgets/mod.rs`**

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
  pub mod static_text;
  pub mod trivial;
  pub mod worktree;
  ```
- `new_string`:
  ```rust
  pub mod static_text;
  pub mod transcript_tokens;
  pub mod trivial;
  pub mod worktree;
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

Затем добавить case'ы в `build_one`:

Edit `src/widgets/mod.rs`:

- `old_string`:
  ```rust
          // Phase 5 — Task 7 (PR):
          WidgetConfig::GitPr => Box::new(git_pr::GitPr),
      }
  }
  ```
- `new_string`:
  ```rust
          // Phase 5 — Task 7 (PR):
          WidgetConfig::GitPr => Box::new(git_pr::GitPr),

          // Phase 6 — Task 6 (transcript tokens cluster):
          WidgetConfig::TokensCached => Box::new(transcript_tokens::TokensCached),
          WidgetConfig::TokensTotal => Box::new(transcript_tokens::TokensTotal),
          WidgetConfig::InputSpeed => Box::new(transcript_tokens::InputSpeed),
          WidgetConfig::OutputSpeed => Box::new(transcript_tokens::OutputSpeed),
          WidgetConfig::TotalSpeed => Box::new(transcript_tokens::TotalSpeed),
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 6: Запустить tests кластера**

```bash
cargo test --locked util::format_tokens 2>&1 | tail -10
cargo test --locked widgets::transcript_tokens 2>&1 | tail -15
cargo test --locked types::config::tests::parses_phase6_token_widgets 2>&1 | tail -5
```

Expected: 8 + 9 + 1 = 18+ тестов passed.

- [ ] **Step 7: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: все exit 0.

- [ ] **Step 8: Verification — task-specific**

```bash
grep -c 'pub struct TokensCached' src/widgets/transcript_tokens.rs
grep -c 'pub fn format_tokens' src/util/format_tokens.rs
grep -c 'TokensCached =>' src/widgets/mod.rs
grep -c 'TotalSpeed,' src/types/config.rs
```

Expected:
```
1
1
1
1+
```

- [ ] **Step 9: Commit**

```bash
git add src/util/format_tokens.rs src/util/mod.rs src/widgets/transcript_tokens.rs src/widgets/mod.rs src/types/config.rs
git commit -m "feat(phase-6): T6 tokens cluster — 5 widgets + format_tokens helper

Adds 5 transcript token widgets:
- TokensCached: 'cT: <fmt>' for cache_read + cache_creation totals
- TokensTotal:  'totT: <fmt>' for in + out + cached
- InputSpeed:   '↓N t/s' from last_assistant message
- OutputSpeed:  '↑N t/s'
- TotalSpeed:   '⇅N t/s' (in + out per duration)

format_tokens helper:
- n < 1000              → '{n}'
- 1000 ≤ n < 1_000_000  → 'X.Yk' (no trailing .0)
- n ≥ 1_000_000         → 'X.YM'

WidgetConfig +5 kebab-case variants; build_one +5 cases. Speed widgets
floor duration to 1 sec to avoid div-by-zero on sub-second responses.

Task 6/10 of Phase 6. T7 adds BlockTimer + SessionDuration."
```

## Verification

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

## Definition of Done

- [ ] `src/util/format_tokens.rs` создан; `format_tokens(n)` корректно работает на 8 граничных значениях
- [ ] `src/widgets/transcript_tokens.rs` содержит 5 виджетов (`TokensCached`, `TokensTotal`, `InputSpeed`, `OutputSpeed`, `TotalSpeed`)
- [ ] Каждый виджет имеет ≥2 unit-теста (None case + happy case)
- [ ] `WidgetConfig` получает 5 новых kebab-case вариантов; serde парсинг покрыт тестом
- [ ] `build_one` 5 case'ов
- [ ] `cargo test util::format_tokens` зелёный (8 тестов); `cargo test widgets::transcript_tokens` зелёный (9 тестов)
- [ ] Никаких `unwrap`/`expect` в production-коде транскрипт-виджетов (только в тестах через `#[allow]`)
- [ ] Один commit `feat(phase-6): T6 tokens cluster ...`

## Files touched

- `src/util/format_tokens.rs` (created)
- `src/util/mod.rs` (modified — `pub mod format_tokens;`)
- `src/widgets/transcript_tokens.rs` (created)
- `src/widgets/mod.rs` (modified — `pub mod transcript_tokens;` + 5 build_one cases)
- `src/types/config.rs` (modified — 5 enum variants + serde test)

## Risks & rollback

- **Format тяжелее ожидаемого**: `format!` heap-аллок 24 байта на строку × 5 виджетов × N рендеров = ~120 байт overhead. Незначительно. Hyperfine T9 поймает регрессию если есть.
- **Unicode strelochki ⇅↑↓**: в plain рендере отображаются как 1-2 cell чары; на старых Windows может быть `?` в cmd.exe. README: "transcript widgets best in modern terminals (mac iTerm2 / Windows Terminal / VSCode)".
- **`f64` precision**: `n / 1000.0` для u64 < 2^53 точно; больше — теоретически теряется LSB. На наших масштабах (max ~1B токенов в сессии) — не релевантно.
- **Rollback**: `git revert HEAD` удаляет 5 виджетов + helper; T1–T5 продолжают компилироваться.
