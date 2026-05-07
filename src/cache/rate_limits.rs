//! Disk-cache последних известных `RateLimits`.
//!
//! Зачем: при первом рендере свежей CC-сессии `payload.rate_limits` пуст —
//! Claude Code заполняет его только после первого API-запроса. Чтобы
//! `session-usage` / `block-reset-timer` не «мигали», мы сохраняем
//! последний known-good `RateLimits` на диск, ключ — `claude_account_email`.
//!
//! Стратегия:
//! - Read-path: bincode → `Option<RateLimits>`; любая ошибка → `None`.
//! - Write-path: атомарный rename через `<path>.tmp` (паттерн из `store.rs`).
//! - Истёкшие buckets (`resets_at < now`) сдвигаются вперёд на длину окна
//!   (5h / 7d) и `used_percentage` сбрасывается в 0.

#![deny(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File};
use std::hash::Hasher;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::types::payload::{RateBucket, RateLimits};

const FIVE_HOUR_SECS: i64 = 5 * 3600;
const SEVEN_DAY_SECS: i64 = 7 * 86_400;

#[must_use]
pub fn cache_path_for_email(email: &str) -> PathBuf {
    cache_path_for_email_in(&super::cache_root(), email)
}

#[must_use]
fn cache_path_for_email_in(root: &Path, email: &str) -> PathBuf {
    let mut h = siphasher::sip::SipHasher24::new();
    h.write(email.as_bytes());
    let hex = format!("{:016x}", h.finish());
    root.join(format!("rate_limits-{hex}.bincode"))
}

#[must_use]
pub fn read_cached(email: &str) -> Option<RateLimits> {
    if email.is_empty() {
        return None;
    }
    read_cached_at(&cache_path_for_email(email))
}

pub fn write_cache_best_effort(email: &str, rl: &RateLimits) {
    if email.is_empty() {
        return;
    }
    let path = cache_path_for_email(email);
    let _ = write_atomic(&path, rl);
}

fn read_cached_at(path: &Path) -> Option<RateLimits> {
    let f = File::open(path).ok()?;
    let mut reader = BufReader::new(f);
    bincode::deserialize_from(&mut reader).ok()
}

fn write_atomic(path: &Path, rl: &RateLimits) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    {
        let f = File::create(&tmp)?;
        let mut writer = BufWriter::new(f);
        bincode::serialize_into(&mut writer, rl).map_err(std::io::Error::other)?;
        writer.flush()?;
    }
    fs::rename(&tmp, path)
}

/// Если `resets_at < now`, сдвигаем вперёд на `window_secs` пока не станет в будущем,
/// и сбрасываем `used_percentage` в `Some(0.0)`. Если `resets_at = None` — no-op.
const fn roll_bucket(b: &mut RateBucket, now_unix_s: i64, window_secs: i64) {
    let Some(mut resets) = b.resets_at else {
        return;
    };
    if resets >= now_unix_s {
        return;
    }
    while resets < now_unix_s {
        resets += window_secs;
    }
    b.resets_at = Some(resets);
    b.used_percentage = Some(0.0);
}

