//! # CSV Parser
//!
//! Header-driven readers for the `.txt` tables of a GTFS dataset.
//! Column order does not matter and unrecognized columns are
//! ignored, as the specification requires. Values are trimmed; an
//! empty value maps to `None` for optional fields.
//!
//! Read one table from a path with [`read_path`] (e.g. only
//! `agency.txt`), a whole unpacked feed directory with [`read_dir`],
//! or any [`std::io::Read`] source with [`read`]. Entities implement
//! [`CsvRecord`]; the same trait can be implemented for custom types
//! to read GTFS extensions.
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
mod stop_times;
mod stops;
#[cfg(test)]
mod test_support;
mod transfers;
mod translations;
mod trips;

pub use reader::{read, read_dir, read_path};
pub use record::CsvRecord;
pub use row::Row;
pub(crate) use row::opt_string;
