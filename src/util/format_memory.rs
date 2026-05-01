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
        return std::format!("{bytes}b");
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
        std::format!("{}{unit}", rounded.trunc() as u64)
    } else {
        std::format!("{rounded:.1}{unit}")
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
