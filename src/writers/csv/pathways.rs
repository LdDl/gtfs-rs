//! `pathways.txt` writer.

use crate::model::Pathway;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Pathway {
    const FILE_NAME: &'static str = "pathways.txt";

    const HEADER: &'static [&'static str] = &[
        "pathway_id",
        "from_stop_id",
        "to_stop_id",
        "pathway_mode",
        "is_bidirectional",
        "length",
        "traversal_time",
        "stair_count",
        "max_slope",
        "min_width",
        "signposted_as",
        "reversed_signposted_as",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.pathway_id.clone(),
            self.from_stop_id.clone(),
            self.to_stop_id.clone(),
            self.pathway_mode.code().to_string(),
            if self.is_bidirectional { "1" } else { "0" }.to_string(),
            self.length.map(|v| v.to_string()).unwrap_or_default(),
            self.traversal_time
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.stair_count.map(|v| v.to_string()).unwrap_or_default(),
            self.max_slope.map(|v| v.to_string()).unwrap_or_default(),
            self.min_width.map(|v| v.to_string()).unwrap_or_default(),
            self.signposted_as.clone().unwrap_or_default(),
            self.reversed_signposted_as.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Pathway, PathwayMode};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_pathway_fields() {
        let mut stairs = Pathway::new(
            "pw1",
            "entrance_1",
            "platform_2",
            PathwayMode::Stairs,
            false,
        )
        .with_traversal_time(45);
        stairs.stair_count = Some(-15);
        let fields = stairs.fields();
        assert_eq!(fields.len(), Pathway::HEADER.len());
        assert_eq!(fields[0], "pw1");
        assert_eq!(fields[3], "2");
        // unidirectional renders as "0"
        assert_eq!(fields[4], "0");
        assert_eq!(fields[6], "45");
        // negative stair count: going down in the from-to direction
        assert_eq!(fields[7], "-15");
        assert_eq!(fields[5], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_pathway_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut stairs = Pathway::new(
            "pw1",
            "entrance_1",
            "platform_2",
            PathwayMode::Stairs,
            false,
        )
        .with_length(12.5);
        stairs.stair_count = Some(-15);
        stairs.signposted_as = Some("To platform 2".to_string());
        let mut out = Vec::new();
        write("pathways.txt", &[stairs], &mut out)?;
        let parsed: Vec<Pathway> = read("pathways.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].pathway_id, "pw1");
        assert_eq!(parsed[0].from_stop_id, "entrance_1");
        assert_eq!(parsed[0].to_stop_id, "platform_2");
        assert_eq!(parsed[0].pathway_mode, PathwayMode::Stairs);
        assert!(!parsed[0].is_bidirectional);
        assert_eq!(parsed[0].length, Some(12.5));
        assert_eq!(parsed[0].stair_count, Some(-15));
        assert_eq!(parsed[0].signposted_as.as_deref(), Some("To platform 2"));
        assert_eq!(parsed[0].traversal_time, None);
        Ok(())
    }
}
