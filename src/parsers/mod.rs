//! # Feed Parsers
//!
//! Readers that turn dataset files into the `model` entities. One
//! submodule per format; the shared [`ParseError`] carries full
//! context (file, line, field) for every failure.
//!
//! Currently available (behind the `parse` cargo feature):
//!
//! - [`csv`] - the `.txt` tables of a GTFS dataset.
//!
//! Planned: `geojson` (`locations.geojson`), `zip` (whole feed
//! archives).

pub mod csv;
mod error;

pub use error::{ParseError, ParseErrorKind};
