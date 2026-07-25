//! # Feed Parsers
//!
//! Readers that turn dataset files into the `model` entities. One
//! submodule per format; the shared [`ParseError`] carries full
//! context (file, line, field) for every failure.
//!
//! Currently available:
//!
//! - [`csv`] - the `.txt` tables of a GTFS dataset (`parse` cargo
//!   feature);
//! - [`geojson`] - the GTFS-Flex `locations.geojson` zones
//!   (`geojson` cargo feature, implies `parse`);
//! - [`read_dir`] - a whole unpacked feed directory, orchestrating
//!   the format parsers above.
//!
//! Planned: `zip` (whole feed archives).

pub mod csv;
mod error;
mod feed;
#[cfg(feature = "geojson")]
pub mod geojson;

pub use feed::read_dir;

pub use error::{ParseError, ParseErrorKind};
