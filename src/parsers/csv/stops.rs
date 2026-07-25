//! `stops.txt` reader.

use crate::model::{LocationType, Stop, StopAccess, WheelchairBoarding};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row, opt_string};

impl CsvRecord for Stop {
    const FILE_NAME: &'static str = "stops.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut stop = Stop::new(row.req("stop_id")?);
        stop.stop_code = opt_string(row, "stop_code");
        stop.stop_name = opt_string(row, "stop_name");
        stop.tts_stop_name = opt_string(row, "tts_stop_name");
        stop.stop_desc = opt_string(row, "stop_desc");
        stop.stop_lat = row.opt_num("stop_lat", "a latitude")?;
        stop.stop_lon = row.opt_num("stop_lon", "a longitude")?;
        stop.zone_id = opt_string(row, "zone_id");
        stop.stop_url = opt_string(row, "stop_url");
        stop.location_type = row
            .opt_code("location_type", LocationType::from_code, "code 0-4")?
            .unwrap_or_default();
        stop.parent_station = opt_string(row, "parent_station");
        stop.stop_access = row.opt_code("stop_access", StopAccess::from_code, "code 0-1")?;
        stop.stop_timezone = opt_string(row, "stop_timezone");
        stop.wheelchair_boarding = row
            .opt_code(
                "wheelchair_boarding",
                WheelchairBoarding::from_code,
                "code 0-2",
            )?
            .unwrap_or_default();
        stop.level_id = opt_string(row, "level_id");
        stop.platform_code = opt_string(row, "platform_code");
        Ok(stop)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{LocationType, Stop};
    use crate::parsers::ParseError;
    use crate::parsers::csv::{read, read_path, test_support::feed_file};

    #[test]
    fn test_sample_stops() -> Result<(), ParseError> {
        let stops: Vec<Stop> = read_path(feed_file("stops.txt"))?;
        assert_eq!(stops.len(), 9);
        assert_eq!(stops[0].stop_id, "FUR_CREEK_RES");
        assert_eq!(
            stops[0].stop_name.as_deref(),
            Some("Furnace Creek Resort (Demo)")
        );
        assert_eq!(stops[0].stop_lat, Some(36.425288));
        assert_eq!(stops[0].stop_lon, Some(-117.133162));
        Ok(())
    }

    #[test]
    fn test_station_hierarchy() -> Result<(), ParseError> {
        let data = "\
stop_id,stop_name,location_type,parent_station
S1,Central,1,
S1_p2,Central,0,S1
";
        let stops: Vec<Stop> = read("stops.txt", data.as_bytes())?;
        assert_eq!(stops[0].location_type, LocationType::Station);
        assert_eq!(stops[1].parent_station.as_deref(), Some("S1"));
        Ok(())
    }
}
