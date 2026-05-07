# Task 5 — Cost cluster (SessionClock, SessionCost) + util/duration

**Files:**
- Modify: `src/util/duration.rs` (заменить stub на `format_duration` — `HH:MM:SS` если ≥ часа, `MM:SS` иначе)
- Modify: `src/widgets/session.rs` (добавить `SessionClock` и `SessionCost`)
- Modify: `src/widgets/mod.rs` (2 match-arms заменяют `Stub` на реальные impl)

## Goal

Маленький, фокусный кластер — 2 виджета.

| Widget | Source | Render |
|---|---|---|
| `SessionCost` | `payload.cost?.total_cost_usd` | `Some(format!("${:.2}", v))` |
| `SessionClock` | `payload.cost?.total_duration_ms` | `HH:MM:SS` если ≥ часа, `MM:SS` иначе (через `util::duration::format_duration`) |

`util::duration::format_duration(total_ms: u64) -> String`:

- сек = `total_ms / 1000`
- часы = `сек / 3600`, минуты = `(сек / 60) % 60`, секунды = `сек % 60`
- если `часы > 0` → `format!("{:02}:{:02}:{:02}", h, m, s)`
- если `часы == 0` → `format!("{:02}:{:02}", m, s)`

`SessionCost` форматирует `f64` как `"$0.72"` (две цифры после точки, с долларом). 0.7181426 → "$0.72".

## Inputs

- T1, T2, T3, T4 закрыты.
- `payload.cost` типизирован как `Option<CostInfo>` с полями `total_cost_usd: Option<f64>`, `total_duration_ms: Option<u64>` (T1).
- `widgets/session.rs` существует и содержит `SessionName` (T3).

---

- [ ] **Step 1: Реализовать `util::duration` с тестами**

Полностью заменить `/Users/igor/mp/startup/cchud/src/util/duration.rs`:

```rust
//! Duration formatter — `HH:MM:SS` (≥ 1h) или `MM:SS` (< 1h).
//!
//! Используется `widgets::session::SessionClock`. Phase 4/8 могут
//! получить colored variant; для Phase 3 — plain string.

#![deny(clippy::unwrap_used, clippy::expect_used)]

#[must_use]
pub fn format_duration(total_ms: u64) -> String {
    let total_secs = total_ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs / 60) % 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn zero_duration() {
        assert_eq!(format_duration(0), "00:00");
    }

    #[test]
    fn under_minute() {
        assert_eq!(format_duration(5_000), "00:05");
        assert_eq!(format_duration(59_999), "00:59");
    }

    #[test]
    fn under_hour() {
        // 90 sec = 1m30s
        assert_eq!(format_duration(90_000), "01:30");
        // 59m59s
        assert_eq!(format_duration(3_599_000), "59:59");
    }

    #[test]
    fn one_hour_threshold() {
        // 3600 sec = 1h00m00s
        assert_eq!(format_duration(3_600_000), "01:00:00");
    }

    #[test]
    fn multiple_hours() {
        // 2h05m07s = 7507 sec = 7_507_000 ms
        assert_eq!(format_duration(7_507_000), "02:05:07");
    }

    #[test]
    fn drops_sub_second_remainder() {
        // 1499 ms < 1.5 sec → 0 sec floor
        assert_eq!(format_duration(1_499), "00:01");
        // Нет, 1499 / 1000 = 1, потому "00:01". Корректно.
    }

    #[test]
    fn double_digit_hours() {
        // 100h00m00s = 360_000 sec = 360_000_000 ms
        assert_eq!(format_duration(360_000_000), "100:00:00");
    }
}
```

- [ ] **Step 2: Запустить util-тесты**

```bash
cargo test --locked --lib util::duration
```

Expected: 7 тестов passed.

- [ ] **Step 3: Дополнить `widgets/session.rs` — SessionCost + SessionClock**

Open `src/widgets/session.rs`. После существующего `SessionName` impl и ПЕРЕД `#[cfg(test)] mod tests` добавить:

Edit:
- `old_string`:
  ```rust
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
  ```
