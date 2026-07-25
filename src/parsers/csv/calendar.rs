//! `calendar.txt` and `calendar_dates.txt` readers.

use crate::model::{Calendar, CalendarDate, ExceptionType};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for Calendar {
    const FILE_NAME: &'static str = "calendar.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut calendar = Calendar::new(
            row.req("service_id")?,
            row.req_date("start_date")?,
            row.req_date("end_date")?,
        );
        calendar.monday = row.req_bool01("monday")?;
        calendar.tuesday = row.req_bool01("tuesday")?;
        calendar.wednesday = row.req_bool01("wednesday")?;
        calendar.thursday = row.req_bool01("thursday")?;
        calendar.friday = row.req_bool01("friday")?;
        calendar.saturday = row.req_bool01("saturday")?;
        calendar.sunday = row.req_bool01("sunday")?;
        Ok(calendar)
    }
}

impl CsvRecord for CalendarDate {
    const FILE_NAME: &'static str = "calendar_dates.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        Ok(CalendarDate::new(
            row.req("service_id")?,
            row.req_date("date")?,
            row.req_code("exception_type", ExceptionType::from_code, "code 1-2")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::{Calendar, CalendarDate, ExceptionType};
    use crate::parsers::ParseError;
    use crate::parsers::csv::{
        read_path,
        test_support::{feed_file, model},
    };

    #[test]
    fn test_sample_calendar() -> Result<(), ParseError> {
        let calendar: Vec<Calendar> = read_path(feed_file("calendar.txt"))?;
        assert_eq!(calendar.len(), 2);
        let fullw = &calendar[0];
        assert_eq!(fullw.service_id, "FULLW");
        assert!(fullw.monday && fullw.sunday);
        assert_eq!(fullw.start_date, GtfsDate::new(2007, 1, 1).map_err(model)?);
        let weekend = &calendar[1];
        assert_eq!(weekend.service_id, "WE");
        assert!(!weekend.monday && weekend.saturday && weekend.sunday);
        Ok(())
    }

    #[test]
    fn test_sample_calendar_dates() -> Result<(), ParseError> {
        let dates: Vec<CalendarDate> = read_path(feed_file("calendar_dates.txt"))?;
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0].service_id, "FULLW");
        assert_eq!(dates[0].date, GtfsDate::new(2007, 6, 4).map_err(model)?);
        assert_eq!(dates[0].exception_type, ExceptionType::Removed);
        Ok(())
    }
}
