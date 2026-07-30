//! # GeoJSON Writer
//!
//! Serializes GTFS-Flex zones back into `locations.geojson` - a
//! `FeatureCollection` of `Feature`s with `Polygon`/`MultiPolygon`
//! geometries, as the specification requires. Writing needs no JSON
//! dependency, so this module is part of the core (unlike the
//! reader, which sits behind the `geojson` cargo feature).
//!
//! Write with [`write_locations`] (into any writer),
//! [`write_locations_path`] (to a file) or take the text with
//! [`locations_to_string`].

mod writer;

pub use writer::{locations_to_string, write_locations, write_locations_path};
