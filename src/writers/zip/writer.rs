//! Writing functions for zipped GTFS feeds.

use std::fs::File;
use std::io::{Cursor, Seek, Write};
use std::path::Path;
use std::slice;

use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::reference::GtfsReference;
use crate::writers::csv::{self, CsvWrite};
use crate::writers::{WriteError, WriteErrorKind, geojson};

/// Packs a whole [`GtfsReference`] into a zip archive at a file
/// path, creating or overwriting the file.
///
/// The table selection mirrors
/// [`write_dir`](crate::writers::write_dir): the five required
/// tables are always included, other tables only when non-empty,
/// and `locations.geojson` when there are zones. Entries are
/// deflate-compressed.
///
/// # Arguments
///
/// * `gtfs` - The dataset to pack
/// * `path` - Destination `.zip` path
///
/// # Errors
///
/// Returns a [`WriteError`] if the file cannot be created or the
/// archive cannot be assembled.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::GtfsReference;
/// use gtfs_rs::writers::{WriteError, zip};
///
/// fn main() -> Result<(), WriteError> {
///     let gtfs = GtfsReference::new();
///     zip::write_zip(&gtfs, "feed.zip")?;
///     Ok(())
/// }
/// ```
pub fn write_zip(gtfs: &GtfsReference, path: impl AsRef<Path>) -> Result<(), WriteError> {
    let path = path.as_ref();
    let label = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.display().to_string(),
    };
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(WriteError {
                file: label,
                kind: WriteErrorKind::Io(e),
            });
        }
    };
    write_archive(&label, gtfs, file)
}

/// Packs a whole [`GtfsReference`] into an in-memory zip archive -
/// e.g. to upload it without touching the disk.
///
/// # Arguments
///
/// * `gtfs` - The dataset to pack
///
/// # Errors
///
/// Returns a [`WriteError`] if the archive cannot be assembled.
///
/// # Examples
///
/// ```
/// use gtfs_rs::GtfsReference;
/// use gtfs_rs::writers::{WriteError, zip};
///
/// fn main() -> Result<(), WriteError> {
///     let gtfs = GtfsReference::new();
///     let bytes = zip::write_zip_bytes(&gtfs)?;
///     assert!(!bytes.is_empty());
///     Ok(())
/// }
/// ```
pub fn write_zip_bytes(gtfs: &GtfsReference) -> Result<Vec<u8>, WriteError> {
    let mut cursor = Cursor::new(Vec::new());
    write_archive("feed.zip", gtfs, &mut cursor)?;
    Ok(cursor.into_inner())
}

/// Assembles the archive into any write-and-seek target.
fn write_archive<W: Write + Seek>(
    label: &str,
    gtfs: &GtfsReference,
    out: W,
) -> Result<(), WriteError> {
    let mut writer = ZipWriter::new(out);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // required tables - always included, header only when empty
    required(&mut writer, options, label, &gtfs.agencies)?;
    required(&mut writer, options, label, &gtfs.stops)?;
    required(&mut writer, options, label, &gtfs.routes)?;
    required(&mut writer, options, label, &gtfs.trips)?;
    required(&mut writer, options, label, &gtfs.stop_times)?;
    // conditionally required and optional tables
    optional(&mut writer, options, label, &gtfs.calendar)?;
    optional(&mut writer, options, label, &gtfs.calendar_dates)?;
    optional(&mut writer, options, label, &gtfs.fare_attributes)?;
    optional(&mut writer, options, label, &gtfs.fare_rules)?;
    optional(&mut writer, options, label, &gtfs.timeframes)?;
    optional(&mut writer, options, label, &gtfs.rider_categories)?;
    optional(&mut writer, options, label, &gtfs.fare_media)?;
    optional(&mut writer, options, label, &gtfs.fare_products)?;
    optional(&mut writer, options, label, &gtfs.fare_leg_rules)?;
    optional(&mut writer, options, label, &gtfs.fare_leg_join_rules)?;
    optional(&mut writer, options, label, &gtfs.fare_transfer_rules)?;
    optional(&mut writer, options, label, &gtfs.areas)?;
    optional(&mut writer, options, label, &gtfs.stop_areas)?;
    optional(&mut writer, options, label, &gtfs.networks)?;
    optional(&mut writer, options, label, &gtfs.route_networks)?;
    optional(&mut writer, options, label, &gtfs.shapes)?;
    optional(&mut writer, options, label, &gtfs.frequencies)?;
    optional(&mut writer, options, label, &gtfs.transfers)?;
    optional(&mut writer, options, label, &gtfs.pathways)?;
    optional(&mut writer, options, label, &gtfs.levels)?;
    optional(&mut writer, options, label, &gtfs.location_groups)?;
    optional(&mut writer, options, label, &gtfs.location_group_stops)?;
    optional(&mut writer, options, label, &gtfs.booking_rules)?;
    optional(&mut writer, options, label, &gtfs.translations)?;
    if let Some(feed_info) = &gtfs.feed_info {
        entry(&mut writer, options, label, "feed_info.txt")?;
        csv::write("feed_info.txt", slice::from_ref(feed_info), &mut writer)?;
    }
    optional(&mut writer, options, label, &gtfs.attributions)?;
    if !gtfs.locations.is_empty() {
        entry(&mut writer, options, label, "locations.geojson")?;
        geojson::write_locations("locations.geojson", &gtfs.locations, &mut writer)?;
    }

    if let Err(e) = writer.finish() {
        return Err(zip_err(label, e));
    }
    Ok(())
}

/// Adds a required table entry, header included when empty.
fn required<T: CsvWrite, W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    label: &str,
    rows: &[T],
) -> Result<(), WriteError> {
    entry(writer, options, label, T::FILE_NAME)?;
    csv::write(T::FILE_NAME, rows, writer)
}

/// Adds an optional table entry only when it has records.
fn optional<T: CsvWrite, W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    label: &str,
    rows: &[T],
) -> Result<(), WriteError> {
    if rows.is_empty() {
        return Ok(());
    }
    required(writer, options, label, rows)
}

/// Starts a new archive entry.
fn entry<W: Write + Seek>(
    writer: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    label: &str,
    name: &str,
) -> Result<(), WriteError> {
    match writer.start_file(name, options) {
        Ok(()) => Ok(()),
        Err(e) => Err(zip_err(label, e)),
    }
}

/// Builds an archive-level error.
fn zip_err(label: &str, e: zip::result::ZipError) -> WriteError {
    WriteError {
        file: label.to_string(),
        kind: WriteErrorKind::Zip(e),
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::parsers::csv::test_support::FEED_DIR;
    use crate::parsers::read_dir;
    use crate::parsers::zip::read_zip_bytes;

    #[test]
    fn test_zip_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let original = read_dir(FEED_DIR)?;
        let bytes = write_zip_bytes(&original)?;
        let reread = read_zip_bytes("roundtrip.zip", &bytes)?;

        assert_eq!(reread.stops.len(), original.stops.len());
        assert_eq!(reread.trips.len(), original.trips.len());
        assert_eq!(reread.stop_times.len(), original.stop_times.len());
        assert_eq!(
            reread.stop_times[0].departure_time,
            original.stop_times[0].departure_time
        );
        assert!(reread.validate().is_valid());
        Ok(())
    }
}
