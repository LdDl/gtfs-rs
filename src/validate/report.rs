//! The validation report types: [`ValidationIssue`] with its
//! [`Rule`] and [`Severity`], collected into a
//! [`ValidationReport`].

use std::fmt;
use std::slice;
use std::vec;

/// How serious a validation issue is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The dataset violates a hard specification requirement.
    Error,
    /// The dataset is questionable but not spec-invalid.
    Warning,
}

/// Machine-readable identifier of a validation rule.
///
/// Match on it to filter or group issues without parsing messages;
/// the enum is `#[non_exhaustive]` because rules will be added over
/// time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Rule {
    /// A primary key value appears more than once in its table
    DuplicateId,
    /// A `stop_id`, `location_group_id` or `locations.geojson` id
    /// collides with another id of the shared GTFS-Flex id space
    IdSpaceCollision,
    /// A foreign key references a record that does not exist
    UnknownReference,
    /// A `parent_station` of a stop/platform, entrance or generic
    /// node references a location that is not a station
    /// (`location_type` 1)
    ParentStationNotStation,
    /// A `parent_station` of a boarding area (`location_type` 4)
    /// references a location that is not a stop/platform
    /// (`location_type` 0)
    ParentStationNotPlatform,
    /// A `stop_times.stop_id` references a location that is not a
    /// stop/platform (`location_type` 0)
    StopTimeStopNotPlatform,
    /// Neither `route_short_name` nor `route_long_name` is set
    MissingRouteName,
    /// A stop/platform, station or entrance has no `stop_name`
    MissingStopName,
    /// A stop/platform, station or entrance has no coordinates
    MissingStopCoordinates,
    /// A station (`location_type` 1) has a `parent_station`
    ForbiddenParentStation,
    /// An entrance, generic node or boarding area lacks the
    /// required `parent_station`
    MissingParentStation,
    /// A stop time references none of `stop_id`,
    /// `location_group_id`, `location_id`
    MissingStopTimeLocation,
    /// A stop time references more than one of `stop_id`,
    /// `location_group_id`, `location_id`
    ConflictingStopTimeLocation,
    /// Only one of `start_pickup_drop_off_window` and
    /// `end_pickup_drop_off_window` is set
    IncompleteWindow,
    /// `arrival_time`/`departure_time` set together with a
    /// pickup/drop-off window
    TimesWithPickupWindow,
    /// Only one of `timeframes.start_time` and
    /// `timeframes.end_time` is set
    IncompleteTimeframe,
    /// `calendar.start_date` is after `calendar.end_date`
    InvalidServicePeriod,
    /// `frequencies.start_time` is not before
    /// `frequencies.end_time`
    InvalidFrequencyWindow,
    /// An attribution activates none of the producer, operator and
    /// authority roles
    AttributionRoleMissing,
    /// An attribution targets more than one of `agency_id`,
    /// `route_id`, `trip_id`
    AttributionMultipleTargets,
    /// `agency_id` is required because the dataset has more than
    /// one agency
    MissingAgencyId,
    /// `routes.network_id` is used together with
    /// `route_networks.txt`
    NetworkIdConflict,
    /// `translations.txt` is present but `feed_info.txt` is missing
    MissingFeedInfo,
    /// A trip has fewer than two stop times: none at all, or only a
    /// single one
    TripWithoutStopTimes,
    /// Two `frequencies.txt` windows of the same trip overlap
    OverlappingFrequency,
    /// A stop time referencing `location_group_id` or `location_id`
    /// lacks the required pickup/drop-off window
    MissingPickupWindow,
    /// `arrival_time` is missing on the first or last stop time of
    /// a trip
    MissingFirstLastArrivalTime,
    /// `transfers.from_stop_id` or `to_stop_id` is missing for a
    /// `transfer_type` that requires the stop pair (0-3)
    MissingTransferStop,
    /// `transfers.from_trip_id` or `to_trip_id` is missing for an
    /// in-seat `transfer_type` (4 or 5)
    MissingTransferTrip,
    /// A pathway endpoint (`from_stop_id`/`to_stop_id`) references
    /// a station (`location_type` 1)
    PathwayEndpointIsStation,
    /// An exit gate pathway (`pathway_mode` 7) is bidirectional
    BidirectionalExitGate,
    /// A `booking_rules.txt` field required by the record's
    /// `booking_type` is missing
    MissingBookingField,
    /// A `booking_rules.txt` field forbidden for the record's
    /// `booking_type` is set
    ForbiddenBookingField,
}

