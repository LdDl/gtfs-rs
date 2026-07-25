//! # GTFS Schedule Reference Container
//!
//! In-memory container for a complete GTFS Schedule dataset with
//! lookup helpers. One collection per dataset file, in specification
//! order. Populated programmatically; file parsing is out of scope.
//! "Reference" follows the GTFS Schedule Reference naming: the static
//! dataset, as opposed to GTFS Realtime feeds.

use crate::misc::GtfsDate;
use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, ExceptionType, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, Location, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory,
    Route, RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation,
    Trip,
};

/// An in-memory GTFS Schedule dataset.
///
/// # Examples
///
/// ```
/// use gtfs_rs::{Frequency, GtfsReference, Route, RouteType, Stop, StopTime, Trip};
///
/// let mut gtfs = GtfsReference::new();
/// gtfs.stops.push(Stop::new("A"));
/// gtfs.stops.push(Stop::new("B"));
/// gtfs.routes.push(Route::new("L1", RouteType::Bus));
/// gtfs.trips.push(Trip::new("L1_t0", "L1", "daily"));
/// gtfs.stop_times.push(StopTime::new("L1_t0", "A", 0, 8 * 3600));
/// gtfs.stop_times.push(StopTime::new("L1_t0", "B", 1, 8 * 3600 + 600));
/// gtfs.frequencies.push(Frequency::new("L1_t0", 7 * 3600, 10 * 3600, 300));
///
/// let pattern = gtfs.stop_times_of_trip("L1_t0");
/// assert_eq!(pattern.len(), 2);
/// assert_eq!(pattern[0].stop_id.as_deref(), Some("A"));
/// ```
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GtfsReference {
    /// Agencies (`agency.txt`)
    pub agencies: Vec<Agency>,
    /// Stops, stations and other locations (`stops.txt`)
    pub stops: Vec<Stop>,
    /// Routes (`routes.txt`)
    pub routes: Vec<Route>,
    /// Trips (`trips.txt`)
    pub trips: Vec<Trip>,
    /// Stop times (`stop_times.txt`)
    pub stop_times: Vec<StopTime>,
    /// Weekly service patterns (`calendar.txt`)
    pub calendar: Vec<Calendar>,
    /// Service exceptions (`calendar_dates.txt`)
    pub calendar_dates: Vec<CalendarDate>,
    /// Fare classes (`fare_attributes.txt`, GTFS-Fares v1)
    pub fare_attributes: Vec<FareAttributeV1>,
    /// Fare applicability rules (`fare_rules.txt`, GTFS-Fares v1)
    pub fare_rules: Vec<FareRuleV1>,
    /// Fare timeframes (`timeframes.txt`, GTFS-Fares v2)
    pub timeframes: Vec<Timeframe>,
    /// Rider categories (`rider_categories.txt`, GTFS-Fares v2)
    pub rider_categories: Vec<RiderCategory>,
    /// Fare media (`fare_media.txt`, GTFS-Fares v2)
    pub fare_media: Vec<FareMedia>,
    /// Fare products (`fare_products.txt`, GTFS-Fares v2)
    pub fare_products: Vec<FareProduct>,
    /// Fare leg rules (`fare_leg_rules.txt`, GTFS-Fares v2)
    pub fare_leg_rules: Vec<FareLegRule>,
    /// Fare leg join rules (`fare_leg_join_rules.txt`, GTFS-Fares v2)
    pub fare_leg_join_rules: Vec<FareLegJoinRule>,
    /// Fare transfer rules (`fare_transfer_rules.txt`, GTFS-Fares v2)
    pub fare_transfer_rules: Vec<FareTransferRule>,
    /// Fare areas (`areas.txt`, GTFS-Fares v2)
    pub areas: Vec<Area>,
    /// Stop-to-area assignments (`stop_areas.txt`, GTFS-Fares v2)
    pub stop_areas: Vec<StopArea>,
    /// Route networks (`networks.txt`, GTFS-Fares v2)
    pub networks: Vec<Network>,
    /// Route-to-network assignments (`route_networks.txt`, GTFS-Fares v2)
    pub route_networks: Vec<RouteNetwork>,
    /// Shape points (`shapes.txt`)
    pub shapes: Vec<ShapePoint>,
    /// Headway-based service windows (`frequencies.txt`)
    pub frequencies: Vec<Frequency>,
    /// Transfer rules (`transfers.txt`)
    pub transfers: Vec<Transfer>,
    /// Station pathways (`pathways.txt`)
    pub pathways: Vec<Pathway>,
    /// Station levels (`levels.txt`)
    pub levels: Vec<Level>,
    /// Location groups (`location_groups.txt`, GTFS-Flex)
    pub location_groups: Vec<LocationGroup>,
    /// Stop-to-location-group assignments
    /// (`location_group_stops.txt`, GTFS-Flex)
    pub location_group_stops: Vec<LocationGroupStop>,
    /// GeoJSON zones (`locations.geojson`, GTFS-Flex)
    pub locations: Vec<Location>,
    /// Booking rules (`booking_rules.txt`, GTFS-Flex)
    pub booking_rules: Vec<BookingRule>,
    /// Translations (`translations.txt`)
    pub translations: Vec<Translation>,
    /// Dataset metadata (`feed_info.txt`, single record)
    pub feed_info: Option<FeedInfo>,
    /// Attributions (`attributions.txt`)
    pub attributions: Vec<Attribution>,
}

