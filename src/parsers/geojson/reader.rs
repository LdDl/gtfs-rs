//! Reading functions for `locations.geojson`.

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::model::{Location, LocationGeometry};
use crate::parsers::{ParseError, ParseErrorKind};

/// Reads GTFS-Flex zones from a `locations.geojson` file path.
///
/// The file name from the path is used in error messages; the path
/// itself may have any name.
///
/// # Arguments
///
/// * `path` - Path to the GeoJSON file
///
/// # Errors
///
/// Returns a [`ParseError`] if the file cannot be opened, is not
/// valid JSON, or violates the GTFS subset of GeoJSON (see
/// [`read_locations_str`]).
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::parsers::{ParseError, geojson};
///
/// fn main() -> Result<(), ParseError> {
///     let zones = geojson::read_locations("feed/locations.geojson")?;
///     for zone in &zones {
///         println!("zone {}", zone.location_id);
///     }
///     Ok(())
/// }
/// ```
pub fn read_locations(path: impl AsRef<Path>) -> Result<Vec<Location>, ParseError> {
    let path = path.as_ref();
    let label = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.display().to_string(),
    };
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            return Err(ParseError {
                file: label,
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            });
        }
    };
    read_locations_str(&label, &data)
}

/// Reads GTFS-Flex zones from GeoJSON text.
///
/// Enforces the GTFS subset of GeoJSON: the root must be a
/// `FeatureCollection`, every feature must have an `id`, and each
/// geometry must be a `Polygon` or a `MultiPolygon` with `[lon, lat]`
/// positions. The optional `stop_name`/`stop_desc` properties are
/// carried over; unknown properties are ignored.
///
/// # Arguments
///
/// * `file_label` - Name used in error messages
///   (e.g. "locations.geojson")
/// * `data` - GeoJSON text
///
/// # Errors
///
/// Returns a [`ParseError`] with [`ParseErrorKind::Json`] for
/// malformed JSON (with the offending line number) or
/// [`ParseErrorKind::Invalid`] for structural violations; the
/// `field` names the JSON path or feature id.
///
/// # Examples
///
/// ```
/// use gtfs_rs::model::LocationGeometry;
/// use gtfs_rs::parsers::{ParseError, geojson};
///
/// fn main() -> Result<(), ParseError> {
///     let data = r#"{
///         "type": "FeatureCollection",
///         "features": [{
///             "type": "Feature",
///             "id": "zone_a",
///             "properties": { "stop_name": "On-demand zone" },
///             "geometry": {
///                 "type": "Polygon",
///                 "coordinates": [[
///                     [37.60, 55.74], [37.65, 55.74],
///                     [37.65, 55.77], [37.60, 55.74]
///                 ]]
///             }
///         }]
///     }"#;
///     let zones = geojson::read_locations_str("locations.geojson", data)?;
///     assert_eq!(zones[0].location_id, "zone_a");
///     assert!(matches!(zones[0].geometry, LocationGeometry::Polygon(_)));
///     Ok(())
/// }
/// ```
pub fn read_locations_str(file_label: &str, data: &str) -> Result<Vec<Location>, ParseError> {
    let root: Value = match serde_json::from_str(data) {
        Ok(root) => root,
        Err(e) => {
            return Err(ParseError {
                file: file_label.to_string(),
                line: e.line() as u64,
                field: None,
                kind: ParseErrorKind::Json(e),
            });
        }
    };

    let root_type = root.get("type").and_then(Value::as_str);
    if root_type != Some("FeatureCollection") {
        return Err(invalid(
            file_label,
            "type",
            root_type.unwrap_or("<missing>"),
            "\"FeatureCollection\"",
        ));
    }
    let features = match root.get("features").and_then(Value::as_array) {
        Some(features) => features,
        None => {
            return Err(invalid(
                file_label,
                "features",
                "<missing or not an array>",
                "an array of Features",
            ));
        }
    };

    let mut locations = Vec::with_capacity(features.len());
    for (index, feature) in features.iter().enumerate() {
        locations.push(parse_feature(file_label, index, feature)?);
    }
    Ok(locations)
}

