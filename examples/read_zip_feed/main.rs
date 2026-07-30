//! Packs the bundled sample feed into an in-memory zip archive and
//! reads it back with the zip parser - no archive file is stored in
//! this crate repository.
//!
//! Requires the `zip` feature. Run from the repository root:
//!
//! ```sh
//! cargo run --example read_zip_feed --features zip
//! ```

use std::error::Error;
use std::fs;
use std::io::{Cursor, Write};

use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

/// Packs every file of a directory into an in-memory zip archive.
fn pack_dir(dir: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
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

fn main() {
    let bytes = match pack_dir("tests/data/sample_feed") {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to pack the sample feed: {e}");
            return;
        }
    };
    println!("packed the sample feed into {} bytes", bytes.len());

    // in real code the bytes would come from disk or
    // an HTTP download, whatever;
    // the module path is spelled out to avoid clashing with the `zip`
    // crate imported above
    match gtfs_rs::parsers::zip::read_zip_bytes("sample_feed.zip", &bytes) {
        Ok(gtfs) => println!(
            "read back {} stops, {} routes, {} trips",
            gtfs.stops.len(),
            gtfs.routes.len(),
            gtfs.trips.len(),
        ),
        Err(e) => eprintln!("failed to read the archive: {e}"),
    }
}
