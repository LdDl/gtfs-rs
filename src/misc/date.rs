//! # GTFS Service Dates
//!
//! GTFS expresses dates as `YYYYMMDD` in the local timezone of the
//! agency (e.g. `20260724`). Used by `calendar.txt`,
//! `calendar_dates.txt` and `feed_info.txt`.

use std::fmt;

use crate::error::GtfsError;

/// A day of the week, as used by `calendar.txt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday
    Monday,
    /// Tuesday
    Tuesday,
    /// Wednesday
    Wednesday,
    /// Thursday
    Thursday,
    /// Friday
    Friday,
    /// Saturday
    Saturday,
    /// Sunday
    Sunday,
}

/// A GTFS service date (`YYYYMMDD`).
///
/// Field order gives the derived ordering calendar semantics:
/// dates compare chronologically.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::{GtfsDate, Weekday};
///
///     let date = GtfsDate::parse("20260724")?;
///     assert_eq!(date.weekday(), Weekday::Friday);
///     assert_eq!(date.to_string(), "20260724");
///     assert!(date < GtfsDate::new(2026, 12, 31)?);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GtfsDate {
    /// Four-digit year
    pub year: u16,
    /// Month, 1-12
    pub month: u8,
    /// Day of month, 1-31
    pub day: u8,
}

impl GtfsDate {
    /// Creates a date, validating that it exists in the Gregorian
    /// calendar (including leap years).
    ///
    /// # Arguments
    ///
    /// * `year` - Four-digit year
    /// * `month` - Month, 1-12
    /// * `day` - Day of month
    ///
    /// # Errors
    ///
    /// Returns [`GtfsError::InvalidDate`] if the combination is not
    /// an existing calendar date.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::GtfsDate;
    ///
    ///     let date = GtfsDate::new(2026, 7, 24)?;
    ///     assert_eq!(date.to_string(), "20260724");
    ///     assert!(GtfsDate::new(2026, 2, 30).is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, GtfsError> {
        if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
            return Err(GtfsError::InvalidDate {
                value: format!("{:04}{:02}{:02}", year, month, day),
            });
        }
        Ok(GtfsDate { year, month, day })
    }

    /// Parses a GTFS `YYYYMMDD` date string.
    ///
    /// # Arguments
    ///
    /// * `value` - Date string, exactly eight digits
    ///
    /// # Errors
    ///
    /// Returns [`GtfsError::InvalidDate`] if the string is not eight
    /// digits or encodes a nonexistent date.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::GtfsDate;
    ///
    ///     let date = GtfsDate::parse("20260724")?;
    ///     assert_eq!(date, GtfsDate::new(2026, 7, 24)?);
    ///     assert!(GtfsDate::parse("2026-07-24").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn parse(value: &str) -> Result<Self, GtfsError> {
        let invalid = || GtfsError::InvalidDate {
            value: value.to_string(),
        };
        let v = value.trim();
        if v.len() != 8 || !v.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let year: u16 = v[0..4].parse().map_err(|_| invalid())?;
        let month: u8 = v[4..6].parse().map_err(|_| invalid())?;
        let day: u8 = v[6..8].parse().map_err(|_| invalid())?;
        GtfsDate::new(year, month, day).map_err(|_| invalid())
    }

    /// Returns the day of the week (Sakamoto's algorithm).
    /// Ref: <https://www.geeksforgeeks.org/dsa/tomohiko-sakamotos-algorithm-finding-day-week/>
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::{GtfsDate, Weekday};
    ///
    ///     let date = GtfsDate::new(2026, 7, 24)?;
    ///     assert_eq!(date.weekday(), Weekday::Friday);
    ///     Ok(())
    /// }
    /// ```
    pub fn weekday(&self) -> Weekday {
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = i32::from(self.year);
        let m = self.month as usize;
        if m < 3 {
            y -= 1;
        }
        // 0 = Sunday
        let dow = (y + y / 4 - y / 100 + y / 400 + T[m - 1] + i32::from(self.day)) % 7;
        match dow {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }
}

impl fmt::Display for GtfsDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}{:02}{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_validate() -> Result<(), GtfsError> {
        assert_eq!(GtfsDate::parse("20260724")?, GtfsDate::new(2026, 7, 24)?);
        // leap year handling
        assert!(GtfsDate::parse("20240229").is_ok());
        assert!(GtfsDate::parse("20250229").is_err());
        assert!(GtfsDate::parse("20260732").is_err());
        assert!(GtfsDate::parse("20261301").is_err());
        assert!(GtfsDate::parse("2026-07-24").is_err());
        assert!(GtfsDate::parse("202607").is_err());
        Ok(())
    }

    #[test]
    fn test_weekday() -> Result<(), GtfsError> {
        assert_eq!(GtfsDate::new(2026, 7, 24)?.weekday(), Weekday::Friday);
        assert_eq!(GtfsDate::new(2026, 7, 26)?.weekday(), Weekday::Sunday);
        assert_eq!(GtfsDate::new(2024, 2, 29)?.weekday(), Weekday::Thursday);
        assert_eq!(GtfsDate::new(2000, 1, 1)?.weekday(), Weekday::Saturday);
        Ok(())
    }

    #[test]
    fn test_ordering() -> Result<(), GtfsError> {
        let a = GtfsDate::new(2025, 12, 31)?;
        let b = GtfsDate::new(2026, 1, 1)?;
        assert!(a < b);
        Ok(())
    }
}
