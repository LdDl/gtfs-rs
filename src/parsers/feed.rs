//! Whole-feed reading: orchestrates the per-format parsers over a
//! feed container - an unpacked directory here, a zip archive in
//! `parsers::zip`. The shared [`TableSource`] abstraction keeps the
//! list of tables in one place.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttributeV1,
    FareLegJoinRule, FareLegRule, FareMedia, FareProduct, FareRuleV1, FareTransferRule, FeedInfo,
    Frequency, Level, LocationGroup, LocationGroupStop, Network, Pathway, RiderCategory, Route,
    RouteNetwork, ShapePoint, Stop, StopArea, StopTime, Timeframe, Transfer, Translation, Trip,
};
use crate::parsers::csv::{self, CsvRecord};
use crate::parsers::{ParseError, ParseErrorKind};
use crate::reference::GtfsReference;

/// A container of GTFS tables addressed by canonical file name.
///
/// Implemented by the unpacked-directory source here and by the zip
/// source in `parsers::zip`, so [`read_tables`] can fill a
/// [`GtfsReference`] from either without duplicating the table list.
pub trait TableSource {
    /// Opens one table by its canonical file name; `Ok(None)` when
    /// the container has no such file.
    fn open(&mut self, name: &str) -> Result<Option<Box<dyn io::Read + '_>>, ParseError>;

    /// Reads the whole `locations.geojson` text; `Ok(None)` when the
    /// container has no such file.
    #[cfg(feature = "geojson")]
    fn locations_text(&mut self) -> Result<Option<String>, ParseError>;
}

/// Fills a [`GtfsReference`] from any [`TableSource`].
///
/// The five required tables must be present; every other table is
/// read when present and left empty otherwise. `feed_info.txt`
/// contributes at most one record. With the `geojson` cargo feature
/// enabled, a present `locations.geojson` is read as well.
pub fn read_tables<S: TableSource>(source: &mut S) -> Result<GtfsReference, ParseError> {
    let mut gtfs = GtfsReference::new();
    // required tables
    gtfs.agencies = required::<Agency, _>(source)?;
    gtfs.stops = required::<Stop, _>(source)?;
    gtfs.routes = required::<Route, _>(source)?;
    gtfs.trips = required::<Trip, _>(source)?;
    gtfs.stop_times = required::<StopTime, _>(source)?;
    // conditionally required and optional tables
    gtfs.calendar = optional::<Calendar, _>(source)?;
    gtfs.calendar_dates = optional::<CalendarDate, _>(source)?;
    gtfs.fare_attributes = optional::<FareAttributeV1, _>(source)?;
    gtfs.fare_rules = optional::<FareRuleV1, _>(source)?;
    gtfs.timeframes = optional::<Timeframe, _>(source)?;
    gtfs.rider_categories = optional::<RiderCategory, _>(source)?;
    gtfs.fare_media = optional::<FareMedia, _>(source)?;
    gtfs.fare_products = optional::<FareProduct, _>(source)?;
    gtfs.fare_leg_rules = optional::<FareLegRule, _>(source)?;
    gtfs.fare_leg_join_rules = optional::<FareLegJoinRule, _>(source)?;
    gtfs.fare_transfer_rules = optional::<FareTransferRule, _>(source)?;
    gtfs.areas = optional::<Area, _>(source)?;
    gtfs.stop_areas = optional::<StopArea, _>(source)?;
    gtfs.networks = optional::<Network, _>(source)?;
    gtfs.route_networks = optional::<RouteNetwork, _>(source)?;
    gtfs.shapes = optional::<ShapePoint, _>(source)?;
    gtfs.frequencies = optional::<Frequency, _>(source)?;
    gtfs.transfers = optional::<Transfer, _>(source)?;
    gtfs.pathways = optional::<Pathway, _>(source)?;
    gtfs.levels = optional::<Level, _>(source)?;
    gtfs.location_groups = optional::<LocationGroup, _>(source)?;
    gtfs.location_group_stops = optional::<LocationGroupStop, _>(source)?;
    gtfs.booking_rules = optional::<BookingRule, _>(source)?;
    gtfs.translations = optional::<Translation, _>(source)?;
    gtfs.feed_info = optional::<FeedInfo, _>(source)?.into_iter().next();
    gtfs.attributions = optional::<Attribution, _>(source)?;
    #[cfg(feature = "geojson")]
    if let Some(text) = source.locations_text()? {
        gtfs.locations = crate::parsers::geojson::read_locations_str("locations.geojson", &text)?;
    }
    Ok(gtfs)
}

/// Reads a required table; its absence is an error.
fn required<T: CsvRecord, S: TableSource>(source: &mut S) -> Result<Vec<T>, ParseError> {
    match source.open(T::FILE_NAME)? {
        Some(reader) => csv::read(T::FILE_NAME, reader),
        None => Err(ParseError {
            file: T::FILE_NAME.to_string(),
            line: 0,
            field: None,
            kind: ParseErrorKind::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "required table is missing from the feed",
            )),
        }),
    }
}

/// Reads an optional table; its absence yields an empty list.
fn optional<T: CsvRecord, S: TableSource>(source: &mut S) -> Result<Vec<T>, ParseError> {
    match source.open(T::FILE_NAME)? {
        Some(reader) => csv::read(T::FILE_NAME, reader),
        None => Ok(Vec::new()),
    }
}

/// An unpacked feed directory as a [`TableSource`].
struct DirSource {
    dir: PathBuf,
}

impl TableSource for DirSource {
    fn open(&mut self, name: &str) -> Result<Option<Box<dyn io::Read + '_>>, ParseError> {
        let path = self.dir.join(name);
        if !path.exists() {
            return Ok(None);
        }
        match File::open(&path) {
            Ok(file) => Ok(Some(Box::new(file))),
            Err(e) => Err(ParseError {
                file: name.to_string(),
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            }),
        }
    }

    #[cfg(feature = "geojson")]
    fn locations_text(&mut self) -> Result<Option<String>, ParseError> {
        let path = self.dir.join("locations.geojson");
        if !path.exists() {
            return Ok(None);
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(Some(text)),
            Err(e) => Err(ParseError {
                file: "locations.geojson".to_string(),
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            }),
        }
    }
}

/// Reads a whole unpacked GTFS dataset directory into a
/// [`GtfsReference`].
///
/// The five required tables (`agency.txt`, `stops.txt`,
/// `routes.txt`, `trips.txt`, `stop_times.txt`) must exist; every
/// other table is read when its file is present and left empty
/// otherwise. `feed_info.txt` contributes at most one record. With
/// the `geojson` cargo feature enabled, a present
/// `locations.geojson` is read as well; without the feature the file
/// is ignored. For feeds distributed as archives see
/// `parsers::zip::read_zip` (the `zip` cargo feature).
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
    let mut source = DirSource {
        dir: dir.as_ref().to_path_buf(),
    };
    read_tables(&mut source)
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

    #[test]
    fn test_read_dir_missing_required_table() {
        // the flex fixture's parent directory is not a feed
        let Err(err) = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data")) else {
            panic!("expected a missing-required-table error");
        };
        assert_eq!(err.file, "agency.txt");
        assert!(matches!(err.kind, ParseErrorKind::Io(_)));
    }
}