impl GtfsReference {
    /// Creates an empty dataset.
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::GtfsReference;
    ///
    /// let gtfs = GtfsReference::new();
    /// assert!(gtfs.stops.is_empty());
    /// assert!(gtfs.feed_info.is_none());
    /// ```
    pub fn new() -> Self {
        GtfsReference::default()
    }

    /// Returns an agency by its identifier.
    ///
    /// # Arguments
    ///
    /// * `agency_id` - Agency identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Agency, GtfsReference};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.agencies.push(
    ///     Agency::new("City", "https://x.example", "Europe/Moscow")
    ///         .with_id("CT"),
    /// );
    /// assert!(gtfs.agency("CT").is_some());
    /// assert!(gtfs.agency("ZZ").is_none());
    /// ```
    pub fn agency(&self, agency_id: &str) -> Option<&Agency> {
        self.agencies
            .iter()
            .find(|a| a.agency_id.as_deref() == Some(agency_id))
    }

    /// Returns a stop by its identifier.
    ///
    /// # Arguments
    ///
    /// * `stop_id` - Stop identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Stop};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.stops.push(Stop::new("A"));
    /// assert!(gtfs.stop("A").is_some());
    /// assert!(gtfs.stop("B").is_none());
    /// ```
    pub fn stop(&self, stop_id: &str) -> Option<&Stop> {
        self.stops.iter().find(|s| s.stop_id == stop_id)
    }

    /// Returns a route by its identifier.
    ///
    /// # Arguments
    ///
    /// * `route_id` - Route identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Route, RouteType};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.routes.push(Route::new("L1", RouteType::Bus));
    /// assert!(gtfs.route("L1").is_some());
    /// assert!(gtfs.route("L2").is_none());
    /// ```
    pub fn route(&self, route_id: &str) -> Option<&Route> {
        self.routes.iter().find(|r| r.route_id == route_id)
    }

