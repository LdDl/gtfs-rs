//! # gtfs-rs
//!
//! Complete in-memory data model of the [GTFS Schedule
//! specification](https://gtfs.org/documentation/schedule/reference/):
//! every dataset file is represented by a typed struct, every
//! enumerated field by a typed enum, collected in the
//! [`GtfsReference`] container.
//!
//! Covered: agencies, stops, routes, trips, stop times, calendars,
//! fares v1 and v2 (media, products, leg/transfer rules, areas,
//! networks, timeframes, rider categories), shapes, frequencies,
//! transfers, pathways, levels, GTFS-Flex (location groups, GeoJSON
//! locations, booking rules), translations, feed info and
//! attributions. Legacy Fares v1 structs carry a `V1` name suffix
//! ([`FareAttributeV1`], [`FareRuleV1`]); unsuffixed fare types
//! belong to the current Fares v2 framework.
//!
//! Datasets are populated programmatically or read with the
//! feature-gated parsers (see below), and written back with the
//! zero-dependency [`writers`]; entity structs mirror the spec
//! field-for-field. Structural validation is built in:
//! [`GtfsReference::validate`] reports every intra-record,
//! uniqueness and referential-integrity problem at once.
//! Deliberately out of scope: GTFS Realtime.
//!
//! # Crate layout
//!
//! The source is split into two layers:
//!
//! - [`misc`] - field-level value types shared by many dataset
//!   files: [`GtfsDate`] (`YYYYMMDD` dates), [`CurrencyAmount`]
//!   (exact decimal money), [`Weekday`] and the
//!   [`parse_gtfs_time`]/[`format_gtfs_time`] helpers for `HH:MM:SS`
//!   values;
//! - [`model`] - the entities: one struct per dataset file
//!   (`agency.txt` is [`Agency`], `stops.txt` is [`Stop`], ...) and
//!   one enum per enumerated field, with `from_code`/`code`
//!   conversions.
//!
//! On top of them sit [`GtfsReference`] - the whole dataset as a
//! single value with lookup helpers - the [`validate`] module with
//! the structural checks, the [`writers`] module serializing
//! datasets back to CSV tables, `locations.geojson` and unpacked
//! directories (no extra dependencies), and the [`GtfsError`] error
//! type.
//! Everything is also re-exported flat from the crate root, so
//! downstream code can simply import `gtfs_rs::Stop` instead of the
//! full module path.
//!
//! # Cargo features
//!
//! - `parse` (off by default) - the `parsers` module with CSV
//!   readers for the dataset tables; adds the `csv` dependency.
//! - `geojson` (off by default, implies `parse`) - the
//!   `parsers::geojson` module reading GTFS-Flex
//!   `locations.geojson`; adds the `serde_json` dependency.
//! - `zip` (off by default, implies `parse`) - the `parsers::zip`
//!   module reading whole zipped feeds and the `writers::zip` module
//!   packing them back; adds the `zip` dependency.
//!
//! # Examples
//!
//! ```
//! use gtfs_rs::{GtfsReference, Route, RouteType, Stop, StopTime, Trip};
//!
//! let mut gtfs = GtfsReference::new();
//! gtfs.stops.push(Stop::new("A").with_name("Alpha").with_coordinates(55.75, 37.62));
//! gtfs.routes.push(Route::new("L1", RouteType::Tram).with_short_name("1"));
//! gtfs.trips.push(Trip::new("t0", "L1", "daily"));
//! gtfs.stop_times.push(StopTime::new("t0", "A", 0, 8 * 3600));
//!
//! assert_eq!(gtfs.stop("A").and_then(|s| s.stop_name.as_deref()), Some("Alpha"));
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod error;
pub mod misc;
pub mod model;
#[cfg(feature = "parse")]
pub mod parsers;
mod reference;
pub mod validate;
pub mod writers;

pub use error::GtfsError;
pub use misc::{CurrencyAmount, GtfsDate, Weekday, format_gtfs_time, parse_gtfs_time};
pub use model::*;
pub use reference::GtfsReference;
pub use validate::{Rule, Severity, ValidationIssue, ValidationReport};
pub use writers::{WriteError, WriteErrorKind};
