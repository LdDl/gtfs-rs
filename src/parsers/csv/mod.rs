//! # CSV Parser
//!
//! Header-driven readers for the `.txt` tables of a GTFS dataset.
//! Column order does not matter and unrecognized columns are
//! ignored, as the specification requires. Values are trimmed; an
//! empty value maps to `None` for optional fields.
//!
//! Read one table from a path with [`read_path`] (e.g. only
//! `agency.txt`), or from any [`std::io::Read`] source with
//! [`read`]. Entities implement [`CsvRecord`]; the same trait can be
//! implemented for custom types to read GTFS extensions.

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use super::{ParseError, ParseErrorKind};

mod records;

/// One CSV row with header context, handed to
/// [`CsvRecord::from_row`] implementations.
///
/// Lookups are by column name; positions in the file are irrelevant.
/// The accessors build [`ParseError`]s that already carry the file
/// name, line number and field name.
///
/// # Examples
///
/// ```
/// use gtfs_rs::parsers::ParseError;
/// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
///
/// struct StopName {
///     name: Option<String>,
/// }
///
/// impl CsvRecord for StopName {
///     const FILE_NAME: &'static str = "stops.txt";
///
///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
///         Ok(StopName {
///             name: row.opt("stop_name").map(str::to_string),
///         })
///     }
/// }
///
/// fn main() -> Result<(), ParseError> {
///     let data = "stop_id,stop_name\nA,Central\nB,\n";
///     let rows: Vec<StopName> = csv::read("stops.txt", data.as_bytes())?;
///     assert_eq!(rows[0].name.as_deref(), Some("Central"));
///     assert!(rows[1].name.is_none()); // empty value -> None
///     Ok(())
/// }
/// ```
pub struct Row<'a> {
    file: &'a str,
    line: u64,
    header: &'a HashMap<String, usize>,
    record: &'a csv::StringRecord,
}

