//! Auto-unit byte formatter (Kb/Mb/Gb/Tb).

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)]

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;
const TB: u64 = 1024 * GB;

/// Auto-unit byte formatter (b/k/M/G/T).
/// Drops trailing zeros: `2.0G → 2G`, `7.50G → 7.5G`.
#[must_use]
pub fn format(bytes: u64) -> String {
    if bytes < KB {
        return format!("{bytes}b");
    }
    #[allow(clippy::cast_precision_loss)]
    let (divisor, unit, next_unit): (f64, &str, Option<&str>) = if bytes < MB {
        (KB as f64, "k", Some("M"))
    } else if bytes < GB {
        (MB as f64, "M", Some("G"))
    } else if bytes < TB {
        (GB as f64, "G", Some("T"))
    } else {
        (TB as f64, "T", None)
    };
    #[allow(clippy::cast_precision_loss)]
    let rounded = (bytes as f64 / divisor * 10.0).round() / 10.0;
    if rounded >= 1024.0 {
        if let Some(nu) = next_unit {
            return format!("1{nu}");
        }
    }
    if rounded.fract() == 0.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = rounded as u64;
        format!("{n}{unit}")
    } else {
        format!("{rounded:.1}{unit}")
    }
}

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
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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

    #[test]
    fn boundary_rounding_mb_minus_1() {
        // 1048575 / 1024 = 1023.999…, rounds to 1024.0 → must show "1M" not "1024k"
        assert_eq!(format(MB - 1), "1M");
    }

    #[test]
    fn boundary_rounding_gb_minus_1() {
        assert_eq!(format(GB - 1), "1G");
    }

    #[test]
    fn u64_max_does_not_panic() {
        // u64::MAX / TB ≈ 16_777_216; no higher unit, so stays in T
        assert_eq!(format(u64::MAX), "16777216T");
    }
}
