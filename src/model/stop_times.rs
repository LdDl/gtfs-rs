//! `stop_times.txt` - times a vehicle arrives at and departs from
//! stops for each trip, including GTFS-Flex on-demand windows.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#stop_timestxt>

use super::routes::ContinuousPickupDropOff;

gtfs_enum! {
    /// Pickup or drop-off method at a stop (`pickup_type` and
    /// `drop_off_type` in `stop_times.txt`). The default
    /// ([`PickupDropOffType::Regular`], corresponding to an empty
    /// file value) is encoded via [`Default`].
    #[derive(Default)]
    PickupDropOffType {
        /// Regularly scheduled pickup or drop-off. Forbidden for
        /// both `pickup_type` and `drop_off_type` when
        /// `start_pickup_drop_off_window` or
        /// `end_pickup_drop_off_window` are defined (`0` or empty,
        /// default)
        #[default]
        Regular = 0,
        /// No pickup or drop-off available at this stop (`1`)
        NotAvailable = 1,
        /// Must phone the agency to arrange pickup or drop-off
        /// (`2`)
        PhoneAgency = 2,
        /// Must coordinate with the driver to arrange pickup or
        /// drop-off. Forbidden as `pickup_type` when
        /// `start_pickup_drop_off_window` or
        /// `end_pickup_drop_off_window` are defined (`3`)
        CoordinateWithDriver = 3,
    }
}

gtfs_enum! {
    /// Indicates if arrival and departure times for a stop are
    /// strictly adhered to by the vehicle or if they are instead
    /// approximate and/or interpolated times (`timepoint` in
    /// `stop_times.txt`). This field allows a GTFS producer to
    /// provide interpolated stop times, while indicating that the
    /// times are approximate. All records of `stop_times.txt` with
    /// defined arrival or departure times should have `timepoint`
    /// values populated; if no `timepoint` values are provided, all
    /// times are considered exact. The default
    /// ([`Timepoint::Exact`]) is encoded via [`Default`].
    #[derive(Default)]
    Timepoint {
        /// Times are considered approximate (`0`)
        Approximate = 0,
        /// Times are considered exact. Also assumed when the file
        /// value is empty (`1`, default)
        #[default]
        Exact = 1,
    }
}