- `new_string`:
  ```rust
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

  pub struct SessionCost;

  impl Widget for SessionCost {
      fn id(&self) -> &'static str {
          "SessionCost"
      }
      fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
          let cost = ctx.payload.cost.as_ref()?.total_cost_usd?;
          Some(format!("${cost:.2}"))
      }
  }

  pub struct SessionClock;

  impl Widget for SessionClock {
      fn id(&self) -> &'static str {
          "SessionClock"
      }
      fn render(&self, ctx: &RenderContext<'_>) -> Option<String> {
          let ms = ctx.payload.cost.as_ref()?.total_duration_ms?;
          Some(crate::util::duration::format_duration(ms))
      }
  }

  #[cfg(test)]
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/session.rs`

- [ ] **Step 4: Дополнить тесты `session::tests` — 6 новых**

Open `src/widgets/session.rs`. Внутри `mod tests` ПОСЛЕ существующих 3 тестов добавить:

Edit:
- `old_string`:
  ```rust
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
- `new_string`:
  ```rust
      #[test]
      fn session_name_handles_path_without_extension() {
          let p = payload_with_transcript(Some("/tmp/abc"));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          // file_stem на пути без extension возвращает basename
          assert_eq!(SessionName.render(&ctx), Some("abc".into()));
      }

      // ─── SessionCost ────────────────────────────────────────────

      fn payload_with_cost(cost: Option<crate::types::payload::CostInfo>) -> StatusPayload {
          let mut p = payload_with_transcript(None);
          p.cost = cost;
          p
      }

      #[test]
      fn session_cost_renders_two_decimals() {
          let p = payload_with_cost(Some(crate::types::payload::CostInfo {
              total_cost_usd: Some(0.7181426),
              total_duration_ms: Some(0),
              total_api_duration_ms: None,
              total_lines_added: None,
              total_lines_removed: None,
          }));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionCost.render(&ctx), Some("$0.72".into()));
      }

      #[test]
      fn session_cost_renders_integer_part() {
          let p = payload_with_cost(Some(crate::types::payload::CostInfo {
              total_cost_usd: Some(12.5),
              total_duration_ms: None,
              total_api_duration_ms: None,
              total_lines_added: None,
              total_lines_removed: None,
          }));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionCost.render(&ctx), Some("$12.50".into()));
      }

      #[test]
      fn session_cost_returns_none_without_cost() {
          let p = payload_with_cost(None);
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionCost.render(&ctx), None);
      }

      #[test]
      fn session_cost_returns_none_without_total_cost_usd_field() {
          let p = payload_with_cost(Some(crate::types::payload::CostInfo {
              total_cost_usd: None,
              total_duration_ms: Some(1234),
              total_api_duration_ms: None,
              total_lines_added: None,
              total_lines_removed: None,
          }));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionCost.render(&ctx), None);
      }

      // ─── SessionClock ───────────────────────────────────────────

      #[test]
      fn session_clock_under_hour() {
          // 478472 ms = 7m58s
          let p = payload_with_cost(Some(crate::types::payload::CostInfo {
              total_cost_usd: None,
              total_duration_ms: Some(478_472),
              total_api_duration_ms: None,
              total_lines_added: None,
              total_lines_removed: None,
          }));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionClock.render(&ctx), Some("07:58".into()));
      }

      #[test]
      fn session_clock_over_hour() {
          // 1h 30m 0s = 5_400_000 ms
          let p = payload_with_cost(Some(crate::types::payload::CostInfo {
              total_cost_usd: None,
              total_duration_ms: Some(5_400_000),
              total_api_duration_ms: None,
              total_lines_added: None,
              total_lines_removed: None,
          }));
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionClock.render(&ctx), Some("01:30:00".into()));
      }

      #[test]
      fn session_clock_returns_none_without_cost() {
          let p = payload_with_cost(None);
          let s = default_line();
          let ctx = RenderContext::new(&p, &s);
          assert_eq!(SessionClock.render(&ctx), None);
      }
  }
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/session.rs`

- [ ] **Step 5: Запустить — должны быть зелёные**

```bash
cargo test --locked --lib widgets::session
```

Expected: 10 тестов passed (3 SessionName + 4 SessionCost + 3 SessionClock).

- [ ] **Step 6: Заменить 2 stub'а в `build_one`**

Edit `src/widgets/mod.rs`:
- `old_string`:
  ```rust
          WidgetConfig::SessionClock => Box::new(Stub("SessionClock")),
          WidgetConfig::SessionCost => Box::new(Stub("SessionCost")),
  ```
- `new_string`:
  ```rust
          // Phase 3 — Task 5 (cost cluster):
          WidgetConfig::SessionClock => Box::new(session::SessionClock),
          WidgetConfig::SessionCost => Box::new(session::SessionCost),
  ```
- `file_path`: `/Users/igor/mp/startup/cchud/src/widgets/mod.rs`

- [ ] **Step 7: Smoke-тест с реальным семплом**

```bash
cat benches/samples/payload-cchud-sonnet-xlarge.json | cargo run --release 2>/dev/null
```

Expected: `Sonnet 4.6` (default-line, рендер не сломан; SessionCost не в default-line).

Доп. smoke (вручную задать config через env, но проще — пропустить, smoke на реальных конфигах будет в Task 8).

- [ ] **Step 8: Standard gate**

```bash
cargo build --release --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo fmt --check
```

Expected: всё зелёное. Тестов суммарно ≥89.

- [ ] **Step 9: Verification — task-specific gate**

```bash
grep -c 'pub fn format_duration' src/util/duration.rs
grep -c 'pub struct SessionCost' src/widgets/session.rs
grep -c 'pub struct SessionClock' src/widgets/session.rs
grep -c 'session::SessionCost' src/widgets/mod.rs
grep -c 'session::SessionClock' src/widgets/mod.rs
```

Expected: каждая команда → `1`.

- [ ] **Step 10: Commit**

```bash
git add src/util/duration.rs src/widgets/session.rs src/widgets/mod.rs
git commit -m "feat(phase-3): T5 cost cluster — SessionCost, SessionClock + util duration

