//! `trips.txt` - trips: sequences of two or more stops occurring
//! during a specific time period.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#tripstxt>

gtfs_enum! {
    /// Indicates wheelchair accessibility of a trip
    /// (`wheelchair_accessible`).
    #[derive(Default)]
    WheelchairAccessible {
        /// No accessibility information for the trip (`0` or empty,
        /// default)
        #[default]
        NoInformation = 0,
        /// Vehicle being used on this particular trip can
        /// accommodate at least one rider in a wheelchair (`1`)
        Accessible = 1,
        /// No riders in wheelchairs can be accommodated on this
        /// trip (`2`)
        NotAccessible = 2,
    }
}

gtfs_enum! {
    /// Indicates whether bikes are allowed on a trip
    /// (`bikes_allowed`).
    #[derive(Default)]
    BikesAllowed {
        /// No bike information for the trip (`0` or empty, default)
        #[default]
        NoInformation = 0,
        /// Vehicle being used on this particular trip can
        /// accommodate at least one bicycle (`1`)
        Allowed = 1,
        /// No bicycles are allowed on this trip (`2`)
        NotAllowed = 2,
    }
}

gtfs_enum! {
    /// Indicates whether cars are allowed on a trip
    /// (`cars_allowed`).
    #[derive(Default)]
    CarsAllowed {
        /// No car information for the trip (`0` or empty, default)
        #[default]
        NoInformation = 0,
        /// Vehicle being used on this particular trip can
        /// accommodate at least one car (`1`)
        Allowed = 1,
        /// No cars are allowed on this trip (`2`)
        NotAllowed = 2,
    }
}

gtfs_enum! {
    /// Direction of travel for a trip (`direction_id`).
    Direction {
        /// Travel in one direction, e.g. outbound travel (`0`)
        Outbound = 0,
        /// Travel in the opposite direction, e.g. inbound travel
        /// (`1`)
        Inbound = 1,
    }
}

