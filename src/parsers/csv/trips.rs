//! `trips.txt` reader.

use crate::model::{BikesAllowed, CarsAllowed, Direction, Trip, WheelchairAccessible};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row, opt_string};

impl CsvRecord for Trip {
    const FILE_NAME: &'static str = "trips.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut trip = Trip::new(
            row.req("trip_id")?,
            row.req("route_id")?,
            row.req("service_id")?,
        );
        trip.trip_headsign = opt_string(row, "trip_headsign");
        trip.trip_short_name = opt_string(row, "trip_short_name");
        trip.direction_id = row.opt_code("direction_id", Direction::from_code, "code 0-1")?;
        trip.block_id = opt_string(row, "block_id");
        trip.shape_id = opt_string(row, "shape_id");
        trip.wheelchair_accessible = row
            .opt_code(
                "wheelchair_accessible",
                WheelchairAccessible::from_code,
                "code 0-2",
            )?
            .unwrap_or_default();
        trip.bikes_allowed = row
            .opt_code("bikes_allowed", BikesAllowed::from_code, "code 0-2")?
            .unwrap_or_default();
        trip.cars_allowed = row
            .opt_code("cars_allowed", CarsAllowed::from_code, "code 0-2")?
            .unwrap_or_default();
        trip.safe_duration_factor = row.opt_num("safe_duration_factor", "a float")?;
        trip.safe_duration_offset = row.opt_num("safe_duration_offset", "a float")?;
        Ok(trip)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Direction, Trip};
    use crate::parsers::ParseError;
    use crate::parsers::csv::{read_path, test_support::feed_file};

    #[test]
    fn test_sample_trips() -> Result<(), ParseError> {
        let trips: Vec<Trip> = read_path(feed_file("trips.txt"))?;
        assert_eq!(trips.len(), 11);
        assert_eq!(trips[0].trip_id, "AB1");
        assert_eq!(trips[0].route_id, "AB");
        assert_eq!(trips[0].service_id, "FULLW");
        assert_eq!(trips[0].direction_id, Some(Direction::Outbound));
        assert_eq!(trips[1].direction_id, Some(Direction::Inbound));
        Ok(())
    }
}
