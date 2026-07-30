//! `transfers.txt` writer.

use crate::model::Transfer;
use crate::writers::csv::CsvWrite;

impl CsvWrite for Transfer {
    const FILE_NAME: &'static str = "transfers.txt";

    const HEADER: &'static [&'static str] = &[
        "from_stop_id",
        "to_stop_id",
        "from_route_id",
        "to_route_id",
        "from_trip_id",
        "to_trip_id",
        "transfer_type",
        "min_transfer_time",
    ];

    fn fields(&self) -> Vec<String> {
        vec![
            self.from_stop_id.clone().unwrap_or_default(),
            self.to_stop_id.clone().unwrap_or_default(),
            self.from_route_id.clone().unwrap_or_default(),
            self.to_route_id.clone().unwrap_or_default(),
            self.from_trip_id.clone().unwrap_or_default(),
            self.to_trip_id.clone().unwrap_or_default(),
            self.transfer_type.code().to_string(),
            self.min_transfer_time
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Transfer, TransferType};
    #[cfg(feature = "parse")]
    use crate::parsers::csv::read;
    use crate::writers::csv::CsvWrite;
    #[cfg(feature = "parse")]
    use crate::writers::csv::write;

    #[test]
    fn test_transfer_fields() {
        let rule = Transfer::new(TransferType::MinimumTime)
            .between_stops("A", "B")
            .with_min_transfer_time(180);
        let fields = rule.fields();
        assert_eq!(fields.len(), Transfer::HEADER.len());
        assert_eq!(fields[0], "A");
        assert_eq!(fields[1], "B");
        assert_eq!(fields[6], "2");
        assert_eq!(fields[7], "180");
        // absent optional endpoints render as empty fields
        assert_eq!(fields[2], "");
        assert_eq!(fields[5], "");
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_transfer_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let rules = vec![
            Transfer::new(TransferType::MinimumTime)
                .between_stops("A", "B")
                .with_min_transfer_time(180),
            Transfer::new(TransferType::InSeat).between_trips("t1", "t2"),
        ];
        let mut out = Vec::new();
        write("transfers.txt", &rules, &mut out)?;
        let parsed: Vec<Transfer> = read("transfers.txt", out.as_slice())?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].from_stop_id.as_deref(), Some("A"));
        assert_eq!(parsed[0].to_stop_id.as_deref(), Some("B"));
        assert_eq!(parsed[0].transfer_type, TransferType::MinimumTime);
        assert_eq!(parsed[0].min_transfer_time, Some(180));
        assert_eq!(parsed[1].transfer_type, TransferType::InSeat);
        assert_eq!(parsed[1].from_trip_id.as_deref(), Some("t1"));
        assert_eq!(parsed[1].to_trip_id.as_deref(), Some("t2"));
        assert_eq!(parsed[1].min_transfer_time, None);
        Ok(())
    }
}
