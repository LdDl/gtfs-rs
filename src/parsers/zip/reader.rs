//! Reading functions for zipped GTFS feeds.

use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use zip::ZipArchive;
use zip::result::ZipError;

use crate::parsers::feed::{TableSource, read_tables};
use crate::parsers::{ParseError, ParseErrorKind};
use crate::reference::GtfsReference;

/// Reads a zipped GTFS feed from a file path into a
/// [`GtfsReference`].
///
/// The archive is expected to hold the tables at its root, as the
/// specification requires. The required tables and the handling of
/// optional ones are the same as for
/// [`read_dir`](crate::parsers::read_dir); with the `geojson` cargo
/// feature enabled, a bundled `locations.geojson` is read as well.
///
/// # Arguments
///
/// * `path` - Path to the `.zip` feed archive
///
/// # Errors
///
/// Returns a [`ParseError`] if the archive cannot be opened or is
/// malformed ([`ParseErrorKind::Zip`]), a required table is missing,
/// or any present table is rejected.
///
/// # Examples
///
/// ```no_run
/// use gtfs_rs::parsers::{ParseError, zip};
///
/// fn main() -> Result<(), ParseError> {
///     let gtfs = zip::read_zip("feed.zip")?;
///     println!("{} stops, {} trips", gtfs.stops.len(), gtfs.trips.len());
///     Ok(())
/// }
/// ```
pub fn read_zip(path: impl AsRef<Path>) -> Result<GtfsReference, ParseError> {
    let path = path.as_ref();
    let label = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.display().to_string(),
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(ParseError {
                file: label,
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            });
        }
    };
    let archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => return Err(zip_err(&label, e)),
    };
    let mut source = ZipSource { label, archive };
    read_tables(&mut source)
}

/// Reads a zipped GTFS feed from bytes already in memory - e.g. an
/// archive just downloaded over HTTP, without touching the disk.
///
/// # Arguments
///
/// * `archive_label` - Name used in error messages
///   (e.g. "feed.zip")
/// * `bytes` - The raw archive bytes
///
/// # Errors
///
/// See [`read_zip`].
///
/// # Examples
///
/// Building a tiny archive in memory and reading it back (in real
/// code the bytes would come from disk or an HTTP download):
///
/// ```
/// use std::io::{Cursor, Write};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut cursor = Cursor::new(Vec::new());
///     let mut writer = zip::ZipWriter::new(&mut cursor);
///     let options = zip::write::SimpleFileOptions::default();
///     for (name, content) in [
///         (
///             "agency.txt",
///             "agency_name,agency_url,agency_timezone\n\
///              Demo,https://demo.example,UTC\n",
///         ),
///         ("stops.txt", "stop_id\nA\n"),
///         ("routes.txt", "route_id,route_type\nL1,3\n"),
///         ("trips.txt", "route_id,service_id,trip_id\nL1,daily,t0\n"),
///         ("stop_times.txt", "trip_id,stop_sequence,stop_id\nt0,1,A\n"),
///     ] {
///         writer.start_file(name, options)?;
///         writer.write_all(content.as_bytes())?;
///     }
///     writer.finish()?;
///     let bytes = cursor.into_inner();
///
///     let gtfs = gtfs_rs::parsers::zip::read_zip_bytes("demo.zip", &bytes)?;
///     assert_eq!(gtfs.trips.len(), 1);
///     Ok(())
/// }
/// ```
pub fn read_zip_bytes(archive_label: &str, bytes: &[u8]) -> Result<GtfsReference, ParseError> {
    let archive = match ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(e) => return Err(zip_err(archive_label, e)),
    };
    let mut source = ZipSource {
        label: archive_label.to_string(),
        archive,
    };
    read_tables(&mut source)
}

/// A zip archive as a [`TableSource`].
struct ZipSource<R: Read + Seek> {
    label: String,
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> TableSource for ZipSource<R> {
    fn open(&mut self, name: &str) -> Result<Option<Box<dyn Read + '_>>, ParseError> {
        match self.archive.by_name(name) {
            Ok(entry) => Ok(Some(Box::new(entry))),
            Err(ZipError::FileNotFound) => Ok(None),
            Err(e) => Err(zip_err(&self.label, e)),
        }
    }

