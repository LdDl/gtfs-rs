//! Writing functions for `locations.geojson`.

use std::fs::File;
use std::io;
use std::path::Path;

use crate::model::{Location, LocationGeometry};
use crate::writers::{WriteError, WriteErrorKind};

/// Renders GTFS-Flex zones as `locations.geojson` text: a GeoJSON
/// `FeatureCollection` per RFC 7946, indented for readability.
///
/// Linear rings that are stored open (last point differing from the
/// first) are closed automatically on output, as RFC 7946 requires.
///
/// # Arguments
///
/// * `locations` - The zones to render
///
/// # Examples
///
/// ```
/// use gtfs_rs::model::{Location, LocationGeometry};
/// use gtfs_rs::writers::geojson;
///
/// let zone = Location::new(
///     "zone_a",
///     LocationGeometry::Polygon(vec![vec![
///         [37.60, 55.74],
///         [37.65, 55.74],
///         [37.65, 55.77],
///         [37.60, 55.74],
///     ]]),
/// )
/// .with_name("On-demand zone");
///
/// let text = geojson::locations_to_string(&[zone]);
/// assert!(text.contains("\"FeatureCollection\""));
/// assert!(text.contains("\"id\": \"zone_a\""));
/// assert!(text.contains("\"stop_name\": \"On-demand zone\""));
/// ```
pub fn locations_to_string(locations: &[Location]) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"type\": \"FeatureCollection\",\n  \"features\": [");
    let mut first = true;
    for location in locations {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str("\n    {\n      \"type\": \"Feature\",\n      \"id\": ");
        push_json_string(&mut out, &location.location_id);
        out.push_str(",\n      \"properties\": {");
        let mut first_property = true;
        for (key, value) in [
            ("stop_name", &location.stop_name),
            ("stop_desc", &location.stop_desc),
        ] {
            if let Some(value) = value {
                if !first_property {
                    out.push(',');
                }
                first_property = false;
                out.push_str("\n        \"");
                out.push_str(key);
                out.push_str("\": ");
                push_json_string(&mut out, value);
            }
        }
        if !first_property {
            out.push_str("\n      ");
        }
        out.push_str("},\n      \"geometry\": {\n        \"type\": ");
        match &location.geometry {
            LocationGeometry::Polygon(rings) => {
                out.push_str("\"Polygon\",\n        \"coordinates\": ");
                push_polygon(&mut out, rings);
            }
            LocationGeometry::MultiPolygon(polygons) => {
                out.push_str("\"MultiPolygon\",\n        \"coordinates\": [");
                let mut first_polygon = true;
                for rings in polygons {
                    if !first_polygon {
                        out.push_str(", ");
                    }
                    first_polygon = false;
                    push_polygon(&mut out, rings);
                }
                out.push(']');
            }
        }
        out.push_str("\n      }\n    }");
    }
    if !first {
        out.push_str("\n  ");
    }
    out.push_str("]\n}\n");
    out
}

/// Writes GTFS-Flex zones as `locations.geojson` into any writer.
///
/// # Arguments
///
/// * `file_label` - Name used in error messages
///   (e.g. "locations.geojson")
/// * `locations` - The zones to write
/// * `out` - Destination of the GeoJSON bytes
///
/// # Errors
///
/// Returns a [`WriteError`] when the underlying writer fails.
///
/// # Examples
///
/// ```
/// use gtfs_rs::model::{Location, LocationGeometry};
/// use gtfs_rs::writers::{WriteError, geojson};
///
/// fn main() -> Result<(), WriteError> {
///     let zone = Location::new(
///         "zone_a",
///         LocationGeometry::Polygon(vec![vec![
///             [37.60, 55.74],
///             [37.65, 55.74],
///             [37.65, 55.77],
///             [37.60, 55.74],
///         ]]),
///     );
///     let mut out = Vec::new();
///     geojson::write_locations("locations.geojson", &[zone], &mut out)?;
///     assert!(!out.is_empty());
///     Ok(())
/// }
/// ```
pub fn write_locations<W: io::Write>(
    file_label: &str,
    locations: &[Location],
    mut out: W,
) -> Result<(), WriteError> {
    let text = locations_to_string(locations);
    if let Err(e) = out.write_all(text.as_bytes()) {
        return Err(WriteError {
            file: file_label.to_string(),
            kind: WriteErrorKind::Io(e),
        });
    }
    Ok(())
}

