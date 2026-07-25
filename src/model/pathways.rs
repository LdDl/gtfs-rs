//! `pathways.txt` - graph of edges connecting locations within
//! stations.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#pathwaystxt>

gtfs_enum! {
    /// Type of pathway between two station locations
    /// (`pathway_mode`).
    PathwayMode {
        /// Walkway (`1`)
        Walkway = 1,
        /// Stairs (`2`)
        Stairs = 2,
        /// Moving sidewalk/travelator (`3`)
        MovingSidewalk = 3,
        /// Escalator (`4`)
        Escalator = 4,
        /// Elevator (`5`)
        Elevator = 5,
        /// Fare gate (or payment gate): a pathway that crosses into
        /// an area of the station where proof of payment is required
        /// to cross. Fare gates may separate paid areas of the
        /// station from unpaid ones, or separate different payment
        /// areas within the same station from each other. This
        /// information can be used to avoid routing passengers
        /// through stations using shortcuts that would require
        /// passengers to make unnecessary payments, like directing a
        /// passenger to walk through a subway platform to reach a
        /// busway (`6`)
        FareGate = 6,
        /// Exit gate: a pathway exiting a paid area into an unpaid
        /// area where proof of payment is not required to cross
        /// (`7`)
        ExitGate = 7,
    }
}

/// A pathway edge from `pathways.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Pathway, PathwayMode};
///
/// // one-way escalator from the entrance up to the platform
/// let up = Pathway::new("pw1", "entrance_1", "platform_2", PathwayMode::Escalator, false)
///     .with_traversal_time(45);
/// assert!(!up.is_bidirectional);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Pathway {
    /// Identifies a pathway. Used by systems as an internal
    /// identifier for the record. Must be unique in the dataset.
    ///
    /// Different pathways may have the same values for
    /// `from_stop_id` and `to_stop_id`. For example, when two
    /// escalators are side-by-side in opposite directions, or when a
    /// stair set and elevator go from the same place to the same
    /// place, different `pathway_id` may have the same
    /// `from_stop_id` and `to_stop_id` values.
    ///
    /// Presence: Required.
    pub pathway_id: String,
    /// Location at which the pathway begins. Must contain a
    /// `stop_id` that identifies a platform (`location_type=0` or
    /// empty), entrance/exit (`location_type=2`), generic node
    /// (`location_type=3`) or boarding area (`location_type=4`).
    /// Values for `stop_id` that identify stations
    /// (`location_type=1`), or stops (`location_type=0` or empty)
    /// with `stop_access=1`, are forbidden. Foreign ID referencing
    /// `stops.stop_id`.
    ///
    /// Presence: Required.
    pub from_stop_id: String,
    /// Location at which the pathway ends. Must contain a `stop_id`
    /// that identifies a platform (`location_type=0` or empty),
    /// entrance/exit (`location_type=2`), generic node
    /// (`location_type=3`) or boarding area (`location_type=4`).
    /// Values for `stop_id` that identify stations
    /// (`location_type=1`), or stops (`location_type=0` or empty)
    /// with `stop_access=1`, are forbidden. Foreign ID referencing
    /// `stops.stop_id`.
    ///
    /// Presence: Required.
    pub to_stop_id: String,
    /// Type of pathway between the specified (`from_stop_id`,
    /// `to_stop_id`) pair. See [`PathwayMode`] for the possible
    /// values and their semantics.
    ///
    /// Presence: Required.
    pub pathway_mode: PathwayMode,
    /// Indicates the direction that the pathway can be taken:
    /// `false` (`0` in the file) for a unidirectional pathway that
    /// can only be used from `from_stop_id` to `to_stop_id`, `true`
    /// (`1` in the file) for a bidirectional pathway that can be
    /// used in both directions.
    ///
    /// Exit gates (`pathway_mode=7`) must not be bidirectional.
    ///
    /// Presence: Required. Stored as a `bool`.
    pub is_bidirectional: bool,
    /// Horizontal length in meters of the pathway from the origin
    /// location (defined in `from_stop_id`) to the destination
    /// location (defined in `to_stop_id`).
    ///
    /// This field is recommended for walkways (`pathway_mode=1`),
    /// fare gates (`pathway_mode=6`) and exit gates
    /// (`pathway_mode=7`).
    ///
    /// Presence: Optional. Non-negative float; `None` when the value
    /// is empty in the file.
    pub length: Option<f64>,
    /// Average time in seconds needed to walk through the pathway
    /// from the origin location (defined in `from_stop_id`) to the
    /// destination location (defined in `to_stop_id`).
    ///
    /// This field is recommended for moving sidewalks
    /// (`pathway_mode=3`), escalators (`pathway_mode=4`) and
    /// elevators (`pathway_mode=5`).
    ///
    /// Presence: Optional. Positive integer; `None` when the value
    /// is empty in the file.
    pub traversal_time: Option<u32>,
    /// Number of stairs of the pathway.
    ///
    /// A positive `stair_count` implies that the rider walks up from
    /// `from_stop_id` to `to_stop_id`, and a negative `stair_count`
    /// implies that the rider walks down from `from_stop_id` to
    /// `to_stop_id`.
    ///
    /// This field is recommended for stairs (`pathway_mode=2`). If
    /// only an estimated stair count can be provided, it is
    /// recommended to approximate 15 stairs for 1 floor.
    ///
    /// Presence: Optional. Non-null integer; `None` when the value
    /// is empty in the file.
    pub stair_count: Option<i32>,
    /// Maximum slope ratio of the pathway: `0` or empty means no
    /// slope; otherwise the slope ratio of the pathway, positive for
    /// upwards, negative for downwards.
    ///
    /// This field should only be used with walkways
    /// (`pathway_mode=1`) and moving sidewalks (`pathway_mode=3`).
    ///
    /// Example: in the US, 0.083 (also written 8.3%) is the maximum
    /// slope ratio for a hand-propelled wheelchair, which means an
    /// increase of 0.083m (so 8.3cm) for each 1m.
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub max_slope: Option<f64>,
    /// Minimum width of the pathway in meters.
    ///
    /// This field is recommended if the minimum width is less than
    /// 1 meter.
    ///
    /// Presence: Optional. Positive float; `None` when the value is
    /// empty in the file.
    pub min_width: Option<f64>,
    /// Public facing text from physical signage that is visible to
    /// riders.
    ///
    /// May be used to provide text directions to riders, such as
    /// "follow signs to". The text in `signposted_as` should appear
    /// exactly how it is printed on the signs.
    ///
    /// When the physical signage is multilingual, this field may be
    /// populated and translated following the example of
    /// `stops.stop_name` in the field definition of
    /// `feed_info.feed_lang`.
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub signposted_as: Option<String>,
    /// Same as `signposted_as`, but when the pathway is used from
    /// the `to_stop_id` to the `from_stop_id`.
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub reversed_signposted_as: Option<String>,
}

