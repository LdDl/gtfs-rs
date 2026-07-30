//! `feed_info.txt` reader.

use crate::model::FeedInfo;
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for FeedInfo {
    const FILE_NAME: &'static str = "feed_info.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut info = FeedInfo::new(
            row.req("feed_publisher_name")?,
            row.req("feed_publisher_url")?,
            row.req("feed_lang")?,
        );
        info.default_lang = opt_string(row, "default_lang");
        info.feed_start_date = row.opt_date("feed_start_date")?;
        info.feed_end_date = row.opt_date("feed_end_date")?;
        info.feed_version = opt_string(row, "feed_version");
        info.feed_contact_email = opt_string(row, "feed_contact_email");
        info.feed_contact_url = opt_string(row, "feed_contact_url");
        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use crate::misc::GtfsDate;
    use crate::model::FeedInfo;
    use crate::parsers::ParseError;
    use crate::parsers::csv::{read, test_support::model};

    #[test]
    fn test_feed_info() -> Result<(), ParseError> {
        let data = "\
feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_version
Demo,https://demo.example,ru,20260101,2026-07
";
        let infos: Vec<FeedInfo> = read("feed_info.txt", data.as_bytes())?;
        assert_eq!(infos[0].feed_lang, "ru");
        assert_eq!(
            infos[0].feed_start_date,
            Some(GtfsDate::new(2026, 1, 1).map_err(model)?)
        );
        assert_eq!(infos[0].feed_version.as_deref(), Some("2026-07"));
        Ok(())
    }
}
