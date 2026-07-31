//! # CSV Writer
//!
//! Serializes the `.txt` tables of a GTFS dataset. Every table is
//! written with its full specification header; absent optional
//! values become empty fields, and values are quoted per RFC 4180
//! when they contain separators, quotes or line breaks.
//!
//! Write one table with its named shortcut (e.g.
//! [`write_agencies`]) or the generic [`write_path`]/[`write()`];
//! whole feeds are written one level up by
//! [`write_dir`](crate::writers::write_dir). Entities implement
//! [`CsvWrite`]; implement it for custom types to write GTFS
//! extension tables with the same machinery.
//!
//! One normalization to be aware of: values are written verbatim,
//! but the `parsers::csv` readers trim surrounding whitespace, so
//! leading/trailing spaces do not survive a write/read roundtrip
//! (`"  x  "` reads back as `"x"`, an all-spaces value as an absent
//! one). GTFS strongly discourages such padding in the first place.
//!
//! Module layout: the [`CsvWrite`] trait and the field escaping live
//! in `table.rs`, the writing functions in `writer.rs`, the named
//! shortcuts in `shortcuts.rs`, and each GTFS table has its own file
//! with the entity's `CsvWrite` implementation and tests, mirroring
//! `parsers::csv`.

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
mod routes;
mod shapes;
mod shortcuts;
mod stop_times;
mod stops;
mod table;
mod transfers;
mod translations;
mod trips;
mod writer;

pub use shortcuts::*;
pub use table::CsvWrite;
pub use writer::{write, write_path};