    #[cfg(feature = "geojson")]
    fn locations_text(&mut self) -> Result<Option<String>, ParseError> {
        let mut entry = match self.archive.by_name("locations.geojson") {
            Ok(entry) => entry,
            Err(ZipError::FileNotFound) => return Ok(None),
            Err(e) => return Err(zip_err(&self.label, e)),
        };
        let mut text = String::new();
        if let Err(e) = entry.read_to_string(&mut text) {
            return Err(ParseError {
                file: "locations.geojson".to_string(),
                line: 0,
                field: None,
                kind: ParseErrorKind::Io(e),
            });
        }
        Ok(Some(text))
    }
}

/// Builds an archive-level error.
fn zip_err(label: &str, e: ZipError) -> ParseError {
    ParseError {
        file: label.to_string(),
        line: 0,
        field: None,
        kind: ParseErrorKind::Zip(e),
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::Write;

    use zip::CompressionMethod;
    use zip::write::SimpleFileOptions;

    use super::*;
    use crate::parsers::csv::test_support::FEED_DIR;
    #[cfg(feature = "geojson")]
    use crate::parsers::csv::test_support::FLEX_DIR;

    /// Packs every file of a fixture directory into an in-memory
    /// zip archive (stored, no compression).
    fn zip_dir(dir: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            writer.start_file(&name, options)?;
            writer.write_all(&fs::read(entry.path())?)?;
        }
        writer.finish()?;
        Ok(cursor.into_inner())
    }

    #[test]
    fn test_read_zip_bytes_sample_feed() -> Result<(), Box<dyn Error>> {
        let bytes = zip_dir(FEED_DIR)?;
        let gtfs = read_zip_bytes("sample_feed.zip", &bytes)?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.stops.len(), 9);
        assert_eq!(gtfs.routes.len(), 5);
        assert_eq!(gtfs.trips.len(), 11);
        assert_eq!(gtfs.stop_times.len(), 28);
        assert_eq!(gtfs.frequencies.len(), 11);
        Ok(())
    }

    #[test]
    fn test_read_zip_path_roundtrip() -> Result<(), Box<dyn Error>> {
        let bytes = zip_dir(FEED_DIR)?;
        let path = std::env::temp_dir().join(format!("gtfs_rs_test_{}.zip", std::process::id()));
        fs::write(&path, &bytes)?;
        let result = read_zip(&path);
        fs::remove_file(&path)?;
        let gtfs = result?;
        assert_eq!(gtfs.stops.len(), 9);
        Ok(())
    }

    #[test]
    fn test_missing_required_table_in_archive() -> Result<(), Box<dyn Error>> {
        // an archive with agency.txt only is not a valid feed
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("agency.txt", options)?;
        writer
            .write_all(b"agency_name,agency_url,agency_timezone\nDemo,https://x.example,UTC\n")?;
        writer.finish()?;

        let Err(err) = read_zip_bytes("broken.zip", &cursor.into_inner()) else {
            panic!("expected a missing-required-table error");
        };
        assert_eq!(err.file, "stops.txt");
        assert!(matches!(err.kind, ParseErrorKind::Io(_)));
        Ok(())
    }

    #[test]
    fn test_garbage_bytes_report_zip_error() {
        let Err(err) = read_zip_bytes("garbage.zip", b"not a zip archive at all") else {
            panic!("expected a zip format error");
        };
        assert_eq!(err.file, "garbage.zip");
        assert!(matches!(err.kind, ParseErrorKind::Zip(_)));
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_read_zip_flex_feed_with_locations() -> Result<(), Box<dyn Error>> {
        let bytes = zip_dir(FLEX_DIR)?;
        let gtfs = read_zip_bytes("flex_feed.zip", &bytes)?;
        assert_eq!(gtfs.locations.len(), 2);
        assert_eq!(gtfs.locations[0].location_id, "zone_a");
        assert_eq!(gtfs.booking_rules.len(), 1);
        Ok(())
    }
}
