//! # Feed Writers
//!
//! Serialization of a [`GtfsReference`](crate::GtfsReference) back
//! to dataset files - the inverse of
//! [`parsers`](crate::parsers). Writing needs no dependencies, so
//! everything here is part of the core; only the zip container
//! (`writers::zip`) sits behind the `zip` cargo feature.
//!
//! - [`csv`] - the `.txt` tables, one file or all at once;
//! - [`geojson`] - the GTFS-Flex `locations.geojson` zones;
//! - [`write_dir`] - a whole feed into a directory, orchestrating
//!   the format writers above;
//! - `zip` - a whole feed into an archive (`zip` cargo feature).
//!
//! Every failure is a [`WriteError`] carrying the file name it
//! happened in.

pub mod csv;
mod error;
mod feed;
pub mod geojson;
#[cfg(feature = "zip")]
pub mod zip;

pub use error::{WriteError, WriteErrorKind};
pub use feed::write_dir;