util::duration::format_duration(total_ms): HH:MM:SS if >= 1h,
MM:SS otherwise. 7 unit tests covering 0/<1m/<1h/exactly-1h/multi-h/
sub-second-truncation/triple-digit-hours.

widgets::session: + SessionCost (payload.cost.total_cost_usd → \$0.72
two-decimals), + SessionClock (payload.cost.total_duration_ms via
format_duration). Both return None when cost field absent.

Task 5/9 of Phase 3.
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

- [ ] `src/util/duration.rs` содержит `format_duration(total_ms: u64) -> String`
- [ ] HH:MM:SS / MM:SS логика правильно ловит границу 1 часа
- [ ] ≥7 unit-тестов в `util::duration::tests` (0, <1m, <1h, ровно 1h, multi-h, sub-second, triple-digit hours)
- [ ] `widgets::session` содержит `SessionCost` (формат `$X.XX`) и `SessionClock` (формат через duration)
- [ ] `SessionCost`/`SessionClock` возвращают None когда `payload.cost` отсутствует или конкретное поле = None
- [ ] `widgets::mod` инстанциирует SessionCost/SessionClock реально (без Stub)
- [ ] `cargo test --locked --lib widgets::session` — 10 тестов зелёные
- [ ] `cargo test --locked` зелёный (≥89 тестов суммарно)
- [ ] Один commit `feat(phase-3): T5 cost cluster ...`

## Files touched

- `src/util/duration.rs` (modified, stub → impl)
- `src/widgets/session.rs` (modified, +2 widgets)
- `src/widgets/mod.rs` (modified, 2 stub→real)

## Risks & rollback

- **`format!("${cost:.2}")` округляет неправильно при `0.005` (round-half-to-even в `f64::format`)**: edge-case for currency, не критично для UI. Если когда-то всплывёт — заменить на `(cost * 100.0).round() / 100.0` форматирование. Phase 3 не парится.
- **Очень большое `total_duration_ms` (overflow `u64::MAX / 1000`)**: 18.4 квинтиллиона ms = ~584 миллиона лет. Не наш кейс.
- **`u64 / 1000` теряет миллисекундную точность**: ОК, мы рендерим до секунды.
- **Phase 2 spec говорил "Phase 6 типизирует cost"**: T1 ускорил типизацию (DECISIONS-запись от 2026-04-26 объясняет). Доступ через `payload.cost.as_ref()?.total_cost_usd?` — компилятор лочит схему.
- **Rollback**: `git revert HEAD` — duration возвращается к stub'у, SessionCost/Clock — Stub'ы, остальные тесты не задеты.
