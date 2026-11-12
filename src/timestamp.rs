//! RFC 3339 timestamps in UTC, to whole seconds, such as `2026-11-11T20:00:00Z`.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A timestamp in the only form issue files use. Its text sorts chronologically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(String);

impl Timestamp {
    /// The current time, truncated to whole seconds.
    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        Self::from_unix(seconds)
    }

    /// The time `seconds` after the Unix epoch.
    pub fn from_unix(seconds: u64) -> Self {
        let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
        let (year, month, day) = civil_from_days(days);
        let second_of_day = seconds % 86_400;
        Self(format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
            second_of_day / 3600,
            second_of_day / 60 % 60,
            second_of_day % 60
        ))
    }

    /// Parses `YYYY-MM-DDTHH:MM:SSZ`, rejecting every other form.
    pub fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 20 {
            return None;
        }
        for (index, separator) in [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')] {
            if bytes[index] != separator {
                return None;
            }
        }
        if bytes[19] != b'Z' {
            return None;
        }
        let number = |start: usize, len: usize| -> Option<u32> {
            let digits = &value[start..start + len];
            digits
                .bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| digits.parse().ok())?
        };
        let year = number(0, 4)?;
        let month = number(5, 2)?;
        let day = number(8, 2)?;
        let valid = (1..=12).contains(&month)
            && (1..=days_in_month(year, month)).contains(&day)
            && number(11, 2)? <= 23
            && number(14, 2)? <= 59
            // RFC 3339 allows a leap second.
            && number(17, 2)? <= 60;
        valid.then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Converts days since 1970-01-01 to a proleptic Gregorian date, after Howard
/// Hinnant's `civil_from_days`.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_unix_times() {
        assert_eq!(Timestamp::from_unix(0).as_str(), "1970-01-01T00:00:00Z");
        assert_eq!(
            Timestamp::from_unix(951_825_599).as_str(),
            "2000-02-29T11:59:59Z"
        );
        assert_eq!(
            Timestamp::from_unix(1_794_430_800).as_str(),
            "2026-11-11T21:00:00Z"
        );
        assert_eq!(
            Timestamp::from_unix(4_107_542_399).as_str(),
            "2100-02-28T23:59:59Z"
        );
    }

    #[test]
    fn now_is_canonical() {
        let now = Timestamp::now();
        assert_eq!(Timestamp::parse(now.as_str()), Some(now));
    }

    #[test]
    fn parses_only_the_canonical_form() {
        for valid in [
            "2026-11-11T20:00:00Z",
            "2024-02-29T00:00:00Z",
            "2016-12-31T23:59:60Z",
        ] {
            assert!(Timestamp::parse(valid).is_some(), "{valid}");
        }
        for invalid in [
            "2026-11-11T20:00:00",
            "2026-11-11T20:00:00z",
            "2026-11-11t20:00:00Z",
            "2026-11-11 20:00:00Z",
            "2026-11-11T20:00:00.5Z",
            "2026-11-11T20:00:00+00:00",
            "2026-11-11",
            "2026-13-11T20:00:00Z",
            "2026-00-11T20:00:00Z",
            "2025-02-29T20:00:00Z",
            "2100-02-29T20:00:00Z",
            "2026-04-31T20:00:00Z",
            "2026-11-11T24:00:00Z",
            "2026-11-11T20:60:00Z",
            "2026-11-11T20:00:61Z",
            "+026-11-11T20:00:00Z",
            "2026-1a-11T20:00:00Z",
        ] {
            assert!(Timestamp::parse(invalid).is_none(), "{invalid}");
        }
    }
}
