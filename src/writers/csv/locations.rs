//! `location_groups.txt` and `location_group_stops.txt` writers
//! (GTFS-Flex). The `locations.geojson` zones are not CSV and are
//! written by the `geojson` writer.

use crate::model::{LocationGroup, LocationGroupStop};
use crate::writers::csv::CsvWrite;

impl CsvWrite for LocationGroup {
    const FILE_NAME: &'static str = "location_groups.txt";

    const HEADER: &'static [&'static str] = &["location_group_id", "location_group_name"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.location_group_id.clone(),
            self.location_group_name.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for LocationGroupStop {
    const FILE_NAME: &'static str = "location_group_stops.txt";

    const HEADER: &'static [&'static str] = &["location_group_id", "stop_id"];

    fn fields(&self) -> Vec<String> {
        vec![self.location_group_id.clone(), self.stop_id.clone()]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{LocationGroup, LocationGroupStop};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_location_group_fields() {
        let group = LocationGroup::new("lg1").with_name("Downtown");
        let fields = group.fields();
        assert_eq!(fields.len(), LocationGroup::HEADER.len());
        assert_eq!(fields[0], "lg1");
        assert_eq!(fields[1], "Downtown");
        // absent optional name renders as an empty field
        assert_eq!(LocationGroup::new("lg2").fields()[1], "");
    }

    #[test]
    fn test_location_group_stop_fields() {
        let assignment = LocationGroupStop::new("lg1", "S1");
        let fields = assignment.fields();
        assert_eq!(fields.len(), LocationGroupStop::HEADER.len());
        assert_eq!(fields[0], "lg1");
        assert_eq!(fields[1], "S1");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_locations_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let groups = vec![LocationGroup::new("lg1").with_name("Downtown")];
        let mut out = Vec::new();
        write("location_groups.txt", &groups, &mut out)?;
        let parsed: Vec<LocationGroup> = read("location_groups.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].location_group_id, "lg1");
        assert_eq!(parsed[0].location_group_name.as_deref(), Some("Downtown"));

        let group_stops = vec![LocationGroupStop::new("lg1", "S1")];
        let mut out = Vec::new();
        write("location_group_stops.txt", &group_stops, &mut out)?;
        let parsed: Vec<LocationGroupStop> = read("location_group_stops.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].location_group_id, "lg1");
        assert_eq!(parsed[0].stop_id, "S1");
        Ok(())
    }
}
