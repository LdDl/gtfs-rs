//! # GTFS Value Types
//!
//! Field-level primitives shared by the entities in `model`: the
//! `YYYYMMDD` service date and the `HH:MM:SS` time encoding. These
//! describe how GTFS encodes single values, not dataset files.

mod date;
mod time;

pub use date::{GtfsDate, Weekday};
pub use time::{format_gtfs_time, parse_gtfs_time};
