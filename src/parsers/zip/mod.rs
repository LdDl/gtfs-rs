//! # Zip Parser
//!
//! Reader for zipped GTFS feeds - the form feeds are actually
//! distributed in most of time. The tables must sit at the archive root,
//! as the specification requires. Available with the `zip` cargo feature
//! (implies `parse`); with `geojson` also enabled, a bundled
//! `locations.geojson` is read too.
//!
//! Read an archive from a file path with [`read_zip`] or from bytes
//! already in memory with [`read_zip_bytes`]; both fill the same
//! [`GtfsReference`](crate::GtfsReference) as
//! [`read_dir`](crate::parsers::read_dir) does for unpacked
//! directories.

mod reader;

pub use reader::{read_zip, read_zip_bytes};
