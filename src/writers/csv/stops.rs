//! `stops.txt` writer.

use crate::model::Stop;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Stop {
    const FILE_NAME: &'static str = "stops.txt";

    const HEADER: &'static [&'static str] = &[
        "stop_id",
        "stop_code",
        "stop_name",
        "tts_stop_name",
        "stop_desc",
        "stop_lat",
        "stop_lon",
        "zone_id",
        "stop_url",
        "location_type",
        "parent_station",
        "stop_access",
        "stop_timezone",
        "wheelchair_boarding",
        "level_id",
        "platform_code",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.stop_id.clone(),
            self.stop_code.clone().unwrap_or_default(),
            self.stop_name.clone().unwrap_or_default(),
            self.tts_stop_name.clone().unwrap_or_default(),
            self.stop_desc.clone().unwrap_or_default(),
            self.stop_lat.map(|v| v.to_string()).unwrap_or_default(),
            self.stop_lon.map(|v| v.to_string()).unwrap_or_default(),
            self.zone_id.clone().unwrap_or_default(),
            self.stop_url.clone().unwrap_or_default(),
            self.location_type.code().to_string(),
            self.parent_station.clone().unwrap_or_default(),
            self.stop_access
                .map(|v| v.code().to_string())
                .unwrap_or_default(),
            self.stop_timezone.clone().unwrap_or_default(),
            self.wheelchair_boarding.code().to_string(),
            self.level_id.clone().unwrap_or_default(),
            self.platform_code.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{LocationType, Stop};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let stop = Stop::new("S1")
            .with_name("Central")
            .with_coordinates(55.751, 37.617)
            .with_location_type(LocationType::Station);
        let fields = stop.fields();
        assert_eq!(Stop::HEADER.len(), fields.len());
        assert_eq!(fields[0], "S1");
        assert_eq!(fields[5], "55.751");
        assert_eq!(fields[9], "1");
        assert_eq!(fields[11], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::{StopAccess, WheelchairBoarding};

        let station = Stop::new("S1")
            .with_name("Central")
            .with_coordinates(55.751, 37.617)
            .with_location_type(LocationType::Station);
        let mut platform = Stop::new("S1_2")
            .with_name("Central")
            .with_coordinates(55.7511, 37.6172)
            .with_parent_station("S1")
            .with_zone_id("Z1")
            .with_platform_code("2");
        platform.stop_access = Some(StopAccess::ViaStation);
        platform.wheelchair_boarding = WheelchairBoarding::Accessible;
        let stops = vec![station, platform];
        let mut out = Vec::new();
        write("stops.txt", &stops, &mut out)?;
        let parsed: Vec<Stop> = read("stops.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].stop_id, "S1");
        assert_eq!(parsed[0].stop_name.as_deref(), Some("Central"));
        assert_eq!(parsed[0].stop_lat, Some(55.751));
        assert_eq!(parsed[0].stop_lon, Some(37.617));
        assert_eq!(parsed[0].location_type, LocationType::Station);
        assert_eq!(parsed[1].stop_id, "S1_2");
        assert_eq!(parsed[1].parent_station.as_deref(), Some("S1"));
        assert_eq!(parsed[1].stop_access, Some(StopAccess::ViaStation));
        assert_eq!(
            parsed[1].wheelchair_boarding,
            WheelchairBoarding::Accessible
        );
        assert_eq!(parsed[1].platform_code.as_deref(), Some("2"));
        Ok(())
    }
}