/// A trip from `trips.txt`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Direction, Trip};
///
/// let trip = Trip::new("t0", "L1", "weekday")
///     .with_direction(Direction::Outbound)
///     .with_headsign("Airport");
/// assert_eq!(trip.direction_id, Some(Direction::Outbound));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Trip {
    /// Identifies a route.
    ///
    /// Required. Foreign ID referencing `routes.route_id`.
    pub route_id: String,
    /// Identifies a set of dates when service is available for one
    /// or more routes.
    ///
    /// Required. Foreign ID referencing `calendar.service_id` or
    /// `calendar_dates.service_id`.
    pub service_id: String,
    /// Identifies a trip.
    ///
    /// Required. Unique ID.
    pub trip_id: String,
    /// Text that appears on signage identifying the trip's
    /// destination to riders. This field is recommended for all
    /// services with headsign text displayed on the vehicle which
    /// may be used to distinguish amongst trips in a route.
    ///
    /// If the headsign changes during a trip, values for
    /// `trip_headsign` may be overridden by defining values in
    /// `stop_times.stop_headsign` for specific stop times along the
    /// trip.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub trip_headsign: Option<String>,
    /// Public facing text used to identify the trip to riders, for
    /// instance, to identify train numbers for commuter rail trips.
    /// If riders do not commonly rely on trip names,
    /// `trip_short_name` should be empty. A `trip_short_name`
    /// value, if provided, should uniquely identify a trip within a
    /// service day; it should not be used for destination names or
    /// limited/express designations.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub trip_short_name: Option<String>,
    /// Indicates the direction of travel for a trip. This field
    /// should not be used in routing; it provides a way to separate
    /// trips by direction when publishing time tables. See
    /// [`Direction`] for the valid options.
    ///
    /// The `trip_headsign` and `direction_id` fields may be used
    /// together to assign a name to travel in each direction for a
    /// set of trips, e.g. headsign "Airport" with
    /// [`Direction::Outbound`] and headsign "Downtown" with
    /// [`Direction::Inbound`].
    ///
    /// Optional. `None` means the value is empty in the file.
    pub direction_id: Option<Direction>,
    /// Identifies the block to which the trip belongs. A block
    /// consists of a single trip or many sequential trips made
    /// using the same vehicle, defined by shared service days and
    /// `block_id`. A `block_id` may have trips with different
    /// service days, making distinct blocks. To provide in-seat
    /// transfers information, transfers of `transfer_type` `4`
    /// should be provided instead.
    ///
    /// Optional. `None` means the value is empty in the file.
    pub block_id: Option<String>,
    /// Identifies a geospatial shape describing the vehicle travel
    /// path for a trip.
    ///
    /// Foreign ID referencing `shapes.shape_id`. Conditionally
    /// Required: required if the trip has a continuous pickup or
    /// drop-off behavior defined either in `routes.txt` or in
    /// `stop_times.txt`; optional otherwise. `None` means the value
    /// is empty in the file.
    pub shape_id: Option<String>,
    /// Indicates wheelchair accessibility.
    ///
    /// Optional. Represented by [`WheelchairAccessible`]; the
    /// default (`0` or empty, no accessibility information) is
    /// encoded via `Default`.
    pub wheelchair_accessible: WheelchairAccessible,
    /// Indicates whether bikes are allowed.
    ///
    /// Optional. Represented by [`BikesAllowed`]; the default (`0`
    /// or empty, no bike information) is encoded via `Default`.
    pub bikes_allowed: BikesAllowed,
    /// Indicates whether cars are allowed.
    ///
    /// Optional. Represented by [`CarsAllowed`]; the default (`0`
    /// or empty, no car information) is encoded via `Default`.
    pub cars_allowed: CarsAllowed,
    /// Multiplier applied to travel time estimates calculated for
    /// on-demand trips (GTFS-Flex). See the "Calculating on-demand
    /// trip time estimates with safe duration fields" section of
    /// the spec for guidance on how to use this and the
    /// `safe_duration_offset` fields.
    ///
    /// Optional. Float. `None` means the value is empty in the
    /// file.
    pub safe_duration_factor: Option<f64>,
    /// Fixed offset value in seconds applied to travel time
    /// estimates calculated for on-demand trips (GTFS-Flex). See
    /// the "Calculating on-demand trip time estimates with safe
    /// duration fields" section of the spec for guidance on how to
    /// use this and the `safe_duration_factor` fields.
    ///
    /// Optional. Float. `None` means the value is empty in the
    /// file.
    pub safe_duration_offset: Option<f64>,
}

impl Trip {
    /// Creates a trip from the required fields.
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Unique trip identifier
    /// * `route_id` - Parent route identifier
    /// * `service_id` - Service calendar identifier
    pub fn new(trip_id: &str, route_id: &str, service_id: &str) -> Self {
        Trip {
            route_id: route_id.to_string(),
            service_id: service_id.to_string(),
            trip_id: trip_id.to_string(),
            trip_headsign: None,
            trip_short_name: None,
            direction_id: None,
            block_id: None,
            shape_id: None,
            wheelchair_accessible: WheelchairAccessible::default(),
            bikes_allowed: BikesAllowed::default(),
            cars_allowed: CarsAllowed::default(),
            safe_duration_factor: None,
            safe_duration_offset: None,
        }
    }

    /// Sets the direction of travel.
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction_id = Some(direction);
        self
    }

    /// Sets the destination sign text.
    pub fn with_headsign(mut self, trip_headsign: &str) -> Self {
        self.trip_headsign = Some(trip_headsign.to_string());
        self
    }

    /// Sets the rider-facing short name.
    pub fn with_short_name(mut self, trip_short_name: &str) -> Self {
        self.trip_short_name = Some(trip_short_name.to_string());
        self
    }

    /// Sets the block of sequential trips.
    pub fn with_block_id(mut self, block_id: &str) -> Self {
        self.block_id = Some(block_id.to_string());
        self
    }

    /// Sets the geospatial shape.
    pub fn with_shape_id(mut self, shape_id: &str) -> Self {
        self.shape_id = Some(shape_id.to_string());
        self
    }
}
