//! # GeoJSON Parser
//!
//! Reader for `locations.geojson` - the GTFS-Flex zones where riders
//! can request pickup or drop off by on-demand services. The file is
//! a strict subset of GeoJSON (RFC 7946): a `FeatureCollection` of
//! `Feature`s with required `id`s and `Polygon`/`MultiPolygon`
//! geometries. Available with the `geojson` cargo feature (implies
//! `parse`).
//!
//! Read the file with [`read_locations`] (by path) or
//! [`read_locations_str`] (from a string). With the feature enabled,
//! [`read_dir`](crate::parsers::read_dir) also picks the file up
//! automatically.

mod reader;

pub use reader::{read_locations, read_locations_str};