impl Pathway {
    /// Creates a pathway edge from the required fields.
    ///
    /// # Arguments
    ///
    /// * `pathway_id` - Unique pathway identifier
    /// * `from_stop_id` - Location the pathway begins at
    /// * `to_stop_id` - Location the pathway ends at
    /// * `pathway_mode` - Type of the pathway
    /// * `is_bidirectional` - Traversable in both directions
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Pathway, PathwayMode};
    ///
    /// let stairs =
    ///     Pathway::new("pw1", "from", "to", PathwayMode::Stairs, true);
    /// assert_eq!(stairs.pathway_id, "pw1");
    /// assert!(stairs.is_bidirectional);
    /// ```
    pub fn new(
        pathway_id: &str,
        from_stop_id: &str,
        to_stop_id: &str,
        pathway_mode: PathwayMode,
        is_bidirectional: bool,
    ) -> Self {
        Pathway {
            pathway_id: pathway_id.to_string(),
            from_stop_id: from_stop_id.to_string(),
            to_stop_id: to_stop_id.to_string(),
            pathway_mode,
            is_bidirectional,
            length: None,
            traversal_time: None,
            stair_count: None,
            max_slope: None,
            min_width: None,
            signposted_as: None,
            reversed_signposted_as: None,
        }
    }

    /// Sets the average traversal time in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Pathway, PathwayMode};
    ///
    /// let stairs =
    ///     Pathway::new("pw1", "from", "to", PathwayMode::Stairs, true)
    ///         .with_traversal_time(45);
    /// assert_eq!(stairs.traversal_time, Some(45));
    /// ```
    pub fn with_traversal_time(mut self, traversal_time: u32) -> Self {
        self.traversal_time = Some(traversal_time);
        self
    }

    /// Sets the horizontal length in meters.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Pathway, PathwayMode};
    ///
    /// let stairs =
    ///     Pathway::new("pw1", "from", "to", PathwayMode::Stairs, true)
    ///         .with_length(12.0);
    /// assert_eq!(stairs.length, Some(12.0));
    /// ```
    pub fn with_length(mut self, length: f64) -> Self {
        self.length = Some(length);
        self
    }
}
