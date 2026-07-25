//! # GTFS Time Values
//!
//! GTFS expresses times as `HH:MM:SS` seconds since midnight of the
//! service day. Hours may exceed 23 for trips running past midnight
//! (e.g. `25:10:00`).

use crate::error::GtfsError;

/// Parses a GTFS `HH:MM:SS` time into seconds since midnight.
///
/// Hours may exceed 23 for services running past midnight. Convenient
/// for keeping human-readable times in hardcoded datasets.
///
/// # Arguments
///
/// * `value` - Time string in `HH:MM:SS` or `H:MM:SS` form
///
/// # Errors
///
/// Returns [`GtfsError::InvalidTime`] if the value is not a valid
/// `HH:MM:SS` time.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::parse_gtfs_time;
///
///     assert_eq!(parse_gtfs_time("08:00:00")?, 8 * 3600);
///     assert_eq!(parse_gtfs_time("25:10:30")?, 25 * 3600 + 10 * 60 + 30);
///     assert!(parse_gtfs_time("8am").is_err());
///     Ok(())
/// }
/// ```
pub fn parse_gtfs_time(value: &str) -> Result<u32, GtfsError> {
    let invalid = || GtfsError::InvalidTime {
        value: value.to_string(),
    };
    let mut parts = value.trim().split(':');
    let hours: u32 = parts
        .next()
        .and_then(|p| p.parse().ok())
        .ok_or_else(invalid)?;
    let minutes: u32 = parts
        .next()
        .and_then(|p| p.parse().ok())
        .ok_or_else(invalid)?;
    let seconds: u32 = parts
        .next()
        .and_then(|p| p.parse().ok())
        .ok_or_else(invalid)?;
    if parts.next().is_some() || minutes > 59 || seconds > 59 {
        return Err(invalid());
    }
    Ok(hours * 3600 + minutes * 60 + seconds)
}

/// Formats seconds since midnight as a GTFS `HH:MM:SS` time.
///
/// Hours may exceed 23 for services running past midnight.
///
/// # Arguments
///
/// * `seconds` - Seconds since midnight of the service day
///
/// # Examples
///
/// ```
/// use gtfs_rs::format_gtfs_time;
///
/// assert_eq!(format_gtfs_time(8 * 3600), "08:00:00");
/// assert_eq!(format_gtfs_time(90630), "25:10:30");
/// ```
pub fn format_gtfs_time(seconds: u32) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gtfs_time() {
        assert_eq!(format_gtfs_time(0), "00:00:00");
        assert_eq!(format_gtfs_time(86399), "23:59:59");
        assert_eq!(format_gtfs_time(90630), "25:10:30");
    }

    #[test]
    fn test_parse_gtfs_time() -> Result<(), GtfsError> {
        assert_eq!(parse_gtfs_time("00:00:00")?, 0);
        assert_eq!(parse_gtfs_time("8:05:00")?, 8 * 3600 + 5 * 60);
        assert_eq!(parse_gtfs_time("23:59:59")?, 86399);
        // service past midnight
        assert_eq!(parse_gtfs_time("25:10:30")?, 90630);
        assert!(parse_gtfs_time("").is_err());
        assert!(parse_gtfs_time("12:60:00").is_err());
        assert!(parse_gtfs_time("12:00").is_err());
        assert!(parse_gtfs_time("12:00:00:00").is_err());
        Ok(())
    }
}
