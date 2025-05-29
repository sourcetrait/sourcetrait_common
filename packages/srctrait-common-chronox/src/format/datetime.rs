//! Common data/time formatting options

use chrono::NaiveDateTime;

pub const FMT_LONG: &'static str = "%A, %B %d, %Y";
pub const FMT_YMD_DASH: &'static str = "%Y-%m-%d";
pub const FMT_YMD_SLASH: &'static str = "%Y/%m/%d";
pub const FMT_YM_DASH: &'static str = "%Y-%m";
pub const FMT_YM_SLASH: &'static str = "%Y/%m";
pub const FMT_MD_DASH: &'static str = "%m-%d";
pub const FMT_MD_SLASH: &'static str = "%m/%d";
pub const FMT_YEAR: &'static str = "%Y";
pub const FMT_MONTH: &'static str = "%m";
pub const FMT_DAY: &'static str = "%d";

/// Date and time formatting options for Y-m-d and Y/m/d variants
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DateTimeFormat {
    /// eg, Wednesday, March 3, 2025
    Long,
    /// Year-Month-Day eg, 2025-03-12
    YmdDash,
    /// Year/Month/Day eg, 2025/03/12
    YmdSlash,
    /// Year-Month eg, 2025-03
    YmDash,
    /// Year/Month eg, 2025/03
    YmSlash,
    /// Month-Day eg, 03-12
    MdDash,
    /// Month/Day eg, 03/12
    MdSlash,
    /// Year eg, 2025
    Year,
    /// Month eg, 03
    Month,
    /// Day eg, 12
    Day
}

impl DateTimeFormat {
    /// The strftime format for this kind
    pub fn strftime_format(&self) -> &'static str {
        match self {
            DateTimeFormat::Long => FMT_LONG,
            DateTimeFormat::YmdDash => FMT_YMD_DASH,
            DateTimeFormat::YmdSlash => FMT_YMD_SLASH,
            DateTimeFormat::YmDash => FMT_YM_DASH,
            DateTimeFormat::YmSlash => FMT_YM_SLASH,
            DateTimeFormat::MdDash => FMT_MD_DASH,
            DateTimeFormat::MdSlash => FMT_MD_SLASH,
            DateTimeFormat::Year => FMT_YEAR,
            DateTimeFormat::Month => FMT_MONTH,
            DateTimeFormat::Day => FMT_DAY,
        }
    }
}

/// Provides Display for a date/time and a format
#[derive(Debug)]
pub struct DateTimeDisplay(NaiveDateTime, DateTimeFormat);

impl DateTimeDisplay {
    pub fn new(datetime: NaiveDateTime, format: DateTimeFormat) -> Self {
        Self ( datetime, format )
    }

    pub fn date(&self) -> &NaiveDateTime {
        &self.0
    }

    pub fn format(&self) -> &DateTimeFormat {
        &self.1
    }
}

impl std::fmt::Display for DateTimeDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(self.1.strftime_format()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use super::*;

    #[test]
    fn test_display() {
        let utc = Utc.with_ymd_and_hms(1974, 5, 6, 7, 8, 9).unwrap().naive_utc();

        assert_eq!("1974", DateTimeDisplay::new(utc, DateTimeFormat::Year).to_string());
        assert_eq!("06", DateTimeDisplay::new(utc, DateTimeFormat::Day).to_string());
        assert_eq!("1974-05-06", DateTimeDisplay::new(utc, DateTimeFormat::YmdDash).to_string());
        assert_eq!("1974/05/06", DateTimeDisplay::new(utc, DateTimeFormat::YmdSlash).to_string());
        assert_eq!("1974/05", DateTimeDisplay::new(utc, DateTimeFormat::YmSlash).to_string());
        assert_eq!("05/06", DateTimeDisplay::new(utc, DateTimeFormat::MdSlash).to_string());
    }
}
