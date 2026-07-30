//! `agency.txt` writer.

use crate::model::Agency;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Agency {
    const FILE_NAME: &'static str = "agency.txt";

    const HEADER: &'static [&'static str] = &[
        "agency_id",
        "agency_name",
        "agency_url",
        "agency_timezone",
        "agency_lang",
        "agency_phone",
        "agency_fare_url",
        "agency_email",
        "cemv_support",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.agency_id.clone().unwrap_or_default(),
            self.agency_name.clone(),
            self.agency_url.clone(),
            self.agency_timezone.clone(),
            self.agency_lang.clone().unwrap_or_default(),
            self.agency_phone.clone().unwrap_or_default(),
            self.agency_fare_url.clone().unwrap_or_default(),
            self.agency_email.clone().unwrap_or_default(),
            self.cemv_support.code().to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Agency;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() {
        let agency = Agency::new("Demo", "https://demo.example", "UTC").with_lang("en");
        let fields = agency.fields();
        assert_eq!(Agency::HEADER.len(), fields.len());
        assert_eq!(fields[0], "");
        assert_eq!(fields[1], "Demo");
        assert_eq!(fields[4], "en");
        assert_eq!(fields[8], "0");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::model::CemvSupport;

        let mut first = Agency::new("Demo Transit", "https://demo.example", "Europe/Moscow")
            .with_id("DT")
            .with_lang("ru")
            .with_phone("+7 495 000-00-00");
        first.cemv_support = CemvSupport::Supported;
        let agencies = vec![
            first,
            Agency::new("Other", "https://other.example", "Europe/Moscow"),
        ];
        let mut out = Vec::new();
        write("agency.txt", &agencies, &mut out)?;
        let parsed: Vec<Agency> = read("agency.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].agency_id.as_deref(), Some("DT"));
        assert_eq!(parsed[0].agency_name, "Demo Transit");
        assert_eq!(parsed[0].agency_timezone, "Europe/Moscow");
        assert_eq!(parsed[0].agency_lang.as_deref(), Some("ru"));
        assert_eq!(parsed[0].agency_phone.as_deref(), Some("+7 495 000-00-00"));
        assert_eq!(parsed[0].cemv_support, CemvSupport::Supported);
        assert_eq!(parsed[1].agency_id, None);
        assert_eq!(parsed[1].agency_name, "Other");
        assert_eq!(parsed[1].cemv_support, CemvSupport::NoInformation);
        Ok(())
    }
}
