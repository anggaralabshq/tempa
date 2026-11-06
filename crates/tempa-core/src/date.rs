//! UTC calendar date without a date crate.

use std::time::{SystemTime, UNIX_EPOCH};

/// Convert days since 1970-01-01 to (year, month, day).
/// Howard Hinnant's `civil_from_days` algorithm.
pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    ((if m <= 2 { y + 1 } else { y }) as i32, m as u32, d as u32)
}

pub fn today_utc() -> (i32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil_from_days((secs / 86_400) as i64)
}

/// `YYYY-MM-DD`
pub fn iso_today() -> String {
    let (y, m, d) = today_utc();
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYYMMDD`
pub fn compact_today() -> String {
    let (y, m, d) = today_utc();
    format!("{y:04}{m:02}{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_398), (2025, 11, 6));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn date_formats() {
        assert_eq!(iso_today().len(), 10);
        assert_eq!(compact_today().len(), 8);
    }
}
