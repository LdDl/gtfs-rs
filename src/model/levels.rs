//! `levels.txt` - levels within stations, used with pathways to
//! navigate multi-floor stations.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#levelstxt>

/// A station level from `levels.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::Level;
///
/// let concourse = Level::new("L-1", -1.0).with_name("Underground concourse");
/// assert_eq!(concourse.level_index, -1.0);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Level {
    /// Identifies a level in a station. Must be unique in the
    /// dataset.
    ///
    /// Presence: Required.
    pub level_id: String,
    /// Numeric index of the level that indicates its relative
    /// position.
    ///
    /// Ground level should have index `0`, with levels above ground
    /// indicated by positive indices and levels below ground by
    /// negative indices.
    ///
    /// Presence: Required. Float.
    pub level_index: f64,
    /// Name of the level as seen by the rider inside the building or
    /// station.
    ///
    /// Example: take the elevator to "Mezzanine" or "Platform" or
    /// "-1".
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub level_name: Option<String>,
}

impl Level {
    /// Creates a level.
    ///
    /// # Arguments
    ///
    /// * `level_id` - Unique level identifier
    /// * `level_index` - Numeric index, 0 = ground level
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Level;
    ///
    /// let level = Level::new("L-1", -1.0);
    /// assert_eq!(level.level_id, "L-1");
    /// assert_eq!(level.level_index, -1.0);
    /// assert!(level.level_name.is_none());
    /// ```
    pub fn new(level_id: &str, level_index: f64) -> Self {
        Level {
            level_id: level_id.to_string(),
            level_index,
            level_name: None,
        }
    }

    /// Sets the rider-facing name.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::Level;
    ///
    /// let level = Level::new("L-1", -1.0).with_name("Concourse");
    /// assert_eq!(level.level_name.as_deref(), Some("Concourse"));
    /// ```
    pub fn with_name(mut self, level_name: &str) -> Self {
        self.level_name = Some(level_name.to_string());
        self
    }
}
