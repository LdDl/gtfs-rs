//! # Parse Errors
//!
//! Every parsing failure carries full context: which file, which
//! line, which field, and what exactly went wrong - so an error in a
//! million-row `stop_times.txt` points at the offending record.

use std::fmt;
use std::io;

use crate::error::GtfsError;

/// Where and why parsing failed.
#[derive(Debug)]
#[non_exhaustive]
pub struct ParseError {
    /// File the error occurred in (e.g. "agency.txt")
    pub file: String,
    /// One-based line number within the file; 0 when the error is
    /// not tied to a line (e.g. an I/O failure)
    pub line: u64,
    /// Column name, when the error is field-level
    pub field: Option<String>,
    /// What exactly went wrong
    pub kind: ParseErrorKind,
}

/// The failure kinds of [`ParseError`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// Reading the underlying file failed
    Io(io::Error),
    /// The CSV structure is malformed
    Csv(csv::Error),
    /// The JSON structure is malformed (`locations.geojson`)
    #[cfg(feature = "geojson")]
    Json(serde_json::Error),
    /// The feed archive is malformed
    #[cfg(feature = "zip")]
    Zip(zip::result::ZipError),
    /// A required column is missing from the header
    MissingColumn,
    /// A column name appears more than once in the header
    DuplicateColumn,
    /// A required field has an empty value
    EmptyValue,
    /// A value does not match the expected format
    Invalid {
        /// The rejected raw value
        value: String,
        /// What the parser expected (e.g. "code 0-2")
        expected: String,
    },
    /// A value was rejected by one of the model value types
    /// ([`GtfsDate`](crate::GtfsDate),
    /// [`CurrencyAmount`](crate::CurrencyAmount), times)
    Model(GtfsError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        if self.line > 0 {
            write!(f, ", line {}", self.line)?;
        }
        if let Some(field) = &self.field {
            write!(f, ", field `{}`", field)?;
        }
        match &self.kind {
            ParseErrorKind::Io(e) => write!(f, ": {}", e),
            ParseErrorKind::Csv(e) => write!(f, ": {}", e),
            #[cfg(feature = "geojson")]
            ParseErrorKind::Json(e) => write!(f, ": invalid JSON: {}", e),
            #[cfg(feature = "zip")]
            ParseErrorKind::Zip(e) => write!(f, ": invalid zip archive: {}", e),
            ParseErrorKind::MissingColumn => {
                write!(f, ": required column is missing")
            }
            ParseErrorKind::DuplicateColumn => {
                write!(f, ": duplicate column in header")
            }
            ParseErrorKind::EmptyValue => {
                write!(f, ": required value is empty")
            }
            ParseErrorKind::Invalid { value, expected } => {
                write!(f, ": invalid value '{}' (expected {})", value, expected)
            }
            ParseErrorKind::Model(e) => write!(f, ": {}", e),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ParseErrorKind::Io(e) => Some(e),
            ParseErrorKind::Csv(e) => Some(e),
            #[cfg(feature = "geojson")]
            ParseErrorKind::Json(e) => Some(e),
            #[cfg(feature = "zip")]
            ParseErrorKind::Zip(e) => Some(e),
            ParseErrorKind::Model(e) => Some(e),
            _ => None,
        }
    }
}
