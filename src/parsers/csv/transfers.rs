//! `transfers.txt` reader.

use crate::model::{Transfer, TransferType};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, Row, opt_string};

impl CsvRecord for Transfer {
    const FILE_NAME: &'static str = "transfers.txt";

    fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
        let transfer_type = row
            .opt_code("transfer_type", TransferType::from_code, "code 0-5")?
            .unwrap_or_default();
        let mut transfer = Transfer::new(transfer_type);
        transfer.from_stop_id = opt_string(row, "from_stop_id");
        transfer.to_stop_id = opt_string(row, "to_stop_id");
        transfer.from_route_id = opt_string(row, "from_route_id");
        transfer.to_route_id = opt_string(row, "to_route_id");
        transfer.from_trip_id = opt_string(row, "from_trip_id");
        transfer.to_trip_id = opt_string(row, "to_trip_id");
        transfer.min_transfer_time = row.opt_num("min_transfer_time", "seconds")?;
        Ok(transfer)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Transfer, TransferType};
    use crate::parsers::ParseError;
    use crate::parsers::csv::read;

    #[test]
    fn test_transfers() -> Result<(), ParseError> {
        let data = "\
from_stop_id,to_stop_id,transfer_type,min_transfer_time
A,B,2,180
C,D,,
";
        let transfers: Vec<Transfer> = read("transfers.txt", data.as_bytes())?;
        assert_eq!(transfers[0].transfer_type, TransferType::MinimumTime);
        assert_eq!(transfers[0].min_transfer_time, Some(180));
        // empty type falls back to the spec default
        assert_eq!(transfers[1].transfer_type, TransferType::Recommended);
        Ok(())
    }
}