/// Обнуляет процент и сдвигает `resets_at` для истёкших бакетов.
pub const fn roll_expired_buckets(rl: &mut RateLimits, now_unix_s: i64) {
    if let Some(b) = rl.five_hour.as_mut() {
        roll_bucket(b, now_unix_s, FIVE_HOUR_SECS);
    }
    if let Some(b) = rl.seven_day.as_mut() {
        roll_bucket(b, now_unix_s, SEVEN_DAY_SECS);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_rl(five_pct: f64, five_resets: i64) -> RateLimits {
        RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(five_pct),
                resets_at: Some(five_resets),
            }),
            seven_day: Some(RateBucket {
                used_percentage: Some(8.0),
                resets_at: Some(five_resets + 86_400),
            }),
        }
    }

    #[test]
    fn cache_path_includes_hex16() {
        let p = cache_path_for_email("a@b.com");
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("rate_limits-"));
        assert!(name.ends_with(".bincode"));
        let hex = name
            .strip_prefix("rate_limits-")
            .unwrap()
            .strip_suffix(".bincode")
            .unwrap();
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cache_path_differs_per_email() {
        let a = cache_path_for_email("alice@x.com");
        let b = cache_path_for_email("bob@x.com");
        assert_ne!(a, b);
    }

    #[test]
    fn empty_email_returns_none_on_read() {
        assert!(read_cached("").is_none());
    }

    #[test]
    fn empty_email_skips_write() {
        // Public write для пустого email — no-op, прозрачен для FS (просто early-return).
        write_cache_best_effort("", &sample_rl(50.0, 1_800_000_000));
        // Никакая ассерция на FS не нужна — функция просто не делает IO.
    }

    #[test]
    fn write_then_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let email = "user@example.com";
        let path = cache_path_for_email_in(dir.path(), email);
        let rl = sample_rl(42.5, 1_800_000_000);
        write_atomic(&path, &rl).unwrap();
        let got = read_cached_at(&path).expect("must read back");
        assert_eq!(got.five_hour.unwrap().used_percentage, Some(42.5));
        assert_eq!(got.five_hour.unwrap().resets_at, Some(1_800_000_000));
        assert_eq!(got.seven_day.unwrap().used_percentage, Some(8.0));
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path_for_email_in(dir.path(), "never-written@x.com");
        assert!(read_cached_at(&path).is_none());
    }

    #[test]
    fn read_corrupt_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path_for_email_in(dir.path(), "corrupt@x.com");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, b"\x00\x01garbage").unwrap();
        assert!(read_cached_at(&path).is_none());
    }

    #[test]
    fn roll_expired_no_op_for_future() {
        let now = 1_700_000_000;
        let mut rl = sample_rl(50.0, now + 3600);
        roll_expired_buckets(&mut rl, now);
        assert_eq!(rl.five_hour.unwrap().used_percentage, Some(50.0));
        assert_eq!(rl.five_hour.unwrap().resets_at, Some(now + 3600));
    }

    #[test]
    fn roll_expired_shifts_one_window() {
        let now = 1_700_000_000;
        let mut rl = sample_rl(50.0, now - 100);
        roll_expired_buckets(&mut rl, now);
        let five = rl.five_hour.unwrap();
        assert_eq!(five.used_percentage, Some(0.0));
        assert_eq!(five.resets_at, Some(now - 100 + FIVE_HOUR_SECS));
    }

    #[test]
    fn roll_expired_shifts_multiple_windows() {
        let now = 1_700_000_000;
        // bucket истёк 3 окна назад.
        let mut rl = sample_rl(50.0, now - 3 * FIVE_HOUR_SECS - 60);
        roll_expired_buckets(&mut rl, now);
        let five = rl.five_hour.unwrap();
        assert_eq!(five.used_percentage, Some(0.0));
        let resets = five.resets_at.unwrap();
        assert!(resets >= now, "resets_at must be in future");
        assert!(resets < now + FIVE_HOUR_SECS, "resets_at must be within next window");
    }

    #[test]
    fn roll_expired_handles_missing_resets_at() {
        let now = 1_700_000_000;
        let mut rl = RateLimits {
            five_hour: Some(RateBucket {
                used_percentage: Some(50.0),
                resets_at: None,
            }),
            seven_day: None,
        };
        roll_expired_buckets(&mut rl, now);
        let five = rl.five_hour.unwrap();
        assert_eq!(five.used_percentage, Some(50.0));
        assert!(five.resets_at.is_none());
    }

    #[test]
    fn roll_expired_seven_day_uses_7d_window() {
        let now = 1_700_000_000;
        let mut rl = RateLimits {
            five_hour: None,
            seven_day: Some(RateBucket {
                used_percentage: Some(80.0),
                resets_at: Some(now - 100),
            }),
        };
        roll_expired_buckets(&mut rl, now);
        let seven = rl.seven_day.unwrap();
        assert_eq!(seven.used_percentage, Some(0.0));
        assert_eq!(seven.resets_at, Some(now - 100 + SEVEN_DAY_SECS));
    }
}
