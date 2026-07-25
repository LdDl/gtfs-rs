//! Reading functions: one table from a reader or path, or a
//! whole unpacked feed directory.

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use super::{CsvRecord, Row};
use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route,
    RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};
use crate::parsers::{ParseError, ParseErrorKind};
use crate::reference::GtfsReference;

/// Reads all records of one GTFS table from any reader.
///
/// Which table is being read is decided by the entity type `T`
/// alone; `file_label` only labels error messages, and matching the
/// real table name is a convention, not a requirement.
///
/// # Arguments
///
/// * `file_label` - Name used in error messages (e.g. "agency.txt")
/// * `reader` - Source of the CSV bytes
///
/// # Errors
///
/// Returns a [`ParseError`] on malformed CSV or on the first row
/// rejected by [`CsvRecord::from_row`].
///
/// # Examples
///
/// ```
/// use gtfs_rs::Agency;
/// use gtfs_rs::parsers::{ParseError, csv};
///
/// fn main() -> Result<(), ParseError> {
///     let data = "\
/// agency_name,agency_url,agency_timezone
/// Demo,https://demo.example,Europe/Moscow
/// ";
///     let agencies: Vec<Agency> = csv::read("agency.txt", data.as_bytes())?;
///     assert_eq!(agencies[0].agency_name, "Demo");
///     Ok(())
/// }
/// ```
pub fn read<T: CsvRecord, R: io::Read>(file_label: &str, reader: R) -> Result<Vec<T>, ParseError> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let header_record = match csv_reader.headers() {
        Ok(headers) => headers.clone(),
        Err(e) => {
            return Err(ParseError {
                file: file_label.to_string(),
                line: 1,
                field: None,
                kind: ParseErrorKind::Csv(e),
            });
        }
    };
    let mut header = HashMap::new();
    for (index, name) in header_record.iter().enumerate() {
        header.insert(name.trim().to_string(), index);
    }

    let mut out = Vec::new();
    for result in csv_reader.records() {
        let record = match result {
            Ok(record) => record,
            Err(e) => {
                let line = e.position().map_or(0, |p| p.line());
                return Err(ParseError {
                    file: file_label.to_string(),
                    line,
                    field: None,
                    kind: ParseErrorKind::Csv(e),
                });
            }
        };
        let line = record.position().map_or(0, |p| p.line());
        let row = Row {
            file: file_label,
            line,
            header: &header,
            record: &record,
        };
        out.push(T::from_row(&row)?);
    }
    Ok(out)
}

/// Reads one GTFS table from a file path - e.g. only `agency.txt`,
/// without requiring the rest of the feed.
///
/// Which table is being read is decided by the entity type `T`
/// alone - usually inferred from the assignment, or spelled
/// explicitly as `read_path::<Agency>(path)`. The path is only the
/// source of bytes: it may have any file name (the name goes into
/// error messages), and nothing is guessed from it. Reading a file
/// with the wrong type fails with a
/// [`ParseErrorKind::MissingColumn`](crate::parsers::ParseErrorKind)
/// error on the first required column that is absent.
///
/// # Arguments
///
/// * `path` - Path to the table file
///
/// # Errors
///
/// Returns a [`ParseError`] if the file cannot be opened or its
/// content is rejected (see [`read`]).
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::Agency;
/// use gtfs_rs::parsers::{ParseError, csv};
///
/// fn main() -> Result<(), ParseError> {
///     // the type annotation picks the table...
///     let agencies: Vec<Agency> = csv::read_path("feed/agency.txt")?;
///     // ...or spell it explicitly; the file name may be anything
///     let backup = csv::read_path::<Agency>("backup/agencies_2026.csv")?;
///     println!("{} + {} agencies", agencies.len(), backup.len());
///     Ok(())
/// }
/// ```
pub fn read_path<T: CsvRecord>(path: impl AsRef<Path>) -> Result<Vec<T>, ParseError> {
    let path = path.as_ref();
    let label = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.display().to_string(),
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(ParseError {
                file: label,
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            });
        }
    };
    read(&label, file)
}

