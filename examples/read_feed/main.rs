//! Reads the bundled sample GTFS feed with the CSV parser: a single
//! table first, then the whole unpacked feed directory.
//!
//! Requires the `parse` feature. Run from the repository root:
//!
//! ```sh
//! cargo run --example read_feed --features parse
//! ```

use gtfs_rs::parsers;
use gtfs_rs::parsers::csv;

fn main() {
    // one table via its named shortcut - no other files required
    match csv::read_agencies("tests/data/sample_feed/agency.txt") {
        Ok(agencies) => println!("operated by {}", agencies[0].agency_name),
        Err(e) => eprintln!("failed to read agencies: {e}"),
    }

    // or a whole unpacked feed directory at once
    match parsers::read_dir("tests/data/sample_feed") {
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
