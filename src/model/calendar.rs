//! `calendar.txt` and `calendar_dates.txt` - weekly service patterns
//! and their explicit date exceptions.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#calendartxt>

use crate::misc::{GtfsDate, Weekday};

/// A weekly service pattern from `calendar.txt`.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::{Calendar, GtfsDate};
///
///     let service = Calendar::new(
///         "weekday",
///         GtfsDate::new(2026, 1, 1)?,
///         GtfsDate::new(2026, 12, 31)?,
///     )
///     .with_weekdays();
///
///     // 2026-07-24 is a Friday, 2026-07-26 a Sunday
///     assert!(service.is_active_on(&GtfsDate::new(2026, 7, 24)?));
///     assert!(!service.is_active_on(&GtfsDate::new(2026, 7, 26)?));
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Calendar {
    /// Identifies a set of dates when service is available for one
    /// or more routes. Unique ID, referenced by `trips.service_id`.
    /// Required.
    pub service_id: String,
    /// Indicates whether the service operates on all Mondays in the
    /// date range specified by the `start_date` and `end_date`
    /// fields. Note that exceptions for particular dates may be
    /// listed in `calendar_dates.txt`. Required. Stored as `bool`:
    /// `true` (file value `1`) - service is available for all
    /// Mondays in the date range; `false` (file value `0`) - service
    /// is not available for Mondays in the date range.
    pub monday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Tuesdays. Required. Stored as `bool` (file values `1`/`0`).
    pub tuesday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Wednesdays. Required. Stored as `bool` (file values `1`/`0`).
    pub wednesday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Thursdays. Required. Stored as `bool` (file values `1`/`0`).
    pub thursday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Fridays. Required. Stored as `bool` (file values `1`/`0`).
    pub friday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Saturdays. Required. Stored as `bool` (file values `1`/`0`).
    pub saturday: bool,
    /// Functions in the same way as `monday` except applies to
    /// Sundays. Required. Stored as `bool` (file values `1`/`0`).
    pub sunday: bool,
    /// Start service day for the service interval. This service day
    /// is included in the interval. Required. Stored as
    /// [`GtfsDate`] (file format `YYYYMMDD`).
    pub start_date: GtfsDate,
    /// End service day for the service interval. This service day is
    /// included in the interval. Required. Stored as [`GtfsDate`]
    /// (file format `YYYYMMDD`).
    pub end_date: GtfsDate,
}

impl Calendar {
    /// Creates a service pattern with no active days.
    ///
    /// # Arguments
    ///
    /// * `service_id` - Unique service identifier
    /// * `start_date` - First service day (inclusive)
    /// * `end_date` - Last service day (inclusive)
    pub fn new(service_id: &str, start_date: GtfsDate, end_date: GtfsDate) -> Self {
        Calendar {
            service_id: service_id.to_string(),
            monday: false,
            tuesday: false,
            wednesday: false,
            thursday: false,
            friday: false,
            saturday: false,
            sunday: false,
            start_date,
            end_date,
        }
    }

    /// Activates Monday through Friday.
    pub fn with_weekdays(mut self) -> Self {
        self.monday = true;
        self.tuesday = true;
        self.wednesday = true;
        self.thursday = true;
        self.friday = true;
        self
    }

    /// Activates Saturday and Sunday.
    pub fn with_weekends(mut self) -> Self {
        self.saturday = true;
        self.sunday = true;
        self
    }

    /// Activates all seven days.
    pub fn with_all_days(self) -> Self {
        self.with_weekdays().with_weekends()
    }

    /// Returns whether the service runs on the given day of the week.
    pub fn runs_on(&self, weekday: Weekday) -> bool {
        match weekday {
            Weekday::Monday => self.monday,
            Weekday::Tuesday => self.tuesday,
            Weekday::Wednesday => self.wednesday,
            Weekday::Thursday => self.thursday,
            Weekday::Friday => self.friday,
            Weekday::Saturday => self.saturday,
            Weekday::Sunday => self.sunday,
        }
    }

    /// Returns whether the date falls in the service period and on an
    /// active day of the week. Exceptions from `calendar_dates.txt`
    /// are not considered; see
    /// [`GtfsReference::is_service_active`](crate::GtfsReference::is_service_active).
    pub fn is_active_on(&self, date: &GtfsDate) -> bool {
        *date >= self.start_date && *date <= self.end_date && self.runs_on(date.weekday())
    }
}

gtfs_enum! {
    /// Type of a service exception (`exception_type` in
    /// `calendar_dates.txt`). Indicates whether service is available
    /// on the date specified in the `date` field.
    ///
    /// Example: suppose a route has one set of trips available on
    /// holidays and another set of trips available on all other
    /// days. One `service_id` could correspond to the regular
    /// service schedule and another `service_id` could correspond to
    /// the holiday schedule. For a particular holiday,
    /// `calendar_dates.txt` could be used to add the holiday to the
    /// holiday `service_id` and to remove the holiday from the
    /// regular `service_id` schedule.
    ExceptionType {
        /// Service has been added for the specified date (`1`)
        Added = 1,
        /// Service has been removed for the specified date (`2`)
        Removed = 2,
    }
}

/// A service exception from `calendar_dates.txt`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CalendarDate {
    /// Identifies a set of dates when a service exception occurs
    /// for one or more routes. Foreign ID referencing
    /// `calendar.service_id` when `calendar.txt` is used, or an ID
    /// when `calendar.txt` is omitted. Each (`service_id`, `date`)
    /// pair may only appear once in `calendar_dates.txt` if using
    /// `calendar.txt` and `calendar_dates.txt` in conjunction. If a
    /// `service_id` value appears in both `calendar.txt` and
    /// `calendar_dates.txt`, the information in
    /// `calendar_dates.txt` modifies the service information
    /// specified in `calendar.txt`. Required.
    pub service_id: String,
    /// Date when service exception occurs. Required. Stored as
    /// [`GtfsDate`] (file format `YYYYMMDD`).
    pub date: GtfsDate,
    /// Indicates whether service is available on the date specified
    /// in the `date` field; see [`ExceptionType`] for the values
    /// (service added or removed for the date). Required.
    pub exception_type: ExceptionType,
}

impl CalendarDate {
    /// Creates a service exception.
    ///
    /// # Arguments
    ///
    /// * `service_id` - Service the exception applies to
    /// * `date` - Date of the exception
    /// * `exception_type` - Added or removed
    pub fn new(service_id: &str, date: GtfsDate, exception_type: ExceptionType) -> Self {
        CalendarDate {
            service_id: service_id.to_string(),
            date,
            exception_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GtfsError;

    #[test]
    fn test_calendar_activity() -> Result<(), GtfsError> {
        let cal = Calendar::new(
            "wd",
            GtfsDate::new(2026, 7, 1)?,
            GtfsDate::new(2026, 7, 31)?,
        )
        .with_weekdays();

        // 2026-07-24 is a Friday, 2026-07-26 a Sunday
        assert!(cal.is_active_on(&GtfsDate::new(2026, 7, 24)?));
        assert!(!cal.is_active_on(&GtfsDate::new(2026, 7, 26)?));
        // outside the service period
        assert!(!cal.is_active_on(&GtfsDate::new(2026, 8, 3)?));
        Ok(())
    }
}