/// A stop time from `stop_times.txt`.
///
/// For fixed-route service set `stop_id`; for GTFS-Flex on-demand
/// service set `location_group_id` or `location_id` together with the
/// pickup/drop-off window instead.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::{StopTime, parse_gtfs_time};
///
///     // arrival 08:00:00, departure after a 30-second dwell
///     let st = StopTime::new("t0", "A", 1, parse_gtfs_time("08:00:00")?)
///         .with_times(8 * 3600, 8 * 3600 + 30);
///     assert_eq!(st.departure_time, Some(28830));
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct StopTime {
    /// Identifies a trip (foreign ID referencing `trips.trip_id`).
    /// Required.
    pub trip_id: String,
    /// Arrival time at the stop (defined by `stop_id`) for a
    /// specific trip (defined by `trip_id`), in the time zone
    /// specified by `agency.agency_timezone`, not
    /// `stops.stop_timezone`. Stored as `u32` seconds since midnight
    /// of the service day (file format `HH:MM:SS`); for times
    /// occurring after midnight on the service day the file value is
    /// greater than `24:00:00`, so hours may exceed 23. If there are
    /// not separate times for arrival and departure at a stop,
    /// `arrival_time` and `departure_time` should be the same. If
    /// exact arrival and departure times (`timepoint = 1`) are not
    /// available, estimated or interpolated arrival and departure
    /// times (`timepoint = 0`) should be provided. `None` means the
    /// value is empty in the file.
    ///
    /// Conditionally Required:
    /// - Required for the first and last stop in a trip (defined by
    ///   `stop_sequence`).
    /// - Required for `timepoint = 1` ([`Timepoint::Exact`]).
    /// - Forbidden when `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub arrival_time: Option<u32>,
    /// Departure time from the stop (defined by `stop_id`) for a
    /// specific trip (defined by `trip_id`), in the time zone
    /// specified by `agency.agency_timezone`, not
    /// `stops.stop_timezone`. Stored as `u32` seconds since midnight
    /// of the service day (file format `HH:MM:SS`); for times
    /// occurring after midnight on the service day the file value is
    /// greater than `24:00:00`, so hours may exceed 23. If there are
    /// not separate times for arrival and departure at a stop,
    /// `arrival_time` and `departure_time` should be the same. If
    /// exact times are not available, estimated or interpolated
    /// times should be provided. `None` means the value is empty in
    /// the file.
    ///
    /// Conditionally Required:
    /// - Required for `timepoint = 1` ([`Timepoint::Exact`]).
    /// - Forbidden when `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub departure_time: Option<u32>,
    /// Identifies the serviced stop (foreign ID referencing
    /// `stops.stop_id`). All stops serviced during a trip must have
    /// a record in `stop_times.txt`. Referenced locations must be
    /// stops/platforms, i.e. their `stops.location_type` value must
    /// be `0` or empty. A stop may be serviced multiple times in the
    /// same trip, and multiple trips and routes may service the same
    /// stop.
    ///
    /// On-demand service using stops should be referenced in the
    /// sequence in which service is available at those stops. A data
    /// consumer should assume that travel is possible from one stop
    /// or location to another stop or location in the sequence,
    /// provided that the `pickup_type`/`drop_off_type` of each stop
    /// time and the time constraints of each
    /// `start_pickup_drop_off_window`/`end_pickup_drop_off_window`
    /// do not forbid it.
    ///
    /// Conditionally Required:
    /// - Required if `location_group_id` AND `location_id` are NOT
    ///   defined.
    /// - Forbidden if `location_group_id` or `location_id` are
    ///   defined.
    pub stop_id: Option<String>,
    /// Identifies the serviced location group that indicates groups
    /// of stops where riders may request pickup or drop off (foreign
    /// ID referencing `location_groups.location_group_id`,
    /// GTFS-Flex). All location groups serviced during a trip must
    /// have a record in `stop_times.txt`. Multiple trips and routes
    /// may service the same location group.
    ///
    /// On-demand service using location groups should be referenced
    /// in the sequence in which service is available at those
    /// location groups. A data consumer should assume that travel is
    /// possible from one stop or location to another stop or
    /// location in the sequence, provided that the
    /// `pickup_type`/`drop_off_type` of each stop time and the time
    /// constraints of each pickup/drop-off window do not forbid it.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden if `stop_id` or `location_id` are defined.
    /// - Optional otherwise.
    pub location_group_id: Option<String>,
    /// Identifies the GeoJSON location that corresponds to a
    /// serviced zone where riders may request pickup or drop off
    /// (foreign ID referencing `id` from `locations.geojson`,
    /// GTFS-Flex). All GeoJSON locations serviced during a trip must
    /// have a record in `stop_times.txt`. Multiple trips and routes
    /// may service the same GeoJSON location.
    ///
    /// On-demand service within locations should be referenced in
    /// the sequence in which service is available in those
    /// locations. A data consumer should assume that travel is
    /// possible from one stop or location to another stop or
    /// location in the sequence, provided that the
    /// `pickup_type`/`drop_off_type` of each stop time and the time
    /// constraints of each pickup/drop-off window do not forbid it.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden if `stop_id` or `location_group_id` are defined.
    /// - Optional otherwise.
    pub location_id: Option<String>,
    /// Order of stops, location groups, or GeoJSON locations for a
    /// particular trip. The values must increase along the trip but
    /// do not need to be consecutive. Required.
    ///
    /// Example: the first location on the trip could have
    /// `stop_sequence = 1`, the second location on the trip could
    /// have `stop_sequence = 23`, the third location could have
    /// `stop_sequence = 40`, and so on.
    ///
    /// Travel within the same location group or GeoJSON location
    /// requires two records in `stop_times.txt` with the same
    /// `location_group_id` or `location_id`.
    pub stop_sequence: u32,
    /// Text that appears on signage identifying the trip's
    /// destination to riders. This field overrides the default
    /// `trips.trip_headsign` when the headsign changes between
    /// stops. If the headsign is displayed for an entire trip,
    /// `trips.trip_headsign` should be used instead. Optional
    /// (`None` when empty in the file).
    ///
    /// A `stop_headsign` value specified for one stop time does not
    /// apply to subsequent stop times in the same trip. To override
    /// the trip headsign for multiple stop times in the same trip,
    /// the `stop_headsign` value must be repeated in each stop time
    /// row.
    pub stop_headsign: Option<String>,
    /// Time that on-demand service becomes available in a GeoJSON
    /// location, location group, or stop (GTFS-Flex). Stored as
    /// `u32` seconds since midnight of the service day (file format
    /// `HH:MM:SS`, hours may exceed 23). `None` means the value is
    /// empty in the file.
    ///
    /// Conditionally Required:
    /// - Required if `location_group_id` or `location_id` is
    ///   defined.
    /// - Required if `end_pickup_drop_off_window` is defined.
    /// - Forbidden if `arrival_time` or `departure_time` is
    ///   defined.
    /// - Optional otherwise.
    pub start_pickup_drop_off_window: Option<u32>,
    /// Time that on-demand service ends in a GeoJSON location,
    /// location group, or stop (GTFS-Flex). Stored as `u32` seconds
    /// since midnight of the service day (file format `HH:MM:SS`,
    /// hours may exceed 23). `None` means the value is empty in the
    /// file.
    ///
    /// Conditionally Required:
    /// - Required if `location_group_id` or `location_id` is
    ///   defined.
    /// - Required if `start_pickup_drop_off_window` is defined.
    /// - Forbidden if `arrival_time` or `departure_time` is
    ///   defined.
    /// - Optional otherwise.
    pub end_pickup_drop_off_window: Option<u32>,
    /// Indicates the pickup method at this stop; see
    /// [`PickupDropOffType`] for the values. The default
    /// ([`PickupDropOffType::Regular`], corresponding to an empty
    /// file value) is encoded via [`Default`].
    ///
    /// Conditionally Forbidden:
    /// - `pickup_type = 0` ([`PickupDropOffType::Regular`]) is
    ///   forbidden if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - `pickup_type = 3`
    ///   ([`PickupDropOffType::CoordinateWithDriver`]) is forbidden
    ///   if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub pickup_type: PickupDropOffType,
    /// Indicates the drop-off method at this stop; see
    /// [`PickupDropOffType`] for the values. The default
    /// ([`PickupDropOffType::Regular`], corresponding to an empty
    /// file value) is encoded via [`Default`].
    ///
    /// Conditionally Forbidden:
    /// - `drop_off_type = 0` ([`PickupDropOffType::Regular`]) is
    ///   forbidden if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub drop_off_type: PickupDropOffType,
    /// Indicates that the rider can board the transit vehicle at any
    /// point along the vehicle's travel path as described by
    /// `shapes.txt`, from this stop time to the next stop time in
    /// the trip's `stop_sequence`; see [`ContinuousPickupDropOff`]
    /// for the values. If this field is populated, it overrides any
    /// continuous pickup behavior defined in `routes.txt`; if it is
    /// empty, the stop time inherits any continuous pickup behavior
    /// defined in `routes.txt`. The default
    /// ([`ContinuousPickupDropOff::NotAvailable`]) is encoded via
    /// [`Default`].
    ///
    /// Conditionally Forbidden:
    /// - Any value other than `1`
    ///   ([`ContinuousPickupDropOff::NotAvailable`]) or empty is
    ///   forbidden if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub continuous_pickup: ContinuousPickupDropOff,
    /// Indicates that the rider can alight from the transit vehicle
    /// at any point along the vehicle's travel path as described by
    /// `shapes.txt`, from this stop time to the next stop time in
    /// the trip's `stop_sequence`; see [`ContinuousPickupDropOff`]
    /// for the values. If this field is populated, it overrides any
    /// continuous drop-off behavior defined in `routes.txt`; if it
    /// is empty, the stop time inherits any continuous drop-off
    /// behavior defined in `routes.txt`. The default
    /// ([`ContinuousPickupDropOff::NotAvailable`]) is encoded via
    /// [`Default`].
    ///
    /// Conditionally Forbidden:
    /// - Any value other than `1`
    ///   ([`ContinuousPickupDropOff::NotAvailable`]) or empty is
    ///   forbidden if `start_pickup_drop_off_window` or
    ///   `end_pickup_drop_off_window` are defined.
    /// - Optional otherwise.
    pub continuous_drop_off: ContinuousPickupDropOff,
    /// Actual distance traveled along the associated shape, from the
    /// first stop to the stop specified in this record. This field
    /// specifies how much of the shape to draw between any two stops
    /// during a trip. Must be in the same units used in
    /// `shapes.txt`. Values used for `shape_dist_traveled` must
    /// increase along with `stop_sequence`; they must not be used to
    /// show reverse travel along a route. Recommended for routes
    /// that have looping or inlining (the vehicle crosses or travels
    /// over the same portion of alignment in one trip). Optional
    /// (`None` when empty in the file).
    ///
    /// Example: if a bus travels a distance of 5.25 kilometers from
    /// the start of the shape to the stop,
    /// `shape_dist_traveled = 5.25`.
    pub shape_dist_traveled: Option<f64>,
    /// Indicates if arrival and departure times for this stop are
    /// strictly adhered to by the vehicle or if they are instead
    /// approximate and/or interpolated times; see [`Timepoint`] for
    /// the values. Optional. All records of `stop_times.txt` with
    /// defined arrival or departure times should have `timepoint`
    /// values populated; if no `timepoint` values are provided, all
    /// times are considered exact. The default
    /// ([`Timepoint::Exact`], corresponding to an empty file value)
    /// is encoded via [`Default`].
    pub timepoint: Timepoint,
    /// Identifies the boarding booking rule at this stop time
    /// (foreign ID referencing `booking_rules.booking_rule_id`,
    /// GTFS-Flex). Recommended when `pickup_type = 2`
    /// ([`PickupDropOffType::PhoneAgency`]). Optional (`None` when
    /// empty in the file).
    pub pickup_booking_rule_id: Option<String>,
    /// Identifies the alighting booking rule at this stop time
    /// (foreign ID referencing `booking_rules.booking_rule_id`,
    /// GTFS-Flex). Recommended when `drop_off_type = 2`
    /// ([`PickupDropOffType::PhoneAgency`]). Optional (`None` when
    /// empty in the file).
    pub drop_off_booking_rule_id: Option<String>,
}

