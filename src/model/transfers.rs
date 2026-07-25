//! `transfers.txt` - rules for making connections between routes,
//! trips and stops.
//!
//! Reference: <https://gtfs.org/documentation/schedule/reference/#transferstxt>

gtfs_enum! {
    /// Type of connection between two stops, routes or trips
    /// (`transfer_type`).
    #[derive(Default)]
    TransferType {
        /// Recommended transfer point between routes. Written as `0`
        /// or empty in the file (`0`, default)
        #[default]
        Recommended = 0,
        /// Timed transfer point between two routes. The departing
        /// vehicle is expected to wait for the arriving one and
        /// leave sufficient time for a rider to transfer between
        /// routes (`1`)
        Timed = 1,
        /// Transfer requires a minimum amount of time between
        /// arrival and departure to ensure a connection. The time
        /// required to transfer is specified by `min_transfer_time`
        /// (`2`)
        MinimumTime = 2,
        /// Transfers are not possible between routes at the location
        /// (`3`)
        NotPossible = 3,
        /// Passengers can transfer from one trip to another by
        /// staying onboard the same vehicle (an "in-seat transfer").
        /// The linked trips must be operated by the same vehicle,
        /// and the trip pair `from_trip_id`/`to_trip_id` is
        /// required. If both a linked-trips transfer and a
        /// `trips.block_id` are provided and they produce
        /// conflicting results, the linked-trips transfer is used
        /// (`4`)
        InSeat = 4,
        /// In-seat transfers are not allowed between sequential
        /// trips. The passenger must alight from the vehicle and
        /// re-board. The trip pair `from_trip_id`/`to_trip_id` is
        /// required (`5`)
        InSeatNotAllowed = 5,
    }
}

/// A transfer rule from `transfers.txt`.
///
/// The stop pair is conditionally required for transfer types 0-3;
/// the trip pair is required for in-seat transfer types 4 and 5.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Transfer, TransferType};
///
/// let rule = Transfer::new(TransferType::MinimumTime)
///     .between_stops("A", "B")
///     .with_min_transfer_time(180);
/// assert_eq!(rule.min_transfer_time, Some(180));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Transfer {
    /// Identifies a stop (`location_type=0`) or a station
    /// (`location_type=1`) where a connection between routes begins.
    /// If this field refers to a station, the transfer rule applies
    /// to all its child stops. It must refer to a stop
    /// (`location_type=0`) if `transfer_type` is `4` or `5`. Foreign
    /// ID referencing `stops.stop_id`.
    ///
    /// Presence: Conditionally Required. Required if `transfer_type`
    /// is empty, `0`, `1`, `2`, or `3`; optional if `transfer_type`
    /// is `4` or `5`. `None` when the value is empty in the file.
    pub from_stop_id: Option<String>,
    /// Identifies a stop (`location_type=0`) or a station
    /// (`location_type=1`) where a connection between routes ends.
    /// If this field refers to a station, the transfer rule applies
    /// to all child stops. It must refer to a stop
    /// (`location_type=0`) if `transfer_type` is `4` or `5`. Foreign
    /// ID referencing `stops.stop_id`.
    ///
    /// Presence: Conditionally Required. Required if `transfer_type`
    /// is empty, `0`, `1`, `2`, or `3`; optional if `transfer_type`
    /// is `4` or `5`. `None` when the value is empty in the file.
    pub to_stop_id: Option<String>,
    /// Identifies a route where a connection begins. If
    /// `from_route_id` is defined, the transfer will apply to the
    /// arriving trip on the route for the given `from_stop_id`. If
    /// both `from_trip_id` and `from_route_id` are defined, the
    /// `trip_id` must belong to the `route_id`, and `from_trip_id`
    /// will take precedence. Foreign ID referencing
    /// `routes.route_id`.
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub from_route_id: Option<String>,
    /// Identifies a route where a connection ends. If `to_route_id`
    /// is defined, the transfer will apply to the departing trip on
    /// the route for the given `to_stop_id`. If both `to_trip_id`
    /// and `to_route_id` are defined, the `trip_id` must belong to
    /// the `route_id`, and `to_trip_id` will take precedence.
    /// Foreign ID referencing `routes.route_id`.
    ///
    /// Presence: Optional. `None` when the value is empty in the
    /// file.
    pub to_route_id: Option<String>,
    /// Identifies a trip where a connection between routes begins.
    /// If `from_trip_id` is defined, the transfer will apply to the
    /// arriving trip for the given `from_stop_id`. If both
    /// `from_trip_id` and `from_route_id` are defined, the `trip_id`
    /// must belong to the `route_id`, and `from_trip_id` will take
    /// precedence. Foreign ID referencing `trips.trip_id`.
    ///
    /// Presence: Conditionally Required. Required if `transfer_type`
    /// is `4` or `5` (in-seat transfer types); optional otherwise.
    /// `None` when the value is empty in the file.
    pub from_trip_id: Option<String>,
    /// Identifies a trip where a connection between routes ends. If
    /// `to_trip_id` is defined, the transfer will apply to the
    /// departing trip for the given `to_stop_id`. If both
    /// `to_trip_id` and `to_route_id` are defined, the `trip_id`
    /// must belong to the `route_id`, and `to_trip_id` will take
    /// precedence. Foreign ID referencing `trips.trip_id`.
    ///
    /// Presence: Conditionally Required. Required if `transfer_type`
    /// is `4` or `5` (in-seat transfer types); optional otherwise.
    /// `None` when the value is empty in the file.
    pub to_trip_id: Option<String>,
    /// Indicates the type of connection for the specified
    /// (`from_stop_id`, `to_stop_id`) pair. See [`TransferType`] for
    /// the possible values and their semantics.
    ///
    /// Presence: Required. Defaults to [`TransferType::Recommended`]
    /// via `Default` when the value is empty in the file.
    pub transfer_type: TransferType,
    /// Amount of time, in seconds, that must be available to permit
    /// a transfer between routes at the specified stops. The
    /// `min_transfer_time` should be sufficient to permit a typical
    /// rider to move between the two stops, including buffer time to
    /// allow for schedule variance on each route.
    ///
    /// Presence: Optional. Non-negative integer; `None` when the
    /// value is empty in the file.
    pub min_transfer_time: Option<u32>,
}