/// Reads a whole unpacked GTFS dataset directory into a
/// [`GtfsReference`].
///
/// The five required tables (`agency.txt`, `stops.txt`,
/// `routes.txt`, `trips.txt`, `stop_times.txt`) must exist; every
/// other table is read when its file is present and left empty
/// otherwise. `feed_info.txt` contributes at most one record.
/// `locations.geojson` is not read here - it is not CSV; a `geojson`
/// parser is planned.
///
/// # Arguments
///
/// * `dir` - Path to the unpacked feed directory
///
/// # Errors
///
/// Returns a [`ParseError`] if a required table is missing or any
/// present table is rejected (see [`read`]).
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::parsers::{ParseError, csv};
///
/// fn main() -> Result<(), ParseError> {
///     let gtfs = csv::read_dir("feed/")?;
///     println!("{} stops, {} trips", gtfs.stops.len(), gtfs.trips.len());
///     Ok(())
/// }
/// ```
pub fn read_dir(dir: impl AsRef<Path>) -> Result<GtfsReference, ParseError> {
    let dir = dir.as_ref();
    let mut gtfs = GtfsReference::new();
    // required tables
    gtfs.agencies = read_path(dir.join(Agency::FILE_NAME))?;
    gtfs.stops = read_path(dir.join(Stop::FILE_NAME))?;
    gtfs.routes = read_path(dir.join(Route::FILE_NAME))?;
    gtfs.trips = read_path(dir.join(Trip::FILE_NAME))?;
    gtfs.stop_times = read_path(dir.join(StopTime::FILE_NAME))?;
    // conditionally required and optional tables
    gtfs.calendar = read_if_present::<Calendar>(dir)?;
    gtfs.calendar_dates = read_if_present::<CalendarDate>(dir)?;
    gtfs.fare_attributes = read_if_present::<FareAttributeV1>(dir)?;
    gtfs.fare_rules = read_if_present::<FareRuleV1>(dir)?;
    gtfs.timeframes = read_if_present::<Timeframe>(dir)?;
    gtfs.rider_categories = read_if_present::<RiderCategory>(dir)?;
    gtfs.fare_media = read_if_present::<FareMedia>(dir)?;
    gtfs.fare_products = read_if_present::<FareProduct>(dir)?;
    gtfs.fare_leg_rules = read_if_present::<FareLegRule>(dir)?;
    gtfs.fare_leg_join_rules = read_if_present::<FareLegJoinRule>(dir)?;
    gtfs.fare_transfer_rules = read_if_present::<FareTransferRule>(dir)?;
    gtfs.areas = read_if_present::<Area>(dir)?;
    gtfs.stop_areas = read_if_present::<StopArea>(dir)?;
    gtfs.networks = read_if_present::<Network>(dir)?;
    gtfs.route_networks = read_if_present::<RouteNetwork>(dir)?;
    gtfs.shapes = read_if_present::<ShapePoint>(dir)?;
    gtfs.frequencies = read_if_present::<Frequency>(dir)?;
    gtfs.transfers = read_if_present::<Transfer>(dir)?;
    gtfs.pathways = read_if_present::<Pathway>(dir)?;
    gtfs.levels = read_if_present::<Level>(dir)?;
    gtfs.location_groups = read_if_present::<LocationGroup>(dir)?;
    gtfs.location_group_stops = read_if_present::<LocationGroupStop>(dir)?;
    gtfs.booking_rules = read_if_present::<BookingRule>(dir)?;
    gtfs.translations = read_if_present::<Translation>(dir)?;
    gtfs.feed_info = read_if_present::<FeedInfo>(dir)?.into_iter().next();
    gtfs.attributions = read_if_present::<Attribution>(dir)?;
    Ok(gtfs)
}

