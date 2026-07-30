//! `location_groups.txt` and `location_group_stops.txt` readers
//! (GTFS-Flex). The `locations.geojson` zones are not CSV and await
//! the planned `geojson` parser.

use crate::model::{LocationGroup, LocationGroupStop};
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for LocationGroup {
    const FILE_NAME: &'static str = "location_groups.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut group = LocationGroup::new(row.req("location_group_id")?);
        group.location_group_name = opt_string(row, "location_group_name");
        Ok(group)
    }
}

impl CsvRecord for LocationGroupStop {
    const FILE_NAME: &'static str = "location_group_stops.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        Ok(LocationGroupStop::new(
            row.req("location_group_id")?,
            row.req("stop_id")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{LocationGroup, LocationGroupStop};
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_location_groups_and_stops() -> Result<(), ParseError> {
        let groups: Vec<LocationGroup> = read(
            "location_groups.txt",
            "location_group_id,location_group_name\nlg1,Downtown\n".as_bytes(),
        )?;
        assert_eq!(groups[0].location_group_name.as_deref(), Some("Downtown"));

        let group_stops: Vec<LocationGroupStop> = read(
            "location_group_stops.txt",
            "location_group_id,stop_id\nlg1,S1\n".as_bytes(),
        )?;
        assert_eq!(group_stops[0].stop_id, "S1");
        Ok(())
    }
}