/// Writes GTFS-Flex zones to a `locations.geojson` file path,
/// creating or overwriting the file.
///
/// # Arguments
///
/// * `locations` - The zones to write
/// * `path` - Destination file path
///
/// # Errors
///
/// Returns a [`WriteError`] if the file cannot be created or
/// written.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::model::{Location, LocationGeometry};
/// use gtfs_rs::writers::{WriteError, geojson};
///
/// fn main() -> Result<(), WriteError> {
///     let zone = Location::new(
///         "zone_a",
///         LocationGeometry::Polygon(vec![vec![
///             [37.60, 55.74],
///             [37.65, 55.74],
///             [37.65, 55.77],
///             [37.60, 55.74],
///         ]]),
///     );
///     geojson::write_locations_path(&[zone], "out/locations.geojson")?;
///     Ok(())
/// }
/// ```
pub fn write_locations_path(
    locations: &[Location],
    path: impl AsRef<Path>,
) -> Result<(), WriteError> {
    let path = path.as_ref();
    let label = path.display().to_string();
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(WriteError {
                file: label,
                kind: WriteErrorKind::Io(e),
            });
        }
    };
    write_locations(&label, locations, file)
}

/// Appends one polygon (a list of rings) as GeoJSON coordinates.
/// RFC 7946 requires linear rings to be closed (first point ==
/// last point); rings stored open are closed on the fly so that the
/// output is always valid GeoJSON and survives a write/read
/// roundtrip.
fn push_polygon(out: &mut String, rings: &[Vec<[f64; 2]>]) {
    out.push('[');
    let mut first_ring = true;
    for ring in rings {
        if !first_ring {
            out.push_str(", ");
        }
        first_ring = false;
        out.push('[');
        let mut first_point = true;
        for [lon, lat] in ring {
            if !first_point {
                out.push_str(", ");
            }
            first_point = false;
            push_point(out, *lon, *lat);
        }
        if let (Some(first), Some(last)) = (ring.first(), ring.last())
            && first != last
        {
            out.push_str(", ");
            push_point(out, first[0], first[1]);
        }
        out.push(']');
    }
    out.push(']');
}

/// Appends one `[lon, lat]` coordinate pair. Coordinates must be
/// finite: JSON has no representation for NaN/infinity, so such
/// values would render the output unparsable (the crate's own
/// parsers never produce them).
fn push_point(out: &mut String, lon: f64, lat: f64) {
    debug_assert!(
        lon.is_finite() && lat.is_finite(),
        "non-finite coordinates produce invalid JSON"
    );
    out.push('[');
    out.push_str(&lon.to_string());
    out.push_str(", ");
    out.push_str(&lat.to_string());
    out.push(']');
}

/// Appends a JSON string literal with the required escaping.
fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "geojson")]
    use crate::parsers::geojson::read_locations_str;

    fn zone() -> Location {
        Location::new(
            "zone_a",
            LocationGeometry::Polygon(vec![vec![
                [-116.80, 36.85],
                [-116.70, 36.85],
                [-116.70, 36.95],
                [-116.80, 36.85],
            ]]),
        )
        .with_name("Demo \"quoted\" zone")
    }

    #[test]
    fn test_renders_escaped_json() {
        let text = locations_to_string(&[zone()]);
        assert!(text.contains("\\\"quoted\\\""));
        assert!(text.contains("[-116.8, 36.85]"));
    }

    #[test]
    fn test_closes_open_rings() {
        let open = Location::new(
            "zone_open",
            LocationGeometry::Polygon(vec![vec![
                [-116.80, 36.85],
                [-116.70, 36.85],
                [-116.70, 36.95],
            ]]),
        );
        let text = locations_to_string(&[open]);
        assert!(
            text.contains("[[-116.8, 36.85], [-116.7, 36.85], [-116.7, 36.95], [-116.8, 36.85]]")
        );
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_roundtrip_through_reader() -> Result<(), crate::parsers::ParseError> {
        let text = locations_to_string(&[zone()]);
        let parsed = read_locations_str("locations.geojson", &text)?;
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].location_id, "zone_a");
        assert_eq!(parsed[0].stop_name.as_deref(), Some("Demo \"quoted\" zone"));
        let LocationGeometry::Polygon(rings) = &parsed[0].geometry else {
            panic!("expected a Polygon");
        };
        assert_eq!(rings[0].len(), 4);
        assert_eq!(rings[0][0], [-116.80, 36.85]);
        Ok(())
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_open_ring_roundtrips_closed() -> Result<(), crate::parsers::ParseError> {
        let open = Location::new(
            "zone_open",
            LocationGeometry::Polygon(vec![vec![
                [-116.80, 36.85],
                [-116.70, 36.85],
                [-116.70, 36.95],
            ]]),
        );
        let text = locations_to_string(&[open]);
        let parsed = read_locations_str("locations.geojson", &text)?;
        let LocationGeometry::Polygon(rings) = &parsed[0].geometry else {
            panic!("expected a Polygon");
        };
        assert_eq!(rings[0].len(), 4);
        assert_eq!(rings[0][0], rings[0][3]);
        Ok(())
    }
}
