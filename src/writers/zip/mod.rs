//! # Zip Writer
//!
//! Packs a whole [`GtfsReference`](crate::GtfsReference) into a zip
//! archive - the form feeds are distributed in. Available with the
//! `zip` cargo feature; the table selection mirrors
//! [`write_dir`](crate::writers::write_dir), and `locations.geojson`
//! is included when there are zones.
//!
//! Write to a file path with [`write_zip`] or take the raw archive
//! bytes with [`write_zip_bytes`].

mod writer;

pub use writer::{write_zip, write_zip_bytes};
