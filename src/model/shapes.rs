//! `shapes.txt` - geospatial paths of vehicle travel, one point per
//! record.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#shapestxt>

/// A single point of a shape from `shapes.txt`.
///
/// A shape is the set of points sharing a `shape_id`, ordered by
/// `shape_pt_sequence`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::ShapePoint;
///
/// let pt = ShapePoint::new("sh1", 55.751, 37.617, 1).with_dist_traveled(0.0);
/// assert_eq!(pt.shape_pt_sequence, 1);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ShapePoint {
    /// Identifies a shape.
    ///
    /// Presence: Required.
    pub shape_id: String,
    /// Latitude of a shape point. Each record in `shapes.txt`
    /// represents a shape point used to define the shape. WGS84
    /// latitude in decimal degrees.
    ///
    /// Presence: Required.
    pub shape_pt_lat: f64,
    /// Longitude of a shape point. WGS84 longitude in decimal
    /// degrees.
    ///
    /// Presence: Required.
    pub shape_pt_lon: f64,
    /// Sequence in which the shape points connect to form the shape.
    /// Values must increase along the trip but do not need to be
    /// consecutive.
    ///
    /// Presence: Required. Non-negative integer.
    pub shape_pt_sequence: u32,
    /// Actual distance traveled along the shape from the first shape
    /// point to the point specified in this record. Used by trip
    /// planners to show the correct portion of the shape on a map.
    /// Values must increase along with `shape_pt_sequence`; they must
    /// not be used to show reverse travel along a route. Distance
    /// units must be consistent with those used in `stop_times.txt`.
    ///
    /// Recommended for routes that have looping or inlining (the
    /// vehicle crosses or travels over the same portion of alignment
    /// in one trip). If a vehicle retraces or crosses the route
    /// alignment at points in the course of a trip,
    /// `shape_dist_traveled` is important to clarify how the points
    /// in `shapes.txt` correspond with records in `stop_times.txt`.
    /// See `stop_times.shape_dist_traveled`.
    ///
    /// Presence: Optional. Non-negative float; `None` when the value
    /// is empty in the file.
    pub shape_dist_traveled: Option<f64>,
}

impl ShapePoint {
    /// Creates a shape point.
    ///
    /// # Arguments
    ///
    /// * `shape_id` - Shape the point belongs to
    /// * `lat` - WGS84 latitude
    /// * `lon` - WGS84 longitude
    /// * `shape_pt_sequence` - Order of the point along the shape
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::ShapePoint;
    ///
    /// let pt = ShapePoint::new("sh1", 55.751, 37.617, 1);
    /// assert_eq!(pt.shape_id, "sh1");
    /// assert_eq!(pt.shape_pt_sequence, 1);
    /// assert!(pt.shape_dist_traveled.is_none());
    /// ```
    pub fn new(shape_id: &str, lat: f64, lon: f64, shape_pt_sequence: u32) -> Self {
        ShapePoint {
            shape_id: shape_id.to_string(),
            shape_pt_lat: lat,
            shape_pt_lon: lon,
            shape_pt_sequence,
            shape_dist_traveled: None,
        }
    }

    /// Sets the distance traveled along the shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::ShapePoint;
    ///
    /// let pt = ShapePoint::new("sh1", 55.751, 37.617, 1)
    ///     .with_dist_traveled(0.0);
    /// assert_eq!(pt.shape_dist_traveled, Some(0.0));
    /// ```
    pub fn with_dist_traveled(mut self, shape_dist_traveled: f64) -> Self {
        self.shape_dist_traveled = Some(shape_dist_traveled);
        self
    }
}
