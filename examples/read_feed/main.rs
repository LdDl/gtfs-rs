//! Reads the bundled sample GTFS feed with the CSV parser: a single
//! table first, then the whole unpacked feed directory.
//!
//! Requires the `parse` feature. Run from the repository root:
//!
//! ```sh
//! cargo run --example read_feed --features parse
//! ```

use gtfs_rs::parsers::{ParseError, csv};
use gtfs_rs::{Agency, GtfsReference};

/// Reads a single table - no other files required.
///
/// The table kind is chosen by the TYPE, not by the file name: the
/// compiler infers `read_path::<Agency>` from the return type. The
/// explicit form is `csv::read_path::<Agency>(path)`, and the path
/// may point to a file with any name.
fn load_agencies(path: &str) -> Result<Vec<Agency>, ParseError> {
    csv::read_path(path)
}

/// Reads a whole unpacked feed directory.
fn load_feed(dir: &str) -> Result<GtfsReference, ParseError> {
    csv::read_dir(dir)
}

fn main() {
    match load_agencies("tests/data/sample_feed/agency.txt") {
        Ok(agencies) => println!("operated by {}", agencies[0].agency_name),
        Err(e) => eprintln!("failed to read agencies: {e}"),
    }

    match load_feed("tests/data/sample_feed") {
        Ok(gtfs) => {
            println!(
                "loaded {} routes, {} trips, {} stop times",
                gtfs.routes.len(),
                gtfs.trips.len(),
                gtfs.stop_times.len(),
            );
            for trip in gtfs.trips_of_route("AB") {
                println!("route AB trip: {}", trip.trip_id);
            }
        }
        // errors point at the exact record, e.g.:
        // "stop_times.txt, line 12401, field `arrival_time`:
        //  invalid GTFS time value: '8h00' (expected HH:MM:SS)"
        Err(e) => eprintln!("failed to load feed: {e}"),
    }
}
