//! `stop_times.txt` writer.

use crate::misc::format_gtfs_time;
use crate::model::StopTime;
use crate::writers::csv::CsvWrite;

impl CsvWrite for StopTime {
    const FILE_NAME: &'static str = "stop_times.txt";

    const HEADER: &'static [&'static str] = &[
        "trip_id",
        "arrival_time",
        "departure_time",
        "stop_id",
        "location_group_id",
        "location_id",
        "stop_sequence",
        "stop_headsign",
        "start_pickup_drop_off_window",
        "end_pickup_drop_off_window",
        "pickup_type",
        "drop_off_type",
        "continuous_pickup",
        "continuous_drop_off",
        "shape_dist_traveled",
        "timepoint",
        "pickup_booking_rule_id",
        "drop_off_booking_rule_id",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.trip_id.clone(),
            self.arrival_time.map(format_gtfs_time).unwrap_or_default(),
            self.departure_time
                .map(format_gtfs_time)
                .unwrap_or_default(),
            self.stop_id.clone().unwrap_or_default(),
            self.location_group_id.clone().unwrap_or_default(),
            self.location_id.clone().unwrap_or_default(),
            self.stop_sequence.to_string(),
            self.stop_headsign.clone().unwrap_or_default(),
            self.start_pickup_drop_off_window
                .map(format_gtfs_time)
                .unwrap_or_default(),
            self.end_pickup_drop_off_window
                .map(format_gtfs_time)
                .unwrap_or_default(),
            self.pickup_type.code().to_string(),
            self.drop_off_type.code().to_string(),
            self.continuous_pickup.code().to_string(),
            self.continuous_drop_off.code().to_string(),
            self.shape_dist_traveled
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.timepoint.code().to_string(),
            self.pickup_booking_rule_id.clone().unwrap_or_default(),
            self.drop_off_booking_rule_id.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "parse")]
    use crate::model::PickupDropOffType;
    use crate::model::StopTime;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let st = StopTime::new("t0", "A", 1, 8 * 3600);
        let fields = st.fields();
        assert_eq!(fields.len(), StopTime::HEADER.len());
        // arrival_time formatted as HH:MM:SS
        assert_eq!(fields[1], "08:00:00");
        // stop_sequence as plain integer
        assert_eq!(fields[6], "1");
        // pickup_type default Regular -> code 0
        assert_eq!(fields[10], "0");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let fixed = StopTime::new("t0", "A", 1, 8 * 3600)
            .with_times(8 * 3600, 8 * 3600 + 30)
            .with_headsign("Downtown");

        let mut flex = StopTime::new("t0", "A", 2, 0);
        flex.stop_id = None;
        flex.arrival_time = None;
        flex.departure_time = None;
        flex.location_id = Some("zone_a".to_string());
        flex.start_pickup_drop_off_window = Some(9 * 3600);
        flex.end_pickup_drop_off_window = Some(17 * 3600);
        flex.pickup_type = PickupDropOffType::PhoneAgency;
        flex.drop_off_type = PickupDropOffType::PhoneAgency;

        let rows = vec![fixed, flex];
        let mut out = Vec::new();
        write("stop_times.txt", &rows, &mut out)?;
        let parsed: Vec<StopTime> = read("stop_times.txt", out.as_slice())?;

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].trip_id, "t0");
        assert_eq!(parsed[0].arrival_time, Some(8 * 3600));
        assert_eq!(parsed[0].departure_time, Some(8 * 3600 + 30));
        assert_eq!(parsed[0].stop_id.as_deref(), Some("A"));
        assert_eq!(parsed[0].stop_headsign.as_deref(), Some("Downtown"));
        assert!(parsed[1].stop_id.is_none());
        assert!(parsed[1].arrival_time.is_none());
        assert_eq!(parsed[1].location_id.as_deref(), Some("zone_a"));
        assert_eq!(parsed[1].start_pickup_drop_off_window, Some(9 * 3600));
        assert_eq!(parsed[1].end_pickup_drop_off_window, Some(17 * 3600));
        assert_eq!(parsed[1].pickup_type, PickupDropOffType::PhoneAgency);
        Ok(())
    }
}
