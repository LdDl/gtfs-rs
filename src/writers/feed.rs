//! Whole-feed writing: serializes a
//! [`GtfsReference`](crate::GtfsReference) into a dataset directory,
//! orchestrating the per-format writers.

use std::fs;
use std::path::Path;
use std::slice;

use crate::reference::GtfsReference;
use crate::writers::csv::{CsvWrite, write_path};
use crate::writers::{WriteError, WriteErrorKind, geojson};

/// Writes a whole [`GtfsReference`] into an unpacked dataset
/// directory, creating it if needed.
///
/// The five required tables (`agency.txt`, `stops.txt`,
/// `routes.txt`, `trips.txt`, `stop_times.txt`) are always written,
/// header included even when empty; every other table is written
/// only when it has records. `feed_info.txt` is written when
/// present, `locations.geojson` when there are zones. Existing files
/// are overwritten; files of tables that became empty are NOT
/// removed - write into a fresh directory for a clean feed.
///
/// # Arguments
///
/// * `gtfs` - The dataset to write
/// * `dir` - Destination directory
///
/// # Errors
///
/// Returns a [`WriteError`] if the directory cannot be created or
/// any file fails to write.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::GtfsReference;
/// use gtfs_rs::writers::{self, WriteError};
///
/// fn main() -> Result<(), WriteError> {
///     let gtfs = GtfsReference::new();
///     writers::write_dir(&gtfs, "out_feed/")?;
///     Ok(())
/// }
/// ```
pub fn write_dir(gtfs: &GtfsReference, dir: impl AsRef<Path>) -> Result<(), WriteError> {
    let dir = dir.as_ref();
    if let Err(e) = fs::create_dir_all(dir) {
        return Err(WriteError {
            file: dir.display().to_string(),
            kind: WriteErrorKind::Io(e),
        });
    }

    // required tables - always written, header included when empty
    required(&gtfs.agencies, dir)?;
    required(&gtfs.stops, dir)?;
    required(&gtfs.routes, dir)?;
    required(&gtfs.trips, dir)?;
    required(&gtfs.stop_times, dir)?;
    // conditionally required and optional tables - written when
    // non-empty
    optional(&gtfs.calendar, dir)?;
    optional(&gtfs.calendar_dates, dir)?;
    optional(&gtfs.fare_attributes, dir)?;
    optional(&gtfs.fare_rules, dir)?;
    optional(&gtfs.timeframes, dir)?;
    optional(&gtfs.rider_categories, dir)?;
    optional(&gtfs.fare_media, dir)?;
    optional(&gtfs.fare_products, dir)?;
    optional(&gtfs.fare_leg_rules, dir)?;
    optional(&gtfs.fare_leg_join_rules, dir)?;
    optional(&gtfs.fare_transfer_rules, dir)?;
    optional(&gtfs.areas, dir)?;
    optional(&gtfs.stop_areas, dir)?;
    optional(&gtfs.networks, dir)?;
    optional(&gtfs.route_networks, dir)?;
    optional(&gtfs.shapes, dir)?;
    optional(&gtfs.frequencies, dir)?;
    optional(&gtfs.transfers, dir)?;
    optional(&gtfs.pathways, dir)?;
    optional(&gtfs.levels, dir)?;
    optional(&gtfs.location_groups, dir)?;
    optional(&gtfs.location_group_stops, dir)?;
    optional(&gtfs.booking_rules, dir)?;
    optional(&gtfs.translations, dir)?;
    if let Some(feed_info) = &gtfs.feed_info {
        write_path(slice::from_ref(feed_info), dir.join("feed_info.txt"))?;
    }
    optional(&gtfs.attributions, dir)?;
    if !gtfs.locations.is_empty() {
        geojson::write_locations_path(&gtfs.locations, dir.join("locations.geojson"))?;
    }
    Ok(())
}

/// Writes a required table, header included even when empty.
fn required<T: CsvWrite>(rows: &[T], dir: &Path) -> Result<(), WriteError> {
    write_path(rows, dir.join(T::FILE_NAME))
}

/// Writes an optional table only when it has records.
fn optional<T: CsvWrite>(rows: &[T], dir: &Path) -> Result<(), WriteError> {
    if rows.is_empty() {
        return Ok(());
    }
    write_path(rows, dir.join(T::FILE_NAME))
}

#[cfg(all(test, feature = "parse"))]
mod roundtrip_tests {
    use super::*;
    use crate::parsers::csv::test_support::FEED_DIR;
    #[cfg(feature = "geojson")]
    use crate::parsers::csv::test_support::FLEX_DIR;
    use crate::parsers::read_dir;
    use std::env;
    use std::process;

    /// A process-unique scratch directory under the system temp dir.
    fn scratch_dir(tag: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!("gtfs_rs_{}_{}", tag, process::id()))
    }

    #[test]
    fn test_sample_feed_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original = read_dir(FEED_DIR)?;
        let dir = scratch_dir("roundtrip");
        write_dir(&original, &dir)?;
        let reread = read_dir(&dir);
        fs::remove_dir_all(&dir)?;
        let reread = reread?;

        assert_eq!(reread.agencies.len(), original.agencies.len());
        assert_eq!(reread.stops.len(), original.stops.len());
        assert_eq!(reread.routes.len(), original.routes.len());
        assert_eq!(reread.trips.len(), original.trips.len());
        assert_eq!(reread.stop_times.len(), original.stop_times.len());
        assert_eq!(reread.calendar.len(), original.calendar.len());
        assert_eq!(reread.fare_attributes.len(), original.fare_attributes.len());
        assert_eq!(reread.frequencies.len(), original.frequencies.len());
        // spot-check values across the roundtrip
        assert_eq!(
            reread.agencies[0].agency_name,
            original.agencies[0].agency_name
        );
        assert_eq!(reread.stops[0].stop_lat, original.stops[0].stop_lat);
        assert_eq!(
            reread.stop_times[0].arrival_time,
            original.stop_times[0].arrival_time
        );
        assert_eq!(
            reread.fare_attributes[0].price,
            original.fare_attributes[0].price
        );
        // the rewritten feed must also validate cleanly
        assert!(reread.validate().is_valid());
        Ok(())
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_flex_feed_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original = read_dir(FLEX_DIR)?;
        let dir = scratch_dir("flex_roundtrip");
        write_dir(&original, &dir)?;
        let reread = read_dir(&dir);
        fs::remove_dir_all(&dir)?;
        let reread = reread?;

        assert_eq!(reread.locations.len(), original.locations.len());
        assert_eq!(reread.booking_rules.len(), original.booking_rules.len());
        assert_eq!(
            reread.stop_times[0].start_pickup_drop_off_window,
            original.stop_times[0].start_pickup_drop_off_window
        );
        assert!(reread.validate().is_valid());
        Ok(())
    }
}
