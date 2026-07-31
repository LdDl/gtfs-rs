//! Reading functions for zipped GTFS feeds.

use std::fs::File;
use std::io::{self, Cursor, Read, Seek};
use std::path::Path;

use zip::ZipArchive;
use zip::result::ZipError;

use crate::parsers::feed::{TableSource, read_tables};
use crate::parsers::{ParseError, ParseErrorKind};
use crate::reference::GtfsReference;

/// Upper bound on the decompressed size of a single archive entry,
/// guarding against zip bombs. Generous because large real-world
/// `stop_times.txt` files exist.
const MAX_DECOMPRESSED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Reads a zipped GTFS feed from a file path into a
/// [`GtfsReference`].
///
/// The archive is expected to hold the tables at its root, as the
/// specification requires; an archive that keeps every entry inside
/// one shared top-level folder (as the macOS Finder "Compress"
/// command produces) is accepted as well, and entry names are
/// matched case-insensitively (`AGENCY.TXT` works). Each entry is
/// decompressed up to a 4 GiB cap to guard against zip bombs. The
/// required tables and the handling of optional ones are the same as
/// for [`read_dir`](crate::parsers::read_dir); with the `geojson`
/// cargo feature enabled, a bundled `locations.geojson` is read as
/// well.
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
    let label = path.display().to_string();
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
    let mut source = ZipSource::new(label, archive);
    read_tables(&mut source)
}

/// Reads a zipped GTFS feed from bytes already in memory - e.g. an
/// archive just downloaded over HTTP, without touching the disk.
///
/// The single-top-level-folder tolerance, the case-insensitive
/// entry-name matching and the per-entry decompression cap of
/// [`read_zip`] apply here as well.
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
    let mut source = ZipSource::new(archive_label.to_string(), archive);
    read_tables(&mut source)
}

/// A zip archive as a [`TableSource`].
struct ZipSource<R: Read + Seek> {
    label: String,
    archive: ZipArchive<R>,
    /// Common top-level folder shared by every data entry (with a
    /// trailing '/'), if the archive keeps the feed in a subfolder.
    prefix: Option<String>,
}

impl<R: Read + Seek> ZipSource<R> {
    fn new(label: String, archive: ZipArchive<R>) -> Self {
        let prefix = detect_prefix(&archive);
        ZipSource {
            label,
            archive,
            prefix,
        }
    }

    /// Resolves a table name to its archive index: the plain name
    /// first, then behind the detected common folder prefix, then -
    /// tolerating spec-violating but unambiguous archives - the
    /// same two forms compared ASCII-case-insensitively
    /// (e.g. `AGENCY.TXT`).
    fn entry_index(&self, name: &str) -> Option<usize> {
        if let Some(index) = self.archive.index_for_name(name) {
            return Some(index);
        }
        if let Some(prefix) = self.prefix.as_ref()
            && let Some(index) = self.archive.index_for_name(&format!("{prefix}{name}"))
        {
            return Some(index);
        }
        // file_names() has no index-order guarantee, so resolve the
        // canonical spelling first and look its index up by name
        let prefixed = self.prefix.as_ref().map(|p| format!("{p}{name}"));
        let found = self
            .archive
            .file_names()
            .find(|candidate| {
                candidate.eq_ignore_ascii_case(name)
                    || prefixed
                        .as_deref()
                        .is_some_and(|p| candidate.eq_ignore_ascii_case(p))
            })
            .map(str::to_string)?;
        self.archive.index_for_name(&found)
    }
}

