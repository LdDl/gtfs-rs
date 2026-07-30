//! `frequencies.txt` writer.

use crate::misc::format_gtfs_time;
use crate::model::Frequency;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Frequency {
    const FILE_NAME: &'static str = "frequencies.txt";

    const HEADER: &'static [&'static str] = &[
        "trip_id",
        "start_time",
        "end_time",
        "headway_secs",
        "exact_times",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.trip_id.clone(),
            format_gtfs_time(self.start_time),
            format_gtfs_time(self.end_time),
            self.headway_secs.to_string(),
            self.exact_times.code().to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Frequency;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let freq = Frequency::new("t0", 8 * 3600, 10 * 3600, 300).with_exact_times();
        let fields = freq.fields();
        assert_eq!(fields.len(), Frequency::HEADER.len());
        // start_time formatted as HH:MM:SS
        assert_eq!(fields[1], "08:00:00");
        // headway_secs as plain integer
        assert_eq!(fields[3], "300");
        // exact_times ScheduleBased -> code 1
        assert_eq!(fields[4], "1");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let rows = vec![
            Frequency::new("t0", 7 * 3600, 10 * 3600, 300),
            Frequency::new("t1", 25 * 3600, 26 * 3600, 600).with_exact_times(),
        ];
        let mut out = Vec::new();
        write("frequencies.txt", &rows, &mut out)?;
        let parsed: Vec<Frequency> = read("frequencies.txt", out.as_slice())?;

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].trip_id, "t0");
        assert_eq!(parsed[0].start_time, 7 * 3600);
        assert_eq!(parsed[0].end_time, 10 * 3600);
        assert_eq!(parsed[0].headway_secs, 300);
        assert_eq!(parsed[0].exact_times, rows[0].exact_times);
        // past-midnight time survives the roundtrip
        assert_eq!(parsed[1].start_time, 25 * 3600);
        assert_eq!(parsed[1].exact_times, rows[1].exact_times);
        Ok(())
    }
}
