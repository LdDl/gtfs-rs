//! `areas.txt` and `stop_areas.txt` readers.

use crate::model::{Area, StopArea};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row, opt_string};

impl CsvRecord for Area {
    const FILE_NAME: &'static str = "areas.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut area = Area::new(row.req("area_id")?);
        area.area_name = opt_string(row, "area_name");
        Ok(area)
    }
}

impl CsvRecord for StopArea {
    const FILE_NAME: &'static str = "stop_areas.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        Ok(StopArea::new(row.req("area_id")?, row.req("stop_id")?))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Area, StopArea};
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_areas_and_stop_areas() -> Result<(), ParseError> {
        let areas: Vec<Area> = read("areas.txt", "area_id,area_name\nzone_a,Zone A\n".as_bytes())?;
        assert_eq!(areas[0].area_name.as_deref(), Some("Zone A"));

        let stop_areas: Vec<StopArea> =
            read("stop_areas.txt", "area_id,stop_id\nzone_a,S1\n".as_bytes())?;
        assert_eq!(stop_areas[0].stop_id, "S1");
        Ok(())
    }
}