/// Reads an optional table: an absent file yields an empty list.
fn read_if_present<T: CsvRecord>(dir: &Path) -> Result<Vec<T>, ParseError> {
    let path = dir.join(T::FILE_NAME);
    if path.exists() {
        read_path(path)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Agency;

    #[test]
    fn test_quoted_fields_and_column_order() -> Result<(), ParseError> {
        // columns reordered, value with a comma inside quotes,
        // unknown column ignored
        let data = "\
agency_url,agency_name,unknown_column,agency_timezone
https://x.example,\"Transit, Inc.\",whatever,Europe/Moscow
";
        let agencies: Vec<Agency> = read("agency.txt", data.as_bytes())?;
        assert_eq!(agencies[0].agency_name, "Transit, Inc.");
        Ok(())
    }

    #[test]
    fn test_utf8_bom_is_stripped() -> Result<(), ParseError> {
        let data = "\u{feff}agency_name,agency_url,agency_timezone\n\
                    Demo,https://x.example,Europe/Moscow\n";
        let agencies: Vec<Agency> = read("agency.txt", data.as_bytes())?;
        assert_eq!(agencies[0].agency_name, "Demo");
        Ok(())
    }

    #[test]
    fn test_missing_required_column() {
        let data = "agency_url,agency_timezone\nhttps://x.example,UTC\n";
        let Err(err) = read::<Agency, _>("agency.txt", data.as_bytes()) else {
            panic!("expected a missing-column error");
        };
        assert_eq!(err.file, "agency.txt");
        assert_eq!(err.line, 2);
        assert_eq!(err.field.as_deref(), Some("agency_name"));
        assert!(matches!(err.kind, ParseErrorKind::MissingColumn));
    }

    #[test]
    fn test_invalid_enum_code_carries_context() {
        let data = "\
agency_name,agency_url,agency_timezone,cemv_support
Demo,https://x.example,UTC,7
";
        let Err(err) = read::<Agency, _>("agency.txt", data.as_bytes()) else {
            panic!("expected an invalid-code error");
        };
        assert_eq!(err.line, 2);
        assert_eq!(err.field.as_deref(), Some("cemv_support"));
        let ParseErrorKind::Invalid { value, .. } = &err.kind else {
            panic!("expected Invalid, got {:?}", err.kind);
        };
        assert_eq!(value, "7");
    }

    #[test]
    fn test_short_row_reads_as_empty() -> Result<(), ParseError> {
        // second row lacks trailing optional columns entirely
        let data = "\
agency_name,agency_url,agency_timezone,agency_lang
Demo,https://x.example,UTC
";
        let agencies: Vec<Agency> = read("agency.txt", data.as_bytes())?;
        assert!(agencies[0].agency_lang.is_none());
        Ok(())
    }

    #[test]
    fn test_read_dir_whole_feed() -> Result<(), ParseError> {
        use super::super::test_support::{FEED_DIR, model};
        use crate::misc::GtfsDate;

        let gtfs = read_dir(FEED_DIR)?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.stops.len(), 9);
        assert_eq!(gtfs.routes.len(), 5);
        assert_eq!(gtfs.trips.len(), 11);
        assert_eq!(gtfs.stop_times.len(), 28);
        assert_eq!(gtfs.calendar.len(), 2);
        assert_eq!(gtfs.calendar_dates.len(), 1);
        assert_eq!(gtfs.fare_attributes.len(), 2);
        assert_eq!(gtfs.fare_rules.len(), 4);
        assert_eq!(gtfs.frequencies.len(), 11);
        // absent optional tables stay empty
        assert!(gtfs.pathways.is_empty());
        assert!(gtfs.feed_info.is_none());
        // cross-table lookups work on the parsed dataset
        let pattern = gtfs.stop_times_of_trip("STBA");
        assert_eq!(pattern.len(), 2);
        assert!(gtfs.is_service_active("FULLW", &GtfsDate::new(2007, 1, 5).map_err(model)?));
        assert!(!gtfs.is_service_active("FULLW", &GtfsDate::new(2007, 6, 4).map_err(model)?));
        Ok(())
    }

    #[test]
    fn test_missing_file_reports_io_error() {
        use super::super::test_support::feed_file;

        let result: Result<Vec<Agency>, ParseError> = read_path(feed_file("no_such.txt"));
        let Err(err) = result else {
            panic!("expected an I/O error for a missing file");
        };
        assert_eq!(err.file, "no_such.txt");
        assert!(matches!(err.kind, ParseErrorKind::Io(_)));
    }
}
