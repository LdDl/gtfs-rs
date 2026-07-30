//! # CSV Parser
//!
//! Header-driven readers for the `.txt` tables of a GTFS dataset.
//! Column order does not matter and unrecognized columns are
//! ignored, as the specification requires. Values are trimmed; an
//! empty value maps to `None` for optional fields.
//!
//! Read one table from a path with its named shortcut (e.g.
//! [`read_agencies`]) or from any [`std::io::Read`] source with
//! [`read`]. The generic [`read_path`] underlies the shortcuts and
//! serves custom extension tables: entities implement [`CsvRecord`],
//! and the same trait can be implemented for user-defined types.
//! Whole unpacked feed directories are read one level up, by
//! [`read_dir`](crate::parsers::read_dir), which orchestrates this
//! module and the `geojson` parser.
//!
//! Module layout: [`Row`] and its typed accessors live in `row.rs`,
//! the [`CsvRecord`] trait in `record.rs`, the reading functions in
//! `reader.rs`, and each GTFS table has its own file with the
//! entity's `CsvRecord` implementation and tests, mirroring the
//! `model` module layout.

mod agency;
mod areas;
mod attributions;
mod booking;
mod calendar;
mod fares;
mod feed_info;
mod frequencies;
mod levels;
mod locations;
mod networks;
mod pathways;
mod reader;
mod record;
mod routes;
mod row;
mod shapes;
mod shortcuts;
mod stop_times;
mod stops;
#[cfg(test)]
pub mod test_support;
mod transfers;
mod translations;
mod trips;

pub use reader::{read, read_path};
pub use record::CsvRecord;
pub use row::Row;
pub use shortcuts::*;
