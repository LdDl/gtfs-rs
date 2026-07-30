//! `feed_info.txt` writer.

use crate::model::FeedInfo;
use crate::writers::csv::CsvWrite;

impl CsvWrite for FeedInfo {
    const FILE_NAME: &'static str = "feed_info.txt";

    const HEADER: &'static [&'static str] = &[
        "feed_publisher_name",
        "feed_publisher_url",
        "feed_lang",
        "default_lang",
        "feed_start_date",
        "feed_end_date",
        "feed_version",
        "feed_contact_email",
        "feed_contact_url",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.feed_publisher_name.clone(),
            self.feed_publisher_url.clone(),
            self.feed_lang.clone(),
            self.default_lang.clone().unwrap_or_default(),
            self.feed_start_date
                .map(|d| d.to_string())
                .unwrap_or_default(),
            self.feed_end_date
                .map(|d| d.to_string())
                .unwrap_or_default(),
            self.feed_version.clone().unwrap_or_default(),
            self.feed_contact_email.clone().unwrap_or_default(),
            self.feed_contact_url.clone().unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::FeedInfo;
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_fields_match_header() -> Result<(), Box<dyn std::error::Error>> {
        let info = FeedInfo::new("Pub", "https://x.example", "ru")
            .with_period(GtfsDate::new(2026, 1, 1)?, GtfsDate::new(2026, 12, 31)?)
            .with_version("2026-07");
        let fields = info.fields();
        assert_eq!(FeedInfo::HEADER.len(), fields.len());
        assert_eq!(fields[0], "Pub");
        assert_eq!(fields[2], "ru");
        assert_eq!(fields[3], "");
        assert_eq!(fields[4], "20260101");
        assert_eq!(fields[5], "20261231");
        assert_eq!(fields[6], "2026-07");
        assert_eq!(fields[8], "");
        Ok(())
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let mut info = FeedInfo::new("City Transit", "https://transit.example", "ru")
            .with_period(GtfsDate::new(2026, 1, 1)?, GtfsDate::new(2026, 12, 31)?)
            .with_version("2026-07");
        info.feed_contact_email = Some("gtfs@transit.example".to_string());
        let infos = vec![info];
        let mut out = Vec::new();
        write("feed_info.txt", &infos, &mut out)?;
        let parsed: Vec<FeedInfo> = read("feed_info.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].feed_publisher_name, "City Transit");
        assert_eq!(parsed[0].feed_publisher_url, "https://transit.example");
        assert_eq!(parsed[0].feed_lang, "ru");
        assert_eq!(parsed[0].default_lang, None);
        assert_eq!(parsed[0].feed_start_date, Some(GtfsDate::new(2026, 1, 1)?));
        assert_eq!(parsed[0].feed_end_date, Some(GtfsDate::new(2026, 12, 31)?));
        assert_eq!(parsed[0].feed_version.as_deref(), Some("2026-07"));
        assert_eq!(
            parsed[0].feed_contact_email.as_deref(),
            Some("gtfs@transit.example")
        );
        assert_eq!(parsed[0].feed_contact_url, None);
        Ok(())
    }
}
