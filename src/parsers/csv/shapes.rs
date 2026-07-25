//! `shapes.txt` reader.

use super::{CsvRecord, Row};
use crate::model::ShapePoint;
use crate::parsers::ParseError;

impl CsvRecord for ShapePoint {
    const FILE_NAME: &'static str = "shapes.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut point = ShapePoint::new(
            row.req("shape_id")?,
            row.req_num("shape_pt_lat", "a latitude")?,
            row.req_num("shape_pt_lon", "a longitude")?,
            row.req_num("shape_pt_sequence", "a non-negative integer")?,
        );
        point.shape_dist_traveled = row.opt_num("shape_dist_traveled", "a distance")?;
        Ok(point)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{read, read_path, test_support::feed_file};
    use crate::model::ShapePoint;
    use crate::parsers::ParseError;

    #[test]
    fn test_sample_shapes_header_only() -> Result<(), ParseError> {
        let shapes: Vec<ShapePoint> = read_path(feed_file("shapes.txt"))?;
        assert!(shapes.is_empty());
        Ok(())
    }

    #[test]
    fn test_shape_points() -> Result<(), ParseError> {
        let data = "\
shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence,shape_dist_traveled
sh1,55.751,37.617,1,0.0
sh1,55.752,37.620,2,
";
        let shapes: Vec<ShapePoint> = read("shapes.txt", data.as_bytes())?;
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].shape_dist_traveled, Some(0.0));
        assert!(shapes[1].shape_dist_traveled.is_none());
        Ok(())
    }
}
