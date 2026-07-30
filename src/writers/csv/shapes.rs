//! `shapes.txt` writer.

use crate::model::ShapePoint;
use crate::writers::csv::CsvWrite;

impl CsvWrite for ShapePoint {
    const FILE_NAME: &'static str = "shapes.txt";

    const HEADER: &'static [&'static str] = &[
        "shape_id",
        "shape_pt_lat",
        "shape_pt_lon",
        "shape_pt_sequence",
        "shape_dist_traveled",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.shape_id.clone(),
            self.shape_pt_lat.to_string(),
            self.shape_pt_lon.to_string(),
            self.shape_pt_sequence.to_string(),
            self.shape_dist_traveled
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::ShapePoint;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let pt = ShapePoint::new("sh1", 55.751, 37.617, 1).with_dist_traveled(0.5);
        let fields = pt.fields();
        assert_eq!(fields.len(), ShapePoint::HEADER.len());
        assert_eq!(fields[1], "55.751");
        assert_eq!(fields[3], "1");
        assert_eq!(fields[4], "0.5");
        // absent optional distance becomes an empty field
        let bare = ShapePoint::new("sh1", 55.752, 37.62, 2);
        assert_eq!(bare.fields()[4], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let rows = vec![
            ShapePoint::new("sh1", 55.751, 37.617, 1).with_dist_traveled(0.0),
            ShapePoint::new("sh1", 55.752, 37.62, 2),
        ];
        let mut out = Vec::new();
        write("shapes.txt", &rows, &mut out)?;
        let parsed: Vec<ShapePoint> = read("shapes.txt", out.as_slice())?;

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].shape_id, "sh1");
        assert_eq!(parsed[0].shape_pt_lat, 55.751);
        assert_eq!(parsed[0].shape_pt_lon, 37.617);
        assert_eq!(parsed[0].shape_pt_sequence, 1);
        assert_eq!(parsed[0].shape_dist_traveled, Some(0.0));
        assert!(parsed[1].shape_dist_traveled.is_none());
        Ok(())
    }
}