impl Row<'_> {
    /// Returns the trimmed value of an optional column, or `None`
    /// when the column is absent or the value is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Desc {
    ///     desc: Option<String>,
    /// }
    ///
    /// impl CsvRecord for Desc {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Desc {
    ///             desc: row.opt("stop_desc").map(str::to_string),
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     // the stop_desc column does not exist at all
    ///     let data = "stop_id\nA\n";
    ///     let rows: Vec<Desc> = csv::read("stops.txt", data.as_bytes())?;
    ///     assert!(rows[0].desc.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt(&self, name: &str) -> Option<&str> {
        let index = *self.header.get(name)?;
        match self.record.get(index).map(str::trim) {
            None | Some("") => None,
            Some(value) => Some(value),
        }
    }

    /// Returns the trimmed value of a required column.
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::MissingColumn`] if the header lacks
    /// the column, or [`ParseErrorKind::EmptyValue`] if the value is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Id {
    ///     id: String,
    /// }
    ///
    /// impl CsvRecord for Id {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Id {
    ///             id: row.req("stop_id")?.to_string(),
    ///         })
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_name\nCentral\n";
    ///     match csv::read::<Id, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected a missing-column error"),
    ///         Err(e) => {
    ///             assert_eq!(e.field.as_deref(), Some("stop_id"));
    ///             assert!(matches!(e.kind, ParseErrorKind::MissingColumn));
    ///         }
    ///     }
    /// }
    /// ```
    pub fn req(&self, name: &str) -> Result<&str, ParseError> {
        if !self.header.contains_key(name) {
            return Err(self.err(name, ParseErrorKind::MissingColumn));
        }
        match self.opt(name) {
            Some(value) => Ok(value),
            None => Err(self.err(name, ParseErrorKind::EmptyValue)),
        }
    }

    /// Parses an optional integer-coded enum column with the given
    /// `from_code` converter (every crate enum provides one).
    ///
    /// # Arguments
    ///
    /// * `name` - Column name
    /// * `from_code` - Converter, e.g. `LocationType::from_code`
    /// * `expected` - Description for error messages, e.g. "code 0-4"
    ///
    /// # Errors
    ///
    /// Returns [`ParseErrorKind::Invalid`] if the value is not an
    /// integer or the code is rejected by the converter.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::LocationType;
    /// use gtfs_rs::parsers::ParseError;
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Kind {
    ///     location_type: Option<LocationType>,
    /// }
    ///
    /// impl CsvRecord for Kind {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Ok(Kind {
    ///             location_type: row.opt_code(
    ///                 "location_type",
    ///                 LocationType::from_code,
    ///                 "code 0-4",
    ///             )?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), ParseError> {
    ///     let data = "stop_id,location_type\nA,1\nB,\n";
    ///     let rows: Vec<Kind> = csv::read("stops.txt", data.as_bytes())?;
    ///     assert_eq!(rows[0].location_type, Some(LocationType::Station));
    ///     assert!(rows[1].location_type.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn opt_code<T>(
        &self,
        name: &str,
        from_code: fn(i32) -> Option<T>,
        expected: &str,
    ) -> Result<Option<T>, ParseError> {
        let raw = match self.opt(name) {
            None => return Ok(None),
            Some(raw) => raw,
        };
        let code: i32 = match raw.parse() {
            Ok(code) => code,
            Err(_) => return Err(self.invalid(name, raw, expected)),
        };
        match from_code(code) {
            Some(value) => Ok(Some(value)),
            None => Err(self.invalid(name, raw, expected)),
        }
    }

    /// Builds a field-level error at this row's position.
    ///
    /// # Arguments
    ///
    /// * `field` - Column name the error is about
    /// * `kind` - The failure kind
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Strict;
    ///
    /// impl CsvRecord for Strict {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         Err(row.err("stop_id", ParseErrorKind::EmptyValue))
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_id\nA\n";
    ///     match csv::read::<Strict, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected an error"),
    ///         Err(e) => {
    ///             assert_eq!(e.file, "stops.txt");
    ///             assert_eq!(e.line, 2);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn err(&self, field: &str, kind: ParseErrorKind) -> ParseError {
        ParseError {
            file: self.file.to_string(),
            line: self.line,
            field: Some(field.to_string()),
            kind,
        }
    }

    /// Builds an invalid-value error at this row's position.
    ///
    /// # Arguments
    ///
    /// * `field` - Column name the error is about
    /// * `value` - The rejected raw value
    /// * `expected` - What the parser expected
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::parsers::{ParseError, ParseErrorKind};
    /// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
    ///
    /// struct Lat;
    ///
    /// impl CsvRecord for Lat {
    ///     const FILE_NAME: &'static str = "stops.txt";
    ///
    ///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
    ///         let raw = row.req("stop_lat")?;
    ///         match raw.parse::<f64>() {
    ///             Ok(_) => Ok(Lat),
    ///             Err(_) => Err(row.invalid("stop_lat", raw, "latitude")),
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let data = "stop_lat\nnorth\n";
    ///     match csv::read::<Lat, _>("stops.txt", data.as_bytes()) {
    ///         Ok(_) => panic!("expected an error"),
    ///         Err(e) => assert!(e.to_string().contains("'north'")),
    ///     }
    /// }
    /// ```
    pub fn invalid(&self, field: &str, value: &str, expected: &str) -> ParseError {
        self.err(
            field,
            ParseErrorKind::Invalid {
                value: value.to_string(),
                expected: expected.to_string(),
            },
        )
    }
}

/// A GTFS entity readable from one CSV row.
///
/// Implemented by the crate for its `model` entities; implement it
/// for custom types to read GTFS extensions with the same machinery.
///
/// # Examples
///
/// ```
/// use gtfs_rs::parsers::ParseError;
/// use gtfs_rs::parsers::csv::{self, CsvRecord, Row};
///
/// // a custom extension table: vehicle_types.txt
/// struct VehicleType {
///     vehicle_type_id: String,
///     description: Option<String>,
/// }
///
/// impl CsvRecord for VehicleType {
///     const FILE_NAME: &'static str = "vehicle_types.txt";
///
///     fn from_row(row: &Row<'_>) -> Result<Self, ParseError> {
///         Ok(VehicleType {
///             vehicle_type_id: row.req("vehicle_type_id")?.to_string(),
///             description: row.opt("description").map(str::to_string),
///         })
///     }
/// }
///
/// fn main() -> Result<(), ParseError> {
///     let data = "vehicle_type_id,description\nbus12,Low-floor bus\n";
///     let types: Vec<VehicleType> =
///         csv::read("vehicle_types.txt", data.as_bytes())?;
///     assert_eq!(types[0].vehicle_type_id, "bus12");
///     Ok(())
/// }
/// ```
pub trait CsvRecord: Sized {
    /// Dataset file the entity comes from (e.g. "agency.txt").
    const FILE_NAME: &'static str;

    /// Builds the entity from one CSV row.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] describing the offending field when
    /// the row violates the entity's format.
    fn from_row(row: &Row<'_>) -> Result<Self, ParseError>;
}

/// Reads all records of one GTFS table from any reader.
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
/// The file name from the path is used in error messages.
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
///     let agencies: Vec<Agency> = csv::read_path("feed/agency.txt")?;
///     println!("{} agencies", agencies.len());
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
}
