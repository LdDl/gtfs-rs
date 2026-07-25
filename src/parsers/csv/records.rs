//! [`CsvRecord`] implementations for the `model` entities, one per
//! GTFS table, in specification order.

use super::{CsvRecord, Row};
use crate::model::{Agency, CemvSupport};
use crate::parsers::ParseError;

impl CsvRecord for Agency {
    const FILE_NAME: &'static str = "agency.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let mut agency = Agency::new(
            row.req("agency_name")?,
            row.req("agency_url")?,
            row.req("agency_timezone")?,
        );
        agency.agency_id = row.opt("agency_id").map(str::to_string);
        agency.agency_lang = row.opt("agency_lang").map(str::to_string);
        agency.agency_phone = row.opt("agency_phone").map(str::to_string);
        agency.agency_fare_url = row.opt("agency_fare_url").map(str::to_string);
        agency.agency_email = row.opt("agency_email").map(str::to_string);
        if let Some(cemv) = row.opt_code("cemv_support", CemvSupport::from_code, "code 0-2")? {
            agency.cemv_support = cemv;
        }
        Ok(agency)
    }
}
