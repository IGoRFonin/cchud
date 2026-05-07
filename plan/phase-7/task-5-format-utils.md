# Task 5 — format_memory + format_duration_long

**Цель:** Реализовать два formatter-helper'а, которые понадобятся в T8/T9: auto-unit byte formatter (`💾 7.5G`) и duration formatters short/long (`4h32m` / `5d 14h`).

**Files:**
- Modify: `src/util/format_memory.rs` — auto-unit formatter (`format(bytes: u64) -> String`)
- Modify: `src/util/format_duration_long.rs` — `format_short(seconds)` + `format_long(seconds)`

---

- [ ] **Step 1: Write failing tests for format_memory**

В `src/util/format_memory.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_under_1k() {
        assert_eq!(format(0), "0b");
        assert_eq!(format(512), "512b");
    }

    #[test]
    fn formats_kilobytes() {
        assert_eq!(format(1_024), "1k");
        assert_eq!(format(2_048), "2k");
        assert_eq!(format(1_500), "1.5k");
    }

    #[test]
    fn formats_megabytes() {
        assert_eq!(format(1_048_576), "1M");
        assert_eq!(format(7_864_320), "7.5M");
    }

    #[test]
    fn formats_gigabytes_with_drop_trailing_zero() {
        assert_eq!(format(2 * 1_073_741_824), "2G");
        assert_eq!(format((7.5 * 1_073_741_824.0) as u64), "7.5G");
    }

    #[test]
    fn formats_terabytes() {
        let tb = 1_099_511_627_776_u64;
        assert_eq!(format(tb), "1T");
        assert_eq!(format(tb * 3 / 2), "1.5T");
    }
}
```

- [ ] **Step 2: Run failing test**

Run: `cargo test --lib util::format_memory`
Expected: FAIL (нет функции `format`).

- [ ] **Step 3: Реализовать format_memory**

В `src/util/format_memory.rs`:

```rust
//! Auto-unit byte formatter (Kb/Mb/Gb/Tb).

#![deny(clippy::unwrap_used, clippy::expect_used)]

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;
const TB: u64 = 1024 * GB;

/// Auto-unit byte formatter (b/k/M/G/T).
/// Drops trailing zeros: `2.0G → 2G`, `7.50G → 7.5G`.
#[must_use]
pub fn format(bytes: u64) -> String {
    let (val, unit) = if bytes < KB {
        return format!("{bytes}b");
    } else if bytes < MB {
        (bytes as f64 / KB as f64, "k")
    } else if bytes < GB {
        (bytes as f64 / MB as f64, "M")
    } else if bytes < TB {
        (bytes as f64 / GB as f64, "G")
    } else {
        (bytes as f64 / TB as f64, "T")
    };

    format_one_decimal(val, unit)
}

fn format_one_decimal(val: f64, unit: &str) -> String {
    let rounded = (val * 10.0).round() / 10.0;
    if (rounded - rounded.trunc()).abs() < f64::EPSILON {
        format!("{}{unit}", rounded.trunc() as u64)
    } else {
        format!("{rounded:.1}{unit}")
    }
}
```

- [ ] **Step 4: Run format_memory tests**

Run: `cargo test --lib util::format_memory`
Expected: PASS — все 5 тестов.

- [ ] **Step 5: Write failing tests for format_duration_long**

В `src/util/format_duration_long.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // format_short: < 1m / Nm / NhMm / Nd Mh
    #[test]
    fn short_under_1_minute() {
        assert_eq!(format_short(0), "< 1m");
        assert_eq!(format_short(45), "< 1m");
    }

    #[test]
    fn short_minutes() {
        assert_eq!(format_short(60), "1m");
        assert_eq!(format_short(42 * 60), "42m");
    }

    #[test]
    fn short_hours_and_minutes() {
        assert_eq!(format_short(4 * 3600 + 32 * 60), "4h32m");
        assert_eq!(format_short(23 * 3600 + 59 * 60), "23h59m");
    }

    #[test]
    fn short_days_and_hours() {
        assert_eq!(format_short(24 * 3600), "1d 0h");
        assert_eq!(format_short(24 * 3600 + 5 * 3600), "1d 5h");
    }

    #[test]
    fn short_negative_clamped_to_under_1m() {
        assert_eq!(format_short(-100), "< 1m");
    }

    // format_long: Nm / NhMm / Nd Mh / Nw Md
    #[test]
    fn long_minutes() {
        assert_eq!(format_long(42 * 60), "42m");
    }

    #[test]
    fn long_hours_and_minutes() {
        assert_eq!(format_long(4 * 3600 + 32 * 60), "4h32m");
    }

    #[test]
    fn long_days_and_hours() {
        assert_eq!(format_long(5 * 86400 + 14 * 3600), "5d 14h");
    }

    #[test]
    fn long_weeks_and_days() {
        assert_eq!(format_long(8 * 86400), "1w 1d");
        assert_eq!(format_long(14 * 86400 + 3 * 86400), "2w 3d");
    }
}
```

- [ ] **Step 6: Run failing tests**

Run: `cargo test --lib util::format_duration_long`
Expected: FAIL.

- [ ] **Step 7: Реализовать format_short / format_long**

```rust
//! Duration formatters — short ("4h32m") and long ("5d 14h") variants.

#![deny(clippy::unwrap_used, clippy::expect_used)]

const MIN: i64 = 60;
const HOUR: i64 = 60 * MIN;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;

/// Short form: `< 1m`, `Nm`, `NhMm`, `Nd Mh`. Negative seconds → `< 1m`.
#[must_use]
pub fn format_short(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < MIN {
        return "< 1m".to_string();
    }
    if s < HOUR {
        return format!("{}m", s / MIN);
    }
    if s < DAY {
        return format!("{}h{}m", s / HOUR, (s % HOUR) / MIN);
    }
    format!("{}d {}h", s / DAY, (s % DAY) / HOUR)
}

/// Long form: `Nm`, `NhMm`, `Nd Mh`, `Nw Md`.
#[must_use]
pub fn format_long(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < HOUR {
        return format!("{}m", (s / MIN).max(0));
    }
    if s < DAY {
        return format!("{}h{}m", s / HOUR, (s % HOUR) / MIN);
    }
    if s < WEEK {
        return format!("{}d {}h", s / DAY, (s % DAY) / HOUR);
    }
    format!("{}w {}d", s / WEEK, (s % WEEK) / DAY)
}
```

- [ ] **Step 8: Run format_duration_long tests**

Run: `cargo test --lib util::format_duration_long`
Expected: PASS — все 9 тестов.

- [ ] **Step 9: Run full test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add src/util/format_memory.rs src/util/format_duration_long.rs
git commit -m "feat(phase-7): T5 — format_memory + format_duration_long helpers"
```