impl StopTime {
    /// Creates a fixed-route stop time with equal arrival and
    /// departure (no dwell).
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Parent trip identifier
    /// * `stop_id` - Visited stop identifier
    /// * `stop_sequence` - Order of the stop within the trip
    /// * `time` - Arrival = departure time, seconds since midnight
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::StopTime;
    ///
    /// let st = StopTime::new("t0", "A", 1, 8 * 3600);
    /// assert_eq!(st.arrival_time, Some(28800));
    /// assert_eq!(st.departure_time, Some(28800));
    /// assert_eq!(st.stop_id.as_deref(), Some("A"));
    /// ```
    pub fn new(trip_id: &str, stop_id: &str, stop_sequence: u32, time: u32) -> Self {
        StopTime {
            trip_id: trip_id.to_string(),
            arrival_time: Some(time),
            departure_time: Some(time),
            stop_id: Some(stop_id.to_string()),
            location_group_id: None,
            location_id: None,
            stop_sequence,
            stop_headsign: None,
            start_pickup_drop_off_window: None,
            end_pickup_drop_off_window: None,
            pickup_type: PickupDropOffType::default(),
            drop_off_type: PickupDropOffType::default(),
            continuous_pickup: ContinuousPickupDropOff::default(),
            continuous_drop_off: ContinuousPickupDropOff::default(),
            shape_dist_traveled: None,
            timepoint: Timepoint::default(),
            pickup_booking_rule_id: None,
            drop_off_booking_rule_id: None,
        }
    }

