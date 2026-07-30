//! `areas.txt` and `stop_areas.txt` writers.

use crate::model::{Area, StopArea};
use crate::writers::csv::CsvWrite;

impl CsvWrite for Area {
    const FILE_NAME: &'static str = "areas.txt";

    const HEADER: &'static [&'static str] = &["area_id", "area_name"];

    fn fields(&self) -> Vec<String> {
        vec![
            self.area_id.clone(),
            self.area_name.clone().unwrap_or_default(),
        ]
    }
}

impl CsvWrite for StopArea {
    const FILE_NAME: &'static str = "stop_areas.txt";

    const HEADER: &'static [&'static str] = &["area_id", "stop_id"];

    fn fields(&self) -> Vec<String> {
        vec![self.area_id.clone(), self.stop_id.clone()]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Area, StopArea};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_area_fields() {
        let zone = Area::new("zone_a").with_name("Zone A");
        let fields = zone.fields();
        assert_eq!(fields.len(), Area::HEADER.len());
        assert_eq!(fields[0], "zone_a");
        assert_eq!(fields[1], "Zone A");

        let unnamed = Area::new("zone_b");
        assert_eq!(unnamed.fields()[1], "");
    }

    #[test]
    fn test_stop_area_fields() {
        let assignment = StopArea::new("zone_a", "S1");
        let fields = assignment.fields();
        assert_eq!(fields.len(), StopArea::HEADER.len());
        assert_eq!(fields[0], "zone_a");
        assert_eq!(fields[1], "S1");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let areas = vec![Area::new("zone_a").with_name("Zone A")];
        let mut out = Vec::new();
        write(Area::FILE_NAME, &areas, &mut out)?;
        let parsed: Vec<Area> = read(Area::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].area_id, areas[0].area_id);
        assert_eq!(parsed[0].area_name, areas[0].area_name);

        let stop_areas = vec![StopArea::new("zone_a", "S1")];
        let mut out = Vec::new();
        write(StopArea::FILE_NAME, &stop_areas, &mut out)?;
        let parsed: Vec<StopArea> = read(StopArea::FILE_NAME, out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].area_id, stop_areas[0].area_id);
        assert_eq!(parsed[0].stop_id, stop_areas[0].stop_id);
        Ok(())
    }
}
