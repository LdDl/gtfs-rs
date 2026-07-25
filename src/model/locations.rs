//! `location_groups.txt`, `location_group_stops.txt` and
//! `locations.geojson` - on-demand service locations (GTFS-Flex).
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#location_groupstxt>

/// A group of stops from `location_groups.txt` (GTFS-Flex).
///
/// Defines location groups, which are groups of stops where a rider
/// may request pickup or drop off. Stop times may reference a
/// location group to describe on-demand service at any of its stops.
///
/// The file is optional; its primary key is (`location_group_id`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LocationGroup {
    /// Identifies a location group. ID must be unique across all
    /// `stops.stop_id`, `locations.geojson` `id`, and
    /// `location_groups.location_group_id` values.
    ///
    /// A location group is a group of stops that together indicate
    /// locations where a rider may request pickup or drop off.
    ///
    /// Required.
    pub location_group_id: String,
    /// The name of the location group as displayed to the rider.
    ///
    /// Optional; `None` when the value is empty in the file.
    pub location_group_name: Option<String>,
}

impl LocationGroup {
    /// Creates a location group.
    ///
    /// # Arguments
    ///
    /// * `location_group_id` - Unique location group identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::LocationGroup;
    ///
    /// let lg = LocationGroup::new("lg1");
    /// assert_eq!(lg.location_group_id, "lg1");
    /// assert!(lg.location_group_name.is_none());
    /// ```
    pub fn new(location_group_id: &str) -> Self {
        LocationGroup {
            location_group_id: location_group_id.to_string(),
            location_group_name: None,
        }
    }

    /// Sets the rider-facing name.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::LocationGroup;
    ///
    /// let lg = LocationGroup::new("lg1").with_name("Flex zone A");
    /// assert_eq!(lg.location_group_name.as_deref(), Some("Flex zone A"));
    /// ```
    pub fn with_name(mut self, location_group_name: &str) -> Self {
        self.location_group_name = Some(location_group_name.to_string());
        self
    }
}

/// A stop-to-location-group assignment from
/// `location_group_stops.txt` (GTFS-Flex).
///
/// Assigns stops from `stops.txt` to location groups. The file is
/// optional; all of its fields form the primary key.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LocationGroupStop {
    /// Identifies a location group to which one or multiple
    /// `stop_id`s belong. The same `stop_id` may be defined in many
    /// `location_group_id`s.
    ///
    /// Foreign ID referencing `location_groups.location_group_id`.
    /// Required.
    pub location_group_id: String,
    /// Identifies a stop belonging to the location group.
    ///
    /// Foreign ID referencing `stops.stop_id`. Required.
    pub stop_id: String,
}

impl LocationGroupStop {
    /// Creates a stop-to-location-group assignment.
    ///
    /// # Arguments
    ///
    /// * `location_group_id` - Location group the stop belongs to
    /// * `stop_id` - Assigned stop
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::LocationGroupStop;
    ///
    /// let lgs = LocationGroupStop::new("lg1", "S1");
    /// assert_eq!(lgs.location_group_id, "lg1");
    /// assert_eq!(lgs.stop_id, "S1");
    /// ```
    pub fn new(location_group_id: &str, stop_id: &str) -> Self {
        LocationGroupStop {
            location_group_id: location_group_id.to_string(),
            stop_id: stop_id.to_string(),
        }
    }
}

/// Geometry of a GeoJSON location, in GeoJSON coordinate order
/// (longitude, latitude).
///
/// Per the spec, the `geometry` object of a `locations.geojson`
/// feature must be of type `"Polygon"` or `"MultiPolygon"`, and each
/// polygon must be valid by the definition of the OpenGIS Simple
/// Features Specification, section 6.1.11.
///
/// A polygon is a list of rings, each ring a closed list of
/// `[lon, lat]` positions; the first ring is the exterior, the rest
/// are holes. This type models the GeoJSON geometry without serde.
#[derive(Debug, Clone)]
pub enum LocationGeometry {
    /// A single polygon (GeoJSON `"Polygon"` geometry type): a list
    /// of rings, each ring a closed list of `[lon, lat]` positions
    Polygon(Vec<Vec<[f64; 2]>>),
    /// Multiple polygons (GeoJSON `"MultiPolygon"` geometry type):
    /// a list of polygons, each a list of rings
    MultiPolygon(Vec<Vec<Vec<[f64; 2]>>>),
}

