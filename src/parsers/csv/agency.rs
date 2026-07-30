//! `agency.txt` reader.

use crate::model::{Agency, CemvSupport};
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for Agency {
    const FILE_NAME: &'static str = "agency.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut agency = Agency::new(
            row.req("agency_name")?,
            row.req("agency_url")?,
            row.req("agency_timezone")?,
        );
        agency.agency_id = opt_string(row, "agency_id");
        agency.agency_lang = opt_string(row, "agency_lang");
        agency.agency_phone = opt_string(row, "agency_phone");
        agency.agency_fare_url = opt_string(row, "agency_fare_url");
        agency.agency_email = opt_string(row, "agency_email");
        agency.cemv_support = row
            .opt_code("cemv_support", CemvSupport::from_code, "code 0-2")?
            .unwrap_or_default();
        Ok(agency)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Agency;
    use crate::parsers::ParseError;
    use crate::parsers::csv::{read_path, test_support::feed_file};

    #[test]
    fn test_sample_agency() -> Result<(), ParseError> {
        let agencies: Vec<Agency> = read_path(feed_file("agency.txt"))?;
        assert_eq!(agencies.len(), 1);
        assert_eq!(agencies[0].agency_id.as_deref(), Some("DTA"));
        assert_eq!(agencies[0].agency_name, "Demo Transit Authority");
        assert_eq!(agencies[0].agency_timezone, "America/Los_Angeles");
        Ok(())
    }
}