    /// Returns a trip by its identifier.
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Trip identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Trip};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.trips.push(Trip::new("t0", "L1", "daily"));
    /// assert!(gtfs.trip("t0").is_some());
    /// assert!(gtfs.trip("t1").is_none());
    /// ```
    pub fn trip(&self, trip_id: &str) -> Option<&Trip> {
        self.trips.iter().find(|t| t.trip_id == trip_id)
    }

    /// Returns the stop times of a trip ordered by `stop_sequence`.
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Trip identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, StopTime};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.stop_times.push(StopTime::new("t0", "B", 2, 600));
    /// gtfs.stop_times.push(StopTime::new("t0", "A", 1, 0));
    ///
    /// let pattern = gtfs.stop_times_of_trip("t0");
    /// assert_eq!(pattern[0].stop_id.as_deref(), Some("A"));
    /// ```
    pub fn stop_times_of_trip(&self, trip_id: &str) -> Vec<&StopTime> {
        let mut times: Vec<&StopTime> = self
            .stop_times
            .iter()
            .filter(|st| st.trip_id == trip_id)
            .collect();
        times.sort_by_key(|st| st.stop_sequence);
        times
    }

    /// Returns the frequency windows of a trip in dataset order.
    ///
    /// # Arguments
    ///
    /// * `trip_id` - Trip identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{Frequency, GtfsReference};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.frequencies.push(Frequency::new("t0", 0, 3600, 300));
    /// assert_eq!(gtfs.frequencies_of_trip("t0").len(), 1);
    /// assert!(gtfs.frequencies_of_trip("t1").is_empty());
    /// ```
    pub fn frequencies_of_trip(&self, trip_id: &str) -> Vec<&Frequency> {
        self.frequencies
            .iter()
            .filter(|f| f.trip_id == trip_id)
            .collect()
    }

    /// Returns the trips of a route in dataset order.
    ///
    /// # Arguments
    ///
    /// * `route_id` - Route identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, Trip};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.trips.push(Trip::new("t0", "L1", "daily"));
    /// gtfs.trips.push(Trip::new("t1", "L1", "daily"));
    /// assert_eq!(gtfs.trips_of_route("L1").len(), 2);
    /// ```
    pub fn trips_of_route(&self, route_id: &str) -> Vec<&Trip> {
        self.trips
            .iter()
            .filter(|t| t.route_id == route_id)
            .collect()
    }

    /// Returns the points of a shape ordered by `shape_pt_sequence`.
    ///
    /// # Arguments
    ///
    /// * `shape_id` - Shape identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use gtfs_rs::{GtfsReference, ShapePoint};
    ///
    /// let mut gtfs = GtfsReference::new();
    /// gtfs.shapes.push(ShapePoint::new("sh", 55.0, 37.1, 2));
    /// gtfs.shapes.push(ShapePoint::new("sh", 55.0, 37.0, 1));
    /// assert_eq!(gtfs.shape("sh")[0].shape_pt_lon, 37.0);
    /// ```
    pub fn shape(&self, shape_id: &str) -> Vec<&ShapePoint> {
        let mut points: Vec<&ShapePoint> = self
            .shapes
            .iter()
            .filter(|p| p.shape_id == shape_id)
            .collect();
        points.sort_by_key(|p| p.shape_pt_sequence);
        points
    }

    /// Returns whether a service runs on a date, combining the weekly
    /// pattern from `calendar.txt` with exceptions from
    /// `calendar_dates.txt`.
    ///
    /// # Arguments
    ///
    /// * `service_id` - Service identifier
    /// * `date` - Date to check
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::{Calendar, GtfsDate, GtfsReference};
    ///
    ///     let mut gtfs = GtfsReference::new();
    ///     let start = GtfsDate::new(2026, 7, 1)?;
    ///     let end = GtfsDate::new(2026, 7, 31)?;
    ///     let cal = Calendar::new("wd", start, end);
    ///     gtfs.calendar.push(cal.with_weekdays());
    ///
    ///     let fri = GtfsDate::new(2026, 7, 24)?;
    ///     let sun = GtfsDate::new(2026, 7, 26)?;
    ///     assert!(gtfs.is_service_active("wd", &fri));
    ///     assert!(!gtfs.is_service_active("wd", &sun));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_service_active(&self, service_id: &str, date: &GtfsDate) -> bool {
        if let Some(exception) = self
            .calendar_dates
            .iter()
            .find(|cd| cd.service_id == service_id && cd.date == *date)
        {
            return exception.exception_type == ExceptionType::Added;
        }
        self.calendar
            .iter()
            .find(|c| c.service_id == service_id)
            .is_some_and(|c| c.is_active_on(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GtfsError;
    use crate::model::{Direction, RouteType};

    #[test]
    fn test_lookups() {
        let mut gtfs = GtfsReference::new();
        gtfs.routes.push(Route::new("L1", RouteType::Bus));
        gtfs.trips
            .push(Trip::new("t0", "L1", "daily").with_direction(Direction::Outbound));
        gtfs.trips
            .push(Trip::new("t1", "L1", "daily").with_direction(Direction::Inbound));
        // out-of-order sequences must be sorted by stop_sequence
        gtfs.stop_times.push(StopTime::new("t0", "B", 5, 100));
        gtfs.stop_times.push(StopTime::new("t0", "A", 1, 0));
        gtfs.frequencies.push(Frequency::new("t0", 0, 3600, 300));
        gtfs.shapes.push(ShapePoint::new("sh", 55.0, 37.1, 2));
        gtfs.shapes.push(ShapePoint::new("sh", 55.0, 37.0, 1));

        assert_eq!(gtfs.trips_of_route("L1").len(), 2);
        let pattern = gtfs.stop_times_of_trip("t0");
        assert_eq!(pattern[0].stop_id.as_deref(), Some("A"));
        assert_eq!(pattern[1].stop_id.as_deref(), Some("B"));
        assert_eq!(gtfs.frequencies_of_trip("t0").len(), 1);
        assert!(gtfs.frequencies_of_trip("t1").is_empty());
        assert_eq!(gtfs.route("L1").map(|r| r.route_type), Some(RouteType::Bus));
        assert_eq!(gtfs.shape("sh")[0].shape_pt_lon, 37.0);
    }

    #[test]
    fn test_service_activity_with_exceptions() -> Result<(), GtfsError> {
        let mut gtfs = GtfsReference::new();
        gtfs.calendar.push(
            Calendar::new(
                "wd",
                GtfsDate::new(2026, 7, 1)?,
                GtfsDate::new(2026, 7, 31)?,
            )
            .with_weekdays(),
        );
        // Friday 2026-07-24 removed, Sunday 2026-07-26 added
        gtfs.calendar_dates.push(CalendarDate::new(
            "wd",
            GtfsDate::new(2026, 7, 24)?,
            ExceptionType::Removed,
        ));
        gtfs.calendar_dates.push(CalendarDate::new(
            "wd",
            GtfsDate::new(2026, 7, 26)?,
            ExceptionType::Added,
        ));

        assert!(gtfs.is_service_active("wd", &GtfsDate::new(2026, 7, 23)?));
        assert!(!gtfs.is_service_active("wd", &GtfsDate::new(2026, 7, 24)?));
        assert!(gtfs.is_service_active("wd", &GtfsDate::new(2026, 7, 26)?));
        assert!(!gtfs.is_service_active("unknown", &GtfsDate::new(2026, 7, 23)?));
        Ok(())
    }
}
