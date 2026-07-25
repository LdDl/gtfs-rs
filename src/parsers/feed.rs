//! Whole-feed reading: orchestrates the per-format parsers
//! ([`csv`](crate::parsers::csv), and
//! `geojson` when that cargo feature is enabled) over an unpacked
//! dataset directory.

use std::path::Path;

use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route,
    RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};
use crate::parsers::ParseError;
use crate::parsers::csv::{CsvRecord, read_path};
use crate::reference::GtfsReference;

/// Reads a whole unpacked GTFS dataset directory into a
/// [`GtfsReference`].
///
/// The five required tables (`agency.txt`, `stops.txt`,
/// `routes.txt`, `trips.txt`, `stop_times.txt`) must exist; every
/// other table is read when its file is present and left empty
/// otherwise. `feed_info.txt` contributes at most one record. With
/// the `geojson` cargo feature enabled, a present
/// `locations.geojson` is read as well; without the feature the file
/// is ignored.
///
/// # Arguments
///
/// * `dir` - Path to the unpacked feed directory
///
/// # Errors
///
/// Returns a [`ParseError`] if a required table is missing or any
/// present file is rejected.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::parsers::{self, ParseError};
///
/// fn main() -> Result<(), ParseError> {
///     let gtfs = parsers::read_dir("feed/")?;
///     println!("{} stops, {} trips", gtfs.stops.len(), gtfs.trips.len());
///     Ok(())
/// }
/// ```
pub fn read_dir(dir: impl AsRef<Path>) -> Result<GtfsReference, ParseError> {
    let dir = dir.as_ref();
    let mut gtfs = GtfsReference::new();
    // required tables
    gtfs.agencies = read_path(dir.join(Agency::FILE_NAME))?;
    gtfs.stops = read_path(dir.join(Stop::FILE_NAME))?;
    gtfs.routes = read_path(dir.join(Route::FILE_NAME))?;
    gtfs.trips = read_path(dir.join(Trip::FILE_NAME))?;
    gtfs.stop_times = read_path(dir.join(StopTime::FILE_NAME))?;
    // conditionally required and optional tables
    gtfs.calendar = read_if_present::<Calendar>(dir)?;
    gtfs.calendar_dates = read_if_present::<CalendarDate>(dir)?;
    gtfs.fare_attributes = read_if_present::<FareAttributeV1>(dir)?;
    gtfs.fare_rules = read_if_present::<FareRuleV1>(dir)?;
    gtfs.timeframes = read_if_present::<Timeframe>(dir)?;
    gtfs.rider_categories = read_if_present::<RiderCategory>(dir)?;
    gtfs.fare_media = read_if_present::<FareMedia>(dir)?;
    gtfs.fare_products = read_if_present::<FareProduct>(dir)?;
    gtfs.fare_leg_rules = read_if_present::<FareLegRule>(dir)?;
    gtfs.fare_leg_join_rules = read_if_present::<FareLegJoinRule>(dir)?;
    gtfs.fare_transfer_rules = read_if_present::<FareTransferRule>(dir)?;
    gtfs.areas = read_if_present::<Area>(dir)?;
    gtfs.stop_areas = read_if_present::<StopArea>(dir)?;
    gtfs.networks = read_if_present::<Network>(dir)?;
    gtfs.route_networks = read_if_present::<RouteNetwork>(dir)?;
    gtfs.shapes = read_if_present::<ShapePoint>(dir)?;
    gtfs.frequencies = read_if_present::<Frequency>(dir)?;
    gtfs.transfers = read_if_present::<Transfer>(dir)?;
    gtfs.pathways = read_if_present::<Pathway>(dir)?;
    gtfs.levels = read_if_present::<Level>(dir)?;
    gtfs.location_groups = read_if_present::<LocationGroup>(dir)?;
    gtfs.location_group_stops = read_if_present::<LocationGroupStop>(dir)?;
    gtfs.booking_rules = read_if_present::<BookingRule>(dir)?;
    gtfs.translations = read_if_present::<Translation>(dir)?;
    gtfs.feed_info = read_if_present::<FeedInfo>(dir)?.into_iter().next();
    gtfs.attributions = read_if_present::<Attribution>(dir)?;
    #[cfg(feature = "geojson")]
    {
        let locations_path = dir.join("locations.geojson");
        if locations_path.exists() {
            gtfs.locations = crate::parsers::geojson::read_locations(locations_path)?;
        }
    }
    Ok(gtfs)
}

/// Reads an optional table: an absent file yields an empty list.
fn read_if_present<T: CsvRecord>(dir: &Path) -> Result<Vec<T>, ParseError> {
    let path = dir.join(T::FILE_NAME);
    if path.exists() {
        read_path(path)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::csv::test_support::{FEED_DIR, FLEX_DIR, model};

    #[test]
    fn test_read_dir_whole_feed() -> Result<(), ParseError> {
        use crate::misc::GtfsDate;

        let gtfs = read_dir(FEED_DIR)?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.stops.len(), 9);
        assert_eq!(gtfs.routes.len(), 5);
        assert_eq!(gtfs.trips.len(), 11);
        assert_eq!(gtfs.stop_times.len(), 28);
        assert_eq!(gtfs.calendar.len(), 2);
        assert_eq!(gtfs.calendar_dates.len(), 1);
        assert_eq!(gtfs.fare_attributes.len(), 2);
        assert_eq!(gtfs.fare_rules.len(), 4);
        assert_eq!(gtfs.frequencies.len(), 11);
        // absent optional tables stay empty
        assert!(gtfs.pathways.is_empty());
        assert!(gtfs.feed_info.is_none());
        // cross-table lookups work on the parsed dataset
        let pattern = gtfs.stop_times_of_trip("STBA");
        assert_eq!(pattern.len(), 2);
        assert!(gtfs.is_service_active("FULLW", &GtfsDate::new(2007, 1, 5).map_err(model)?));
        assert!(!gtfs.is_service_active("FULLW", &GtfsDate::new(2007, 6, 4).map_err(model)?));
        Ok(())
    }

    #[test]
    fn test_read_dir_flex_feed() -> Result<(), ParseError> {
        let gtfs = read_dir(FLEX_DIR)?;
        assert_eq!(gtfs.booking_rules.len(), 1);
        let flex_stop = &gtfs.stop_times[0];
        assert_eq!(flex_stop.location_id.as_deref(), Some("zone_a"));
        assert_eq!(flex_stop.start_pickup_drop_off_window, Some(8 * 3600));
        // locations.geojson is read only with the geojson feature
        #[cfg(feature = "geojson")]
        {
            assert_eq!(gtfs.locations.len(), 2);
            assert_eq!(gtfs.locations[0].location_id, "zone_a");
        }
        #[cfg(not(feature = "geojson"))]
        assert!(gtfs.locations.is_empty());
        Ok(())
    }
}
