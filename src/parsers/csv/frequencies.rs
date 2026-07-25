//! `frequencies.txt` reader.

use crate::model::{ExactTimes, Frequency};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for Frequency {
    const FILE_NAME: &'static str = "frequencies.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut frequency = Frequency::new(
            row.req("trip_id")?,
            row.req_time("start_time")?,
            row.req_time("end_time")?,
            row.req_num("headway_secs", "seconds")?,
        );
        frequency.exact_times = row
            .opt_code("exact_times", ExactTimes::from_code, "code 0-1")?
            .unwrap_or_default();
        Ok(frequency)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{ExactTimes, Frequency};
    use crate::parsers::ParseError;
    use crate::parsers::csv::{read_path, test_support::feed_file};

    #[test]
    fn test_sample_frequencies() -> Result<(), ParseError> {
        let frequencies: Vec<Frequency> = read_path(feed_file("frequencies.txt"))?;
        assert_eq!(frequencies.len(), 11);
        assert_eq!(frequencies[0].trip_id, "STBA");
        assert_eq!(frequencies[0].start_time, 6 * 3600);
        assert_eq!(frequencies[0].end_time, 22 * 3600);
        assert_eq!(frequencies[0].headway_secs, 1800);
        assert_eq!(frequencies[0].exact_times, ExactTimes::FrequencyBased);
        Ok(())
    }
}
