//! Reading functions: one CSV table from a reader or from a file
//! path.

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::parsers::csv::{CsvRecord, Row};
use crate::parsers::{ParseError, ParseErrorKind};

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
    fn test_missing_file_reports_io_error() {
        use crate::parsers::csv::test_support::feed_file;

        let result: Result<Vec<Agency>, ParseError> = read_path(feed_file("no_such.txt"));
        let Err(err) = result else {
            panic!("expected an I/O error for a missing file");
        };
        assert_eq!(err.file, "no_such.txt");
        assert!(matches!(err.kind, ParseErrorKind::Io(_)));
    }
}
