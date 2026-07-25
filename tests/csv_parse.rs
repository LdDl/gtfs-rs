//! Integration tests for the CSV parser against the official GTFS
//! sample feed in `tests/data/sample_feed/`.

#![cfg(feature = "parse")]

use gtfs_rs::Agency;
use gtfs_rs::parsers::{ParseError, ParseErrorKind, csv};

#[test]
fn test_reads_sample_feed_agency() -> Result<(), ParseError> {
    let agencies: Vec<Agency> = csv::read_path("tests/data/sample_feed/agency.txt")?;
    assert_eq!(agencies.len(), 1);
    assert_eq!(agencies[0].agency_id.as_deref(), Some("DTA"));
    assert_eq!(agencies[0].agency_name, "Demo Transit Authority");
    assert_eq!(agencies[0].agency_timezone, "America/Los_Angeles");
    Ok(())
}

#[test]
fn test_missing_file_reports_io_error() {
    let result = csv::read_path::<Agency>("tests/data/sample_feed/no_such.txt");
    let Err(err) = result else {
        panic!("expected an I/O error for a missing file");
    };
    assert_eq!(err.file, "no_such.txt");
    assert!(matches!(err.kind, ParseErrorKind::Io(_)));
}