/// Parses one GeoJSON `Feature` into a [`Location`].
fn parse_feature(file: &str, index: usize, feature: &Value) -> Result<Location, ParseError> {
    let at = |suffix: &str| format!("features[{}]{}", index, suffix);

    let feature_type = feature.get("type").and_then(Value::as_str);
    if feature_type != Some("Feature") {
        return Err(invalid(
            file,
            &at(".type"),
            feature_type.unwrap_or("<missing>"),
            "\"Feature\"",
        ));
    }

    // the spec requires an id; tolerate numeric ids from real feeds
    let id = match feature.get("id") {
        Some(Value::String(id)) => id.clone(),
        Some(Value::Number(id)) => id.to_string(),
        _ => {
            return Err(invalid(
                file,
                &at(".id"),
                "<missing>",
                "a unique location id",
            ));
        }
    };

    let geometry = match feature.get("geometry") {
        Some(geometry) => parse_geometry(file, &at(".geometry"), geometry)?,
        None => {
            return Err(invalid(
                file,
                &at(".geometry"),
                "<missing>",
                "a Polygon or MultiPolygon geometry",
            ));
        }
    };

    let mut location = Location::new(&id, geometry);
    if let Some(properties) = feature.get("properties").and_then(Value::as_object) {
        location.stop_name = properties
            .get("stop_name")
            .and_then(Value::as_str)
            .map(str::to_string);
        location.stop_desc = properties
            .get("stop_desc")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    Ok(location)
}

/// Parses a `Polygon` or `MultiPolygon` geometry object.
fn parse_geometry(
    file: &str,
    path: &str,
    geometry: &Value,
) -> Result<LocationGeometry, ParseError> {
    let geometry_type = geometry.get("type").and_then(Value::as_str);
    let coordinates = geometry.get("coordinates");
    match (geometry_type, coordinates) {
        (Some("Polygon"), Some(coordinates)) => match parse_polygon(coordinates) {
            Some(rings) => Ok(LocationGeometry::Polygon(rings)),
            None => Err(invalid(
                file,
                path,
                "<malformed coordinates>",
                "rings of [lon, lat] positions",
            )),
        },
        (Some("MultiPolygon"), Some(coordinates)) => {
            let polygons = match coordinates.as_array() {
                Some(polygons) => polygons,
                None => {
                    return Err(invalid(
                        file,
                        path,
                        "<malformed coordinates>",
                        "an array of polygons",
                    ));
                }
            };
            let mut out = Vec::with_capacity(polygons.len());
            for polygon in polygons {
                match parse_polygon(polygon) {
                    Some(rings) => out.push(rings),
                    None => {
                        return Err(invalid(
                            file,
                            path,
                            "<malformed coordinates>",
                            "rings of [lon, lat] positions",
                        ));
                    }
                }
            }
            Ok(LocationGeometry::MultiPolygon(out))
        }
        (other, _) => Err(invalid(
            file,
            path,
            other.unwrap_or("<missing>"),
            "\"Polygon\" or \"MultiPolygon\"",
        )),
    }
}

/// Parses polygon coordinates: a list of rings of positions.
fn parse_polygon(coordinates: &Value) -> Option<Vec<Vec<[f64; 2]>>> {
    let rings = coordinates.as_array()?;
    let mut out = Vec::with_capacity(rings.len());
    for ring in rings {
        let positions = ring.as_array()?;
        let mut points = Vec::with_capacity(positions.len());
        for position in positions {
            points.push(parse_position(position)?);
        }
        out.push(points);
    }
    Some(out)
}

/// Parses one `[lon, lat]` position; extra members (altitude) are
/// ignored per RFC 7946.
fn parse_position(position: &Value) -> Option<[f64; 2]> {
    let members = position.as_array()?;
    if members.len() < 2 {
        return None;
    }
    Some([members[0].as_f64()?, members[1].as_f64()?])
}

/// Builds a structural-violation error.
fn invalid(file: &str, field: &str, value: &str, expected: &str) -> ParseError {
    ParseError {
        file: file.to_string(),
        line: 0,
        field: Some(field.to_string()),
        kind: ParseErrorKind::Invalid {
            value: value.to_string(),
            expected: expected.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLEX_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/flex_feed");

    #[test]
    fn test_reads_flex_fixture() -> Result<(), ParseError> {
        let zones = read_locations(format!("{}/locations.geojson", FLEX_DIR))?;
        assert_eq!(zones.len(), 2);
        assert_eq!(zones[0].location_id, "zone_a");
        assert_eq!(zones[0].stop_name.as_deref(), Some("Demo flex zone A"));
        let LocationGeometry::Polygon(rings) = &zones[0].geometry else {
            panic!("expected a Polygon for zone_a");
        };
        // GeoJSON positions are (lon, lat)
        assert_eq!(rings[0][0], [-116.80, 36.85]);
        assert!(matches!(
            zones[1].geometry,
            LocationGeometry::MultiPolygon(_)
        ));
        Ok(())
    }

    #[test]
    fn test_rejects_non_feature_collection() {
        let data = r#"{ "type": "Feature", "id": "x" }"#;
        let Err(err) = read_locations_str("locations.geojson", data) else {
            panic!("expected a root-type error");
        };
        assert_eq!(err.field.as_deref(), Some("type"));
        assert!(matches!(err.kind, ParseErrorKind::Invalid { .. }));
    }

    #[test]
    fn test_rejects_missing_id() {
        let data = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[0.0, 0.0]]] }
            }]
        }"#;
        let Err(err) = read_locations_str("locations.geojson", data) else {
            panic!("expected a missing-id error");
        };
        assert_eq!(err.field.as_deref(), Some("features[0].id"));
    }

    #[test]
    fn test_rejects_unsupported_geometry() {
        let data = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "id": "pt",
                "geometry": { "type": "Point", "coordinates": [0.0, 0.0] }
            }]
        }"#;
        let Err(err) = read_locations_str("locations.geojson", data) else {
            panic!("expected a geometry-type error");
        };
        assert_eq!(err.field.as_deref(), Some("features[0].geometry"));
    }

    #[test]
    fn test_malformed_json_reports_line() {
        let data = "{\n  \"type\": \"FeatureCollection\",\n  broken\n}";
        let Err(err) = read_locations_str("locations.geojson", data) else {
            panic!("expected a JSON syntax error");
        };
        assert_eq!(err.line, 3);
        assert!(matches!(err.kind, ParseErrorKind::Json(_)));
    }
}
