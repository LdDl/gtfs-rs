//! `frequencies.txt` - headway-based service or compressed
//! fixed-schedule service.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#frequenciestxt>

gtfs_enum! {
    /// Type of service in a frequency window (`exact_times`).
    #[derive(Default)]
    ExactTimes {
        /// Frequency-based trips: service does not follow a fixed
        /// schedule throughout the day. Instead, operators attempt
        /// to strictly maintain predetermined headways for trips,
        /// running them at the interval specified by `headway_secs`.
        /// Written as `0` or empty in the file (`0`, default)
        #[default]
        FrequencyBased = 0,
        /// Schedule-based trips with the exact same headway
        /// throughout the day: a compressed representation of
        /// schedule-based service in which operators try to strictly
        /// adhere to a schedule. These trips are scheduled to depart
        /// every `headway_secs` seconds. In this case the `end_time`
        /// value must be greater than the last desired trip
        /// `start_time` but less than the last desired trip
        /// `start_time` + `headway_secs` (`1`)
        ScheduleBased = 1,
    }
}

/// A frequency entry from `frequencies.txt`.
///
/// Defines headway-based service for a trip within a time window.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{ExactTimes, Frequency};
///
/// // every 5 minutes between 07:00 and 10:00
/// let window = Frequency::new("t0", 7 * 3600, 10 * 3600, 300);
/// assert_eq!(window.exact_times, ExactTimes::FrequencyBased);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Frequency {
    /// Identifies a trip to which the specified headway of service
    /// applies. Foreign ID referencing `trips.trip_id`.
    ///
    /// Presence: Required.
    pub trip_id: String,
    /// Time at which the first vehicle departs from the first stop
    /// of the trip with the specified headway.
    ///
    /// Presence: Required. Stored as `u32` seconds since midnight of
    /// the service day (may exceed 24:00:00 for service continuing
    /// past midnight).
    pub start_time: u32,
    /// Time at which service changes to a different headway (or
    /// ceases) at the first stop in the trip. For schedule-based
    /// windows ([`ExactTimes::ScheduleBased`]) this value must be
    /// greater than the last desired trip `start_time` but less than
    /// the last desired trip `start_time` + `headway_secs`.
    ///
    /// Presence: Required. Stored as `u32` seconds since midnight of
    /// the service day (may exceed 24:00:00 for service continuing
    /// past midnight).
    pub end_time: u32,
    /// Time, in seconds, between departures from the same stop
    /// (headway) for the trip, during the time interval specified by
    /// `start_time` and `end_time`. Multiple headways may be defined
    /// for the same trip, but must not overlap. New headways may
    /// start at the exact time the previous headway ends.
    ///
    /// Presence: Required. Positive integer.
    pub headway_secs: u32,
    /// Indicates the type of service for a trip: frequency-based
    /// service that does not follow a fixed schedule, or a
    /// compressed representation of schedule-based service with the
    /// exact same headway over the time period. See [`ExactTimes`]
    /// for the possible values and their semantics.
    ///
    /// Presence: Optional. Defaults to
    /// [`ExactTimes::FrequencyBased`] via `Default` when the value
    /// is empty in the file.
    pub exact_times: ExactTimes,
}

impl Frequency {
    /// Creates a frequency-based entry.
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Trip the headway applies to
    /// * `start_time` - Window start, seconds since midnight
    /// * `end_time` - Window end, seconds since midnight
    /// * `headway_secs` - Time between departures, seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{ExactTimes, Frequency};
    ///
    /// let window = Frequency::new("t0", 7 * 3600, 10 * 3600, 300);
    /// assert_eq!(window.headway_secs, 300);
    /// assert_eq!(window.exact_times, ExactTimes::FrequencyBased);
    /// ```
    pub fn new(trip_id: &str, start_time: u32, end_time: u32, headway_secs: u32) -> Self {
        Frequency {
            trip_id: trip_id.to_string(),
            start_time,
            end_time,
            headway_secs,
            exact_times: ExactTimes::default(),
        }
    }

    /// Marks the window as schedule-based (exact times).
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{ExactTimes, Frequency};
    ///
    /// let window = Frequency::new("t0", 7 * 3600, 10 * 3600, 300)
    ///     .with_exact_times();
    /// assert_eq!(window.exact_times, ExactTimes::ScheduleBased);
    /// ```
    pub fn with_exact_times(mut self) -> Self {
        self.exact_times = ExactTimes::ScheduleBased;
        self
    }
}