impl<R: Read + Seek> TableSource for ZipSource<R> {
    fn open(&mut self, name: &str) -> Result<Option<Box<dyn Read + '_>>, ParseError> {
        let Some(index) = self.entry_index(name) else {
            return Ok(None);
        };
        match self.archive.by_index(index) {
            Ok(entry) => Ok(Some(Box::new(LimitedRead::new(
                entry,
                MAX_DECOMPRESSED_BYTES,
            )))),
            Err(e) => Err(zip_err(&self.label, e)),
        }
    }

    #[cfg(feature = "geojson")]
    fn locations_text(&mut self) -> Result<Option<String>, ParseError> {
        let Some(index) = self.entry_index("locations.geojson") else {
            return Ok(None);
        };
        let entry = match self.archive.by_index(index) {
            Ok(entry) => entry,
            Err(e) => return Err(zip_err(&self.label, e)),
        };
        let mut text = String::new();
        let mut limited = LimitedRead::new(entry, MAX_DECOMPRESSED_BYTES);
        if let Err(e) = limited.read_to_string(&mut text) {
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

/// Detects one top-level folder shared by every data entry of the
/// archive, as produced by e.g. the macOS Finder "Compress" command.
/// Directory entries and `__MACOSX/` junk are ignored; the result
/// keeps the trailing '/'. `None` when any entry sits at the root or
/// the entries disagree on the first path segment.
fn detect_prefix<R: Read + Seek>(archive: &ZipArchive<R>) -> Option<String> {
    let mut prefix: Option<&str> = None;
    for name in archive.file_names() {
        if name.ends_with('/') || name.starts_with("__MACOSX/") {
            continue;
        }
        let (first, _) = name.split_once('/')?;
        match prefix {
            Some(seen) if seen != first => return None,
            _ => prefix = Some(first),
        }
    }
    prefix.map(|p| format!("{p}/"))
}

/// A reader that fails once the wrapped stream tries to produce more
/// bytes than a fixed budget. Exceeding the budget is an error, not
/// silent truncation, so an oversized entry cannot slip through as
/// corrupted data downstream.
struct LimitedRead<R> {
    inner: R,
    limit: u64,
    remaining: u64,
}

impl<R: Read> LimitedRead<R> {
    fn new(inner: R, limit: u64) -> Self {
        LimitedRead {
            inner,
            limit,
            remaining: limit,
        }
    }
}

impl<R: Read> Read for LimitedRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // per the Read contract an empty buffer always reads Ok(0);
        // without this guard the probe below could consume a byte
        if buf.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut probe = [0u8; 1];
            if self.inner.read(&mut probe)? > 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "decompressed data exceeds {} bytes limit (possible zip bomb)",
                        self.limit
                    ),
                ));
            }
            return Ok(0);
        }
        let cap = match usize::try_from(self.remaining) {
            Ok(remaining) => buf.len().min(remaining),
            Err(_) => buf.len(),
        };
        let count = self.inner.read(&mut buf[..cap])?;
        self.remaining -= count as u64;
        Ok(count)
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
    use std::env;
    use std::error::Error;
    use std::fs;
    use std::io::Write;
    use std::process;

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

    /// Packs the given name/content pairs into an in-memory zip
    /// archive (stored, no compression).
    fn zip_entries(entries: &[(&str, &str)]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options)?;
            writer.write_all(content.as_bytes())?;
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
        let path = env::temp_dir().join(format!("gtfs_rs_test_{}.zip", process::id()));
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
        let bytes = zip_entries(&[(
            "agency.txt",
            "agency_name,agency_url,agency_timezone\nDemo,https://x.example,UTC\n",
        )])?;

        let Err(err) = read_zip_bytes("broken.zip", &bytes) else {
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

    #[test]
    fn test_limited_read_over_budget_is_an_error() {
        let mut reader = LimitedRead::new(&[7u8; 10][..], 5);
        let mut sink = Vec::new();
        let Err(err) = reader.read_to_end(&mut sink) else {
            panic!("expected an over-budget error");
        };
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("zip bomb"));
    }

    #[test]
    fn test_limited_read_empty_buffer_after_exhaustion() -> Result<(), Box<dyn Error>> {
        let mut reader = LimitedRead::new(&[7u8; 10][..], 5);
        let mut buf = [0u8; 5];
        reader.read_exact(&mut buf)?;
        // the Read contract: an empty buffer reads Ok(0) and must
        // not consume the probe byte
        assert_eq!(reader.read(&mut [])?, 0);
        let Err(err) = reader.read(&mut [0u8; 1]) else {
            panic!("expected an over-budget error");
        };
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        Ok(())
    }

    #[test]
    fn test_limited_read_within_budget() -> Result<(), Box<dyn Error>> {
        let mut sink = Vec::new();
        LimitedRead::new(&[7u8; 10][..], 10).read_to_end(&mut sink)?;
        assert_eq!(sink, [7u8; 10]);

        sink.clear();
        LimitedRead::new(&[7u8; 10][..], 64).read_to_end(&mut sink)?;
        assert_eq!(sink, [7u8; 10]);
        Ok(())
    }

    #[test]
    fn test_read_zip_bytes_feed_in_subfolder() -> Result<(), Box<dyn Error>> {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.add_directory("gtfs/", options)?;
        for (name, content) in [
            (
                "gtfs/agency.txt",
                "agency_name,agency_url,agency_timezone\nDemo,https://demo.example,UTC\n",
            ),
            ("gtfs/stops.txt", "stop_id\nA\n"),
            ("gtfs/routes.txt", "route_id,route_type\nL1,3\n"),
            (
                "gtfs/trips.txt",
                "route_id,service_id,trip_id\nL1,daily,t0\n",
            ),
            (
                "gtfs/stop_times.txt",
                "trip_id,stop_sequence,stop_id\nt0,1,A\n",
            ),
            ("__MACOSX/gtfs/._agency.txt", "finder junk"),
        ] {
            writer.start_file(name, options)?;
            writer.write_all(content.as_bytes())?;
        }
        writer.finish()?;

        let gtfs = read_zip_bytes("subfolder.zip", &cursor.into_inner())?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.agencies[0].agency_name, "Demo");
        assert_eq!(gtfs.stops.len(), 1);
        assert_eq!(gtfs.routes.len(), 1);
        assert_eq!(gtfs.trips.len(), 1);
        assert_eq!(gtfs.stop_times.len(), 1);
        Ok(())
    }

    #[test]
    fn test_uppercase_entry_names_are_tolerated() -> Result<(), Box<dyn Error>> {
        let bytes = zip_entries(&[
            (
                "AGENCY.TXT",
                "agency_name,agency_url,agency_timezone\nDemo,https://demo.example,UTC\n",
            ),
            ("Stops.txt", "stop_id\nA\n"),
            ("routes.txt", "route_id,route_type\nL1,3\n"),
            ("trips.txt", "route_id,service_id,trip_id\nL1,daily,t0\n"),
            ("stop_times.txt", "trip_id,stop_sequence,stop_id\nt0,1,A\n"),
        ])?;

        let gtfs = read_zip_bytes("uppercase.zip", &bytes)?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.agencies[0].agency_name, "Demo");
        assert_eq!(gtfs.stops.len(), 1);
        Ok(())
    }

    #[test]
    fn test_mixed_root_and_folder_entries_use_root() -> Result<(), Box<dyn Error>> {
        let bytes = zip_entries(&[
            (
                "agency.txt",
                "agency_name,agency_url,agency_timezone\nRoot,https://root.example,UTC\n",
            ),
            ("stops.txt", "stop_id\nA\n"),
            ("routes.txt", "route_id,route_type\nL1,3\n"),
            ("trips.txt", "route_id,service_id,trip_id\nL1,daily,t0\n"),
            ("stop_times.txt", "trip_id,stop_sequence,stop_id\nt0,1,A\n"),
            (
                "nested/agency.txt",
                "agency_name,agency_url,agency_timezone\nNested,https://n.example,UTC\n",
            ),
        ])?;

        let gtfs = read_zip_bytes("mixed.zip", &bytes)?;
        assert_eq!(gtfs.agencies.len(), 1);
        assert_eq!(gtfs.agencies[0].agency_name, "Root");
        Ok(())
    }
}
