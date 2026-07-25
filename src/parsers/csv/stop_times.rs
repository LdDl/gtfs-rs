//! `stop_times.txt` reader.

use super::{CsvRecord, Row, opt_string};
use crate::model::{ContinuousPickupDropOff, PickupDropOffType, StopTime, Timepoint};
use crate::parsers::ParseError;

impl CsvRecord for StopTime {
    const FILE_NAME: &'static str = "stop_times.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        Ok(StopTime {
            trip_id: row.req("trip_id")?.to_string(),
            arrival_time: row.opt_time("arrival_time")?,
            departure_time: row.opt_time("departure_time")?,
            stop_id: opt_string(row, "stop_id"),
            location_group_id: opt_string(row, "location_group_id"),
            location_id: opt_string(row, "location_id"),
            stop_sequence: row.req_num("stop_sequence", "a non-negative integer")?,
            stop_headsign: opt_string(row, "stop_headsign"),
            start_pickup_drop_off_window: row.opt_time("start_pickup_drop_off_window")?,
            end_pickup_drop_off_window: row.opt_time("end_pickup_drop_off_window")?,
            pickup_type: row
                .opt_code("pickup_type", PickupDropOffType::from_code, "code 0-3")?
                .unwrap_or_default(),
            drop_off_type: row
                .opt_code("drop_off_type", PickupDropOffType::from_code, "code 0-3")?
                .unwrap_or_default(),
            continuous_pickup: row
                .opt_code(
                    "continuous_pickup",
                    ContinuousPickupDropOff::from_code,
                    "code 0-3",
                )?
                .unwrap_or_default(),
            continuous_drop_off: row
                .opt_code(
                    "continuous_drop_off",
                    ContinuousPickupDropOff::from_code,
                    "code 0-3",
                )?
                .unwrap_or_default(),
            shape_dist_traveled: row.opt_num("shape_dist_traveled", "a distance")?,
            timepoint: row
                .opt_code("timepoint", Timepoint::from_code, "code 0-1")?
                .unwrap_or_default(),
            pickup_booking_rule_id: opt_string(row, "pickup_booking_rule_id"),
            drop_off_booking_rule_id: opt_string(row, "drop_off_booking_rule_id"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{read, read_path, test_support::feed_file};
    use crate::model::{PickupDropOffType, StopTime};
    use crate::parsers::ParseError;

    #[test]
    fn test_sample_stop_times() -> Result<(), ParseError> {
        let stop_times: Vec<StopTime> = read_path(feed_file("stop_times.txt"))?;
        assert_eq!(stop_times.len(), 28);
        assert_eq!(stop_times[0].trip_id, "STBA");
        // "6:00:00" - single-digit hour
        assert_eq!(stop_times[0].arrival_time, Some(6 * 3600));
        assert_eq!(stop_times[0].stop_id.as_deref(), Some("STAGECOACH"));
        assert_eq!(stop_times[0].stop_sequence, 1);
        Ok(())
    }

    #[test]
    fn test_flex_window_row() -> Result<(), ParseError> {
        let data = "\
trip_id,stop_sequence,location_group_id,start_pickup_drop_off_window,\
end_pickup_drop_off_window,pickup_type
flex1,1,zone_a,08:00:00,18:00:00,2
";
        let stop_times: Vec<StopTime> = read("stop_times.txt", data.as_bytes())?;
        let flex = &stop_times[0];
        assert!(flex.stop_id.is_none());
        assert_eq!(flex.location_group_id.as_deref(), Some("zone_a"));
        assert_eq!(flex.start_pickup_drop_off_window, Some(8 * 3600));
        assert_eq!(flex.end_pickup_drop_off_window, Some(18 * 3600));
        assert_eq!(flex.pickup_type, PickupDropOffType::PhoneAgency);
        Ok(())
    }
}