impl Transfer {
    /// Creates a transfer rule with no endpoints set.
    ///
    /// # Arguments
    ///
    /// * `transfer_type` - Type of the connection
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Transfer, TransferType};
    ///
    /// let rule = Transfer::new(TransferType::MinimumTime);
    /// assert_eq!(rule.transfer_type, TransferType::MinimumTime);
    /// assert!(rule.from_stop_id.is_none());
    /// ```
    pub fn new(transfer_type: TransferType) -> Self {
        Transfer {
            from_stop_id: None,
            to_stop_id: None,
            from_route_id: None,
            to_route_id: None,
            from_trip_id: None,
            to_trip_id: None,
            transfer_type,
            min_transfer_time: None,
        }
    }

    /// Sets the stop pair the rule applies to.
    ///
    /// # Arguments
    ///
    /// * `from_stop_id` - Stop where the connection begins
    /// * `to_stop_id` - Stop where the connection ends
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Transfer, TransferType};
    ///
    /// let rule = Transfer::new(TransferType::MinimumTime)
    ///     .between_stops("A", "B");
    /// assert_eq!(rule.from_stop_id.as_deref(), Some("A"));
    /// assert_eq!(rule.to_stop_id.as_deref(), Some("B"));
    /// ```
    pub fn between_stops(mut self, from_stop_id: &str, to_stop_id: &str) -> Self {
        self.from_stop_id = Some(from_stop_id.to_string());
        self.to_stop_id = Some(to_stop_id.to_string());
        self
    }

    /// Sets the trip pair the rule applies to (in-seat transfers).
    ///
    /// # Arguments
    ///
    /// * `from_trip_id` - Arriving trip
    /// * `to_trip_id` - Departing trip
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Transfer, TransferType};
    ///
    /// let rule = Transfer::new(TransferType::InSeat)
    ///     .between_trips("t1", "t2");
    /// assert_eq!(rule.from_trip_id.as_deref(), Some("t1"));
    /// assert_eq!(rule.to_trip_id.as_deref(), Some("t2"));
    /// ```
    pub fn between_trips(mut self, from_trip_id: &str, to_trip_id: &str) -> Self {
        self.from_trip_id = Some(from_trip_id.to_string());
        self.to_trip_id = Some(to_trip_id.to_string());
        self
    }

    /// Sets the minimum transfer time in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Transfer, TransferType};
    ///
    /// let rule = Transfer::new(TransferType::MinimumTime)
    ///     .between_stops("A", "B")
    ///     .with_min_transfer_time(180);
    /// assert_eq!(rule.min_transfer_time, Some(180));
    /// ```
    pub fn with_min_transfer_time(mut self, min_transfer_time: u32) -> Self {
        self.min_transfer_time = Some(min_transfer_time);
        self
    }
}
