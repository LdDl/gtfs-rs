//! `attributions.txt` reader.

use crate::model::Attribution;
use crate::parsers::ParseError;
use crate::parsers::csv::row::opt_string;
use crate::parsers::csv::{CsvRecord, Row};

impl CsvRecord for Attribution {
    const FILE_NAME: &'static str = "attributions.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut attribution = Attribution::new(row.req("organization_name")?);
        attribution.attribution_id = opt_string(row, "attribution_id");
        attribution.agency_id = opt_string(row, "agency_id");
        attribution.route_id = opt_string(row, "route_id");
        attribution.trip_id = opt_string(row, "trip_id");
        attribution.is_producer = row.opt_bool01("is_producer")?.unwrap_or(false);
        attribution.is_operator = row.opt_bool01("is_operator")?.unwrap_or(false);
        attribution.is_authority = row.opt_bool01("is_authority")?.unwrap_or(false);
        attribution.attribution_url = opt_string(row, "attribution_url");
        attribution.attribution_email = opt_string(row, "attribution_email");
        attribution.attribution_phone = opt_string(row, "attribution_phone");
        Ok(attribution)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Attribution;
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_attributions() -> Result<(), ParseError> {
        let data = "\
organization_name,is_producer,is_operator
Demo Org,1,0
";
        let attributions: Vec<Attribution> = read("attributions.txt", data.as_bytes())?;
        assert!(attributions[0].is_producer);
        assert!(!attributions[0].is_operator);
        assert!(!attributions[0].is_authority);
        Ok(())
    }
}
