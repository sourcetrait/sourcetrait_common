//! Extended traits for parsing chrono dates

use crate::*;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone};
use std::borrow::Cow;

/// Parse a date string relative to this one, with year and month
/// being optional.
///
/// Supports variants of Y-m-d and Y/m/D.
///
/// Implemented for `chrono::{NaiveDate, NaiveDateTime, and DateTime<TZ>}`.
pub trait DateRelativeParsing
where
    Self: Datelike,
{
    /// Parse a date string relative to this one, with year and month
    /// being optional.
    ///
    /// Supports variants of Y-m-d and Y/m/D.
    fn parse_relative_date(&self, date: &str) -> Option<NaiveDate> {
        const FORMATS: [(&'static str, Option<fn(i32, u32, &str) -> String>); 5] = [
            (FMT_YMD_DASH, None),
            (FMT_YMD_SLASH, None),
            (FMT_YMD_DASH, Some(|y, _, s| format!("{y}-{s}"))), // m-d
            (FMT_YMD_SLASH, Some(|y, _, s| format!("{y}/{s}"))), // m/d
            (FMT_YMD_DASH, Some(|y, m, s| format!("{y}-{m}-{s}"))), // d
        ];

        //let now = Local::now().naive_local();
        let (year, month) = (self.year(), self.month());

        for (format, format_fn) in FORMATS {
            let s: Cow<'_, str> = format_fn
                .map(|f| Cow::Owned(f(year, month, date)))
                .unwrap_or(Cow::Borrowed(date));

            if let Ok(date) = NaiveDate::parse_from_str(&s, format) {
                return Some(date);
            }
        }

        None
    }
}

impl DateRelativeParsing for NaiveDate {}
impl DateRelativeParsing for NaiveDateTime {}
impl<TZ: TimeZone> DateRelativeParsing for DateTime<TZ> {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, Utc};

    #[test]
    fn test_local_relative() {
        let now = Local::now();

        assert_eq!(
            NaiveDate::from_ymd_opt(2025, 3, 2),
            now.parse_relative_date("2025-03-02")
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2025, 3, 2),
            now.parse_relative_date("2025/3/2")
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(now.year(), 4, 5),
            now.parse_relative_date("04-05")
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(now.year(), 4, 5),
            now.parse_relative_date("4/5")
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(now.year(), now.month(), 5),
            now.parse_relative_date("5")
        );
    }

    #[test]
    fn test_utc_relative() {
        let now = Utc::now();

        assert_eq!(
            NaiveDate::from_ymd_opt(now.year(), 4, 5),
            now.parse_relative_date("04-05")
        );
    }
}