    /// Sets distinct arrival and departure times (dwell at the stop).
    ///
    /// # Arguments
    ///
    /// * `arrival` - Arrival time, seconds since midnight
    /// * `departure` - Departure time, seconds since midnight
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::StopTime;
    ///
    /// let st = StopTime::new("t0", "A", 1, 8 * 3600)
    ///     .with_times(8 * 3600, 8 * 3600 + 30);
    /// assert_eq!(st.arrival_time, Some(28800));
    /// assert_eq!(st.departure_time, Some(28830));
    /// ```
    pub fn with_times(mut self, arrival: u32, departure: u32) -> Self {
        self.arrival_time = Some(arrival);
        self.departure_time = Some(departure);
        self
    }

    /// Sets the destination sign text from this stop onwards.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::StopTime;
    ///
    /// let st = StopTime::new("t0", "A", 1, 8 * 3600)
    ///     .with_headsign("Downtown");
    /// assert_eq!(st.stop_headsign.as_deref(), Some("Downtown"));
    /// ```
    pub fn with_headsign(mut self, stop_headsign: &str) -> Self {
        self.stop_headsign = Some(stop_headsign.to_string());
        self
    }

    /// Sets the pickup and drop-off methods.
    ///
    /// # Arguments
    ///
    /// * `pickup_type` - Pickup method at this stop
    /// * `drop_off_type` - Drop-off method at this stop
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{PickupDropOffType, StopTime};
    ///
    /// let st = StopTime::new("t0", "A", 1, 8 * 3600)
    ///     .with_pickup_drop_off(
    ///         PickupDropOffType::PhoneAgency,
    ///         PickupDropOffType::NotAvailable,
    ///     );
    /// assert_eq!(st.pickup_type, PickupDropOffType::PhoneAgency);
    /// assert_eq!(st.drop_off_type, PickupDropOffType::NotAvailable);
    /// ```
    pub fn with_pickup_drop_off(
        mut self,
        pickup_type: PickupDropOffType,
        drop_off_type: PickupDropOffType,
    ) -> Self {
        self.pickup_type = pickup_type;
        self.drop_off_type = drop_off_type;
        self
    }

    /// Sets the on-demand pickup/drop-off window (GTFS-Flex).
    ///
    /// # Arguments
    ///
    /// * `start` - Window start, seconds since midnight
    /// * `end` - Window end, seconds since midnight
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::StopTime;
    ///
    /// let st = StopTime::new("t0", "A", 1, 8 * 3600)
    ///     .with_pickup_drop_off_window(9 * 3600, 17 * 3600);
    /// assert_eq!(st.start_pickup_drop_off_window, Some(32400));
    /// assert_eq!(st.end_pickup_drop_off_window, Some(61200));
    /// ```
    pub fn with_pickup_drop_off_window(mut self, start: u32, end: u32) -> Self {
        self.start_pickup_drop_off_window = Some(start);
        self.end_pickup_drop_off_window = Some(end);
        self
    }
}