/// A zone from `locations.geojson` (GTFS-Flex).
///
/// Defines zones where riders can request either pickup or drop off
/// by on-demand services, referenced by `stop_times.location_id`.
/// These zones are represented as GeoJSON polygons.
///
/// The optional `locations.geojson` file uses a subset of the
/// GeoJSON format, described in RFC 7946, and must contain a
/// `FeatureCollection` defining the various stop locations where
/// riders may request pickup or drop off. Every GeoJSON `Feature`
/// must have an `id`; the `id` must be unique across all
/// `stops.stop_id`, `locations.geojson` `id`, and
/// `location_group_id` values.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Location, LocationGeometry};
///
/// // one exterior ring in GeoJSON (lon, lat) order, closed
/// let zone = Location::new(
///     "flex_zone_1",
///     LocationGeometry::Polygon(vec![vec![
///         [37.60, 55.74],
///         [37.65, 55.74],
///         [37.65, 55.77],
///         [37.60, 55.74],
///     ]]),
/// )
/// .with_name("On-demand zone");
/// assert!(zone.stop_desc.is_none());
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Location {
    /// Identifies a location (the GeoJSON feature `id`). ID must be
    /// unique across all `stops.stop_id`, `locations.geojson` `id`,
    /// and `location_groups.location_group_id` values.
    ///
    /// Required.
    pub location_id: String,
    /// Indicates the name of the location as displayed to riders
    /// (GeoJSON property `stop_name`).
    ///
    /// Optional; `None` when the property is absent.
    pub stop_name: Option<String>,
    /// Meaningful description of the location to help orient riders
    /// (GeoJSON property `stop_desc`).
    ///
    /// Optional; `None` when the property is absent.
    pub stop_desc: Option<String>,
    /// Geometry of the location (the GeoJSON `geometry` object).
    /// Must be of type `"Polygon"` or `"MultiPolygon"`, with the
    /// geographic coordinates defining the geometry of the location
    /// in GeoJSON (longitude, latitude) order; see
    /// [`LocationGeometry`].
    ///
    /// Required.
    pub geometry: LocationGeometry,
}

impl Location {
    /// Creates a location zone.
    ///
    /// # Arguments
    ///
    /// * `location_id` - Unique location identifier
    /// * `geometry` - Zone geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Location, LocationGeometry};
    ///
    /// let ring = vec![
    ///     [37.60, 55.74], [37.65, 55.74], [37.65, 55.77], [37.60, 55.74],
    /// ];
    /// let geom = LocationGeometry::Polygon(vec![ring]);
    /// let zone = Location::new("zone1", geom);
    /// assert_eq!(zone.location_id, "zone1");
    /// ```
    pub fn new(location_id: &str, geometry: LocationGeometry) -> Self {
        Location {
            location_id: location_id.to_string(),
            stop_name: None,
            stop_desc: None,
            geometry,
        }
    }

    /// Sets the rider-facing name.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Location, LocationGeometry};
    ///
    /// let ring = vec![
    ///     [37.60, 55.74], [37.65, 55.74], [37.65, 55.77], [37.60, 55.74],
    /// ];
    /// let geom = LocationGeometry::Polygon(vec![ring]);
    /// let zone = Location::new("zone1", geom).with_name("Downtown zone");
    /// assert_eq!(zone.stop_name.as_deref(), Some("Downtown zone"));
    /// ```
    pub fn with_name(mut self, stop_name: &str) -> Self {
        self.stop_name = Some(stop_name.to_string());
        self
    }
}
