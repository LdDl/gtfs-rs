//! `trips.txt` writer.

use crate::model::Trip;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Trip {
    const FILE_NAME: &'static str = "trips.txt";

    const HEADER: &'static [&'static str] = &[
        "route_id",
        "service_id",
        "trip_id",
        "trip_headsign",
        "trip_short_name",
        "direction_id",
        "block_id",
        "shape_id",
        "wheelchair_accessible",
        "bikes_allowed",
        "cars_allowed",
        "safe_duration_factor",
        "safe_duration_offset",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.route_id.clone(),
            self.service_id.clone(),
            self.trip_id.clone(),
            self.trip_headsign.clone().unwrap_or_default(),
            self.trip_short_name.clone().unwrap_or_default(),
            self.direction_id
                .map(|d| d.code().to_string())
                .unwrap_or_default(),
            self.block_id.clone().unwrap_or_default(),
            self.shape_id.clone().unwrap_or_default(),
            self.wheelchair_accessible.code().to_string(),
            self.bikes_allowed.code().to_string(),
            self.cars_allowed.code().to_string(),
            self.safe_duration_factor
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.safe_duration_offset
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Direction, Trip};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let trip = Trip::new("t0", "L1", "weekday").with_direction(Direction::Inbound);
        let fields = trip.fields();
        assert_eq!(Trip::HEADER.len(), fields.len());
        assert_eq!(fields[0], "L1");
        assert_eq!(fields[2], "t0");
        assert_eq!(fields[5], "1");
        assert_eq!(fields[6], "");
        assert_eq!(fields[8], "0");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::{BikesAllowed, WheelchairAccessible};

        let mut first = Trip::new("t0", "L1", "weekday")
            .with_direction(Direction::Outbound)
            .with_headsign("Airport")
            .with_short_name("501")
            .with_block_id("b1")
            .with_shape_id("shp1");
        first.wheelchair_accessible = WheelchairAccessible::Accessible;
        first.bikes_allowed = BikesAllowed::NotAllowed;
        first.safe_duration_factor = Some(1.5);
        let trips = vec![first, Trip::new("t1", "L1", "weekend")];
        let mut out = Vec::new();
        write("trips.txt", &trips, &mut out)?;
        let parsed: Vec<Trip> = read("trips.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].trip_id, "t0");
        assert_eq!(parsed[0].route_id, "L1");
        assert_eq!(parsed[0].service_id, "weekday");
        assert_eq!(parsed[0].trip_headsign.as_deref(), Some("Airport"));
        assert_eq!(parsed[0].direction_id, Some(Direction::Outbound));
        assert_eq!(parsed[0].block_id.as_deref(), Some("b1"));
        assert_eq!(parsed[0].shape_id.as_deref(), Some("shp1"));
        assert_eq!(
            parsed[0].wheelchair_accessible,
            WheelchairAccessible::Accessible
        );
        assert_eq!(parsed[0].bikes_allowed, BikesAllowed::NotAllowed);
        assert_eq!(parsed[0].safe_duration_factor, Some(1.5));
        assert_eq!(parsed[1].trip_id, "t1");
        assert_eq!(parsed[1].direction_id, None);
        Ok(())
    }
}