/// One problem found by validation, with a full trace to its place:
/// file, record, field, machine-readable rule and human-readable
/// message.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{GtfsReference, Route, RouteType, Rule};
///
/// let mut gtfs = GtfsReference::new();
/// gtfs.routes.push(Route::new("L1", RouteType::Bus)); // no name
///
/// let report = gtfs.validate();
/// let issue = &report.issues()[0];
/// assert_eq!(issue.rule, Rule::MissingRouteName);
/// assert_eq!(issue.file, "routes.txt");
/// assert_eq!(issue.entity_id.as_deref(), Some("L1"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ValidationIssue {
    /// How serious the issue is
    pub severity: Severity,
    /// Dataset file the issue belongs to (e.g. "stop_times.txt")
    pub file: &'static str,
    /// Primary-key value of the offending record, when applicable
    pub entity_id: Option<String>,
    /// Field (column) name, when the issue is field-level
    pub field: Option<String>,
    /// Machine-readable rule identifier
    pub rule: Rule,
    /// Human-readable detail
    pub message: String,
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.file)?;
        if let Some(entity_id) = &self.entity_id {
            write!(f, ", record `{}`", entity_id)?;
        }
        if let Some(field) = &self.field {
            write!(f, ", field `{}`", field)?;
        }
        write!(f, ": {}", self.message)
    }
}

/// The outcome of [`GtfsReference::validate`]: every found issue,
/// errors and warnings alike.
///
/// [`GtfsReference::validate`]: crate::GtfsReference::validate
///
/// # Examples
///
/// ```
/// use gtfs_rs::GtfsReference;
///
/// let report = GtfsReference::new().validate();
/// assert!(report.is_valid()); // an empty dataset breaks no rules
/// ```
#[derive(Debug, Clone)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Wraps the collected issues.
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        ValidationReport { issues }
    }

    /// Returns whether the dataset has no [`Severity::Error`]
    /// issues. Warnings do not make a dataset invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Route, RouteType};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.routes.push(Route::new("L1", RouteType::Bus)); // no name
    /// assert!(!gtfs.validate().is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|issue| issue.severity == Severity::Error)
    }

    /// Returns every found issue in detection order.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::GtfsReference;
    ///
    /// let report = GtfsReference::new().validate();
    /// assert!(report.issues().is_empty());
    /// ```
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Iterates over the [`Severity::Error`] issues.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Route, RouteType};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.routes.push(Route::new("L1", RouteType::Bus)); // no name
    /// for issue in gtfs.validate().errors() {
    ///     println!("error: {issue}");
    /// }
    /// ```
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Error)
    }

    /// Iterates over the [`Severity::Warning`] issues.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Route, RouteType, Trip};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.routes.push(Route::new("L1", RouteType::Bus).with_short_name("1"));
    /// // a trip with no stop times is suspicious but not invalid
    /// gtfs.trips.push(Trip::new("t0", "L1", "daily"));
    /// assert_eq!(gtfs.validate().warnings().count(), 1);
    /// ```
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == Severity::Warning)
    }
}

/// Borrowing iteration over every issue, in detection order:
/// `for issue in &report`.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{GtfsReference, Route, RouteType};
///
/// let mut gtfs = GtfsReference::new();
/// // no name
/// gtfs.routes.push(Route::new("L1", RouteType::Bus));
/// let report = gtfs.validate();
/// for issue in &report {
///     println!("{issue}");
/// }
/// ```
impl<'a> IntoIterator for &'a ValidationReport {
    type Item = &'a ValidationIssue;
    type IntoIter = slice::Iter<'a, ValidationIssue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.iter()
    }
}

/// Consuming iteration over every issue, in detection order - e.g.
/// to collect the issues into another structure.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{GtfsReference, Route, RouteType, ValidationIssue};
///
/// let mut gtfs = GtfsReference::new();
/// // no name
/// gtfs.routes.push(Route::new("L1", RouteType::Bus));
/// let issues: Vec<ValidationIssue> = gtfs.validate().into_iter().collect();
/// assert!(!issues.is_empty());
/// ```
impl IntoIterator for ValidationReport {
    type Item = ValidationIssue;
    type IntoIter = vec::IntoIter<ValidationIssue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errors = self.errors().count();
        let warnings = self.warnings().count();
        write!(f, "{} error(s), {} warning(s)", errors, warnings)
    }
}
