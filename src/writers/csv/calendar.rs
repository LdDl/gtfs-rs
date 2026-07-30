//! `calendar.txt` and `calendar_dates.txt` writers.

use crate::model::{Calendar, CalendarDate};
use crate::writers::csv::CsvWrite;

/// Encodes a weekday availability flag as the file values `1`/`0`.
fn bool01(value: bool) -> String {
    if value { "1" } else { "0" }.to_string()
}

impl CsvWrite for Calendar {
    const FILE_NAME: &'static str = "calendar.txt";

    const HEADER: &'static [&'static str] = &[
        "service_id",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "start_date",
        "end_date",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.service_id.clone(),
            bool01(self.monday),
            bool01(self.tuesday),
            bool01(self.wednesday),
            bool01(self.thursday),
            bool01(self.friday),
            bool01(self.saturday),
            bool01(self.sunday),
            self.start_date.to_string(),
            self.end_date.to_string(),
        ]
    }
}

impl CsvWrite for CalendarDate {
    const FILE_NAME: &'static str = "calendar_dates.txt";

    const HEADER: &'static [&'static str] = &["service_id", "date", "exception_type"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.service_id.clone(),
            self.date.to_string(),
            self.exception_type.code().to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::{Calendar, CalendarDate, ExceptionType};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() -> Result<(), Box<dyn std::error::Error>> {
        let cal = Calendar::new(
            "svc",
            GtfsDate::new(2026, 1, 1)?,
            GtfsDate::new(2026, 12, 31)?,
        )
        .with_weekdays();
        let fields = cal.fields();
        assert_eq!(fields.len(), Calendar::HEADER.len());
        // monday active -> "1", saturday inactive -> "0"
        assert_eq!(fields[1], "1");
        assert_eq!(fields[6], "0");
        // start_date formatted as YYYYMMDD
        assert_eq!(fields[8], "20260101");

        let date = CalendarDate::new("svc", GtfsDate::new(2026, 1, 1)?, ExceptionType::Removed);
        let fields = date.fields();
        assert_eq!(fields.len(), CalendarDate::HEADER.len());
        assert_eq!(fields[1], "20260101");
        // exception_type Removed -> code 2
        assert_eq!(fields[2], "2");
        Ok(())
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let calendars = vec![
            Calendar::new(
                "wd",
                GtfsDate::new(2026, 1, 1)?,
                GtfsDate::new(2026, 12, 31)?,
            )
            .with_weekdays(),
        ];
        let mut out = Vec::new();
        write("calendar.txt", &calendars, &mut out)?;
        let parsed: Vec<Calendar> = read("calendar.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].service_id, "wd");
        assert!(parsed[0].monday && parsed[0].friday);
        assert!(!parsed[0].saturday && !parsed[0].sunday);
        assert_eq!(parsed[0].start_date, GtfsDate::new(2026, 1, 1)?);
        assert_eq!(parsed[0].end_date, GtfsDate::new(2026, 12, 31)?);

        let dates = vec![CalendarDate::new(
            "wd",
            GtfsDate::new(2026, 1, 7)?,
            ExceptionType::Added,
        )];
        let mut out = Vec::new();
        write("calendar_dates.txt", &dates, &mut out)?;
        let parsed: Vec<CalendarDate> = read("calendar_dates.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].service_id, "wd");
        assert_eq!(parsed[0].date, GtfsDate::new(2026, 1, 7)?);
        assert_eq!(parsed[0].exception_type, ExceptionType::Added);
        Ok(())
    }
}
