//! `areas.txt` and `stop_areas.txt` - fare area grouping of stops,
//! matched by the fare leg rules (GTFS-Fares v2).
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#areastxt>

/// A fare area from `areas.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Area, StopArea};
///
/// let zone = Area::new("zone_a").with_name("Zone A");
/// let assignment = StopArea::new("zone_a", "S1");
/// assert_eq!(assignment.area_id, zone.area_id);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Area {
    /// Identifies an area. Must be unique in `areas.txt`. Unique
    /// ID. Required.
    pub area_id: String,
    /// The name of the area as displayed to the rider. Optional;
    /// `None` means the value is empty in the file.
    pub area_name: Option<String>,
}

impl Area {
    /// Creates an area.
    ///
    /// # Arguments
    ///
    /// * `area_id` - Unique area identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Area;
    ///
    /// let zone = Area::new("zone_a");
    /// assert_eq!(zone.area_id, "zone_a");
    /// assert_eq!(zone.area_name, None);
    /// ```
    pub fn new(area_id: &str) -> Self {
        Area {
            area_id: area_id.to_string(),
            area_name: None,
        }
    }

    /// Sets the rider-facing name.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Area;
    ///
    /// let zone = Area::new("zone_a").with_name("Zone A");
    /// assert_eq!(zone.area_name.as_deref(), Some("Zone A"));
    /// ```
    pub fn with_name(mut self, area_name: &str) -> Self {
        self.area_name = Some(area_name.to_string());
        self
    }
}

/// A stop-to-area assignment from `stop_areas.txt`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct StopArea {
    /// Identifies an area to which one or multiple `stop_id`s
    /// belong. The same `stop_id` may be defined in many
    /// `area_id`s. Foreign ID referencing `areas.area_id`.
    /// Required.
    pub area_id: String,
    /// Identifies a stop. If a station (i.e. a stop with
    /// `stops.location_type = 1`) is defined in this field, it is
    /// assumed that all of its platforms (i.e. all stops with
    /// `stops.location_type = 0` that have this station defined as
    /// `stops.parent_station`) are part of the same area. This
    /// behavior can be overridden by assigning platforms to other
    /// areas. Foreign ID referencing `stops.stop_id`. Required.
    pub stop_id: String,
}

impl StopArea {
    /// Creates a stop-to-area assignment.
    ///
    /// # Arguments
    ///
    /// * `area_id` - Area the stop belongs to
    /// * `stop_id` - Assigned stop or station
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::StopArea;
    ///
    /// let assignment = StopArea::new("zone_a", "S1");
    /// assert_eq!(assignment.area_id, "zone_a");
    /// assert_eq!(assignment.stop_id, "S1");
    /// ```
    pub fn new(area_id: &str, stop_id: &str) -> Self {
        StopArea {
            area_id: area_id.to_string(),
            stop_id: stop_id.to_string(),
        }
    }
}
