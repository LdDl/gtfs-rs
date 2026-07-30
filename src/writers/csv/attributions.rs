//! `attributions.txt` writer.

use crate::model::Attribution;
use crate::writers::csv::CsvWrite;

/// Encodes a GTFS role flag: `true` -> "1", `false` -> "0".
fn bool01(value: bool) -> String {
    if value { "1" } else { "0" }.to_string()
}

impl CsvWrite for Attribution {
    const FILE_NAME: &'static str = "attributions.txt";

    const HEADER: &'static [&'static str] = &[
        "attribution_id",
        "agency_id",
        "route_id",
        "trip_id",
        "organization_name",
        "is_producer",
        "is_operator",
        "is_authority",
        "attribution_url",
        "attribution_email",
        "attribution_phone",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.attribution_id.clone().unwrap_or_default(),
            self.agency_id.clone().unwrap_or_default(),
            self.route_id.clone().unwrap_or_default(),
            self.trip_id.clone().unwrap_or_default(),
            self.organization_name.clone(),
            bool01(self.is_producer),
            bool01(self.is_operator),
            bool01(self.is_authority),
            self.attribution_url.clone().unwrap_or_default(),
            self.attribution_email.clone().unwrap_or_default(),
            self.attribution_phone.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Attribution;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let attribution = Attribution::new("Demo Org").as_producer().as_authority();
        let fields = attribution.fields();
        assert_eq!(Attribution::HEADER.len(), fields.len());
        assert_eq!(fields[0], "");
        assert_eq!(fields[4], "Demo Org");
        assert_eq!(fields[5], "1");
        assert_eq!(fields[6], "0");
        assert_eq!(fields[7], "1");
        assert_eq!(fields[10], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut first = Attribution::new("City Transit").as_producer().as_operator();
        first.attribution_id = Some("A1".to_string());
        first.route_id = Some("R1".to_string());
        first.attribution_url = Some("https://transit.example".to_string());
        let attributions = vec![first, Attribution::new("Data Co").as_authority()];
        let mut out = Vec::new();
        write("attributions.txt", &attributions, &mut out)?;
        let parsed: Vec<Attribution> = read("attributions.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].attribution_id.as_deref(), Some("A1"));
        assert_eq!(parsed[0].route_id.as_deref(), Some("R1"));
        assert_eq!(parsed[0].organization_name, "City Transit");
        assert!(parsed[0].is_producer);
        assert!(parsed[0].is_operator);
        assert!(!parsed[0].is_authority);
        assert_eq!(
            parsed[0].attribution_url.as_deref(),
            Some("https://transit.example")
        );
        assert_eq!(parsed[1].organization_name, "Data Co");
        assert!(!parsed[1].is_producer);
        assert!(parsed[1].is_authority);
        assert_eq!(parsed[1].attribution_phone, None);
        Ok(())
    }
}
